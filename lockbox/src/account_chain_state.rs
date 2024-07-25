use conjunto_core::{
    AccountProvider, DelegationRecord, DelegationRecordParser,
};
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
};
use dlp::pda;
use serde::{Deserialize, Serialize};
use solana_sdk::{account::Account, clock::Slot, pubkey::Pubkey};

use crate::{
    accounts::predicates::is_owned_by_delegation_program,
    delegation_account::{DelegationAccount, DelegationRecordParserImpl},
    errors::{LockboxError, LockboxResult},
    LockConfig, LockInconsistency,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AccountChainStateSnapshot {
    pub from_slot: Slot,
    pub chain_state: AccountChainState,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum AccountChainState {
    /// The account is not present on chain and thus not delegated either
    /// In this case we assume that this is an account that temporarily exists
    /// on the ephemeral validator and will not have to be undelegated.
    /// However in the short term we don't allow new accounts to be created inside
    /// the validator which means that we reject any transactions that attempt to do so
    NewAccount,
    /// The account was found on chain and is not delegated and therefore should
    /// not be used as writable on the ephemeral validator unless otherwise allowed
    /// via the `require_delegation=false` setting.
    Undelegated { account: Account },
    /// The account was found on chain in a proper delegated state which means we
    /// also found the related accounts like the buffer and delegation
    /// NOTE: commit records and state diff accountsk are not checked since an
    /// account is delegated and then used before the validator commits a state change.
    Delegated {
        account: Account,
        delegated_id: Pubkey,
        delegation_pda: Pubkey,
        config: LockConfig,
    },
    /// The account was found on chain and was partially delegated which means that
    /// it is owned by the delegation program but one or more of the related
    /// accounts were either not present or not owned by the delegation program
    Inconsistent {
        account: Account,
        delegated_id: Pubkey,
        delegation_pda: Pubkey,
        inconsistencies: Vec<LockInconsistency>,
    },
}

impl AccountChainState {
    pub fn is_new(&self) -> bool {
        matches!(self, AccountChainState::NewAccount)
    }

    pub fn is_delegated(&self) -> bool {
        matches!(self, AccountChainState::Delegated { .. })
    }

    pub fn is_undelegated(&self) -> bool {
        matches!(self, AccountChainState::Undelegated { .. })
    }

    pub fn is_inconsistent(&self) -> bool {
        matches!(self, AccountChainState::Inconsistent { .. })
    }

    pub fn lock_config(&self) -> Option<LockConfig> {
        match self {
            AccountChainState::Delegated { config, .. } => Some(config.clone()),
            _ => None,
        }
    }

    pub fn into_account(self) -> Option<Account> {
        match self {
            AccountChainState::NewAccount => None,
            AccountChainState::Undelegated { account } => Some(account),
            AccountChainState::Delegated { account, .. } => Some(account),
            AccountChainState::Inconsistent { account, .. } => Some(account),
        }
    }
}

pub struct AccountChainStateProvider<
    T: AccountProvider,
    U: DelegationRecordParser,
> {
    account_provider: T,
    delegation_record_parser: U,
}

impl<T: AccountProvider, U: DelegationRecordParser>
    AccountChainStateProvider<T, U>
{
    pub fn new(
        config: RpcProviderConfig,
    ) -> AccountChainStateProvider<RpcAccountProvider, DelegationRecordParserImpl>
    {
        let rpc_account_provider = RpcAccountProvider::new(config);
        let delegation_record_parser = DelegationRecordParserImpl;
        AccountChainStateProvider::with_provider_and_parser(
            rpc_account_provider,
            delegation_record_parser,
        )
    }

    pub fn new_with_parser(
        config: RpcProviderConfig,
        delegation_record_parser: U,
    ) -> AccountChainStateProvider<RpcAccountProvider, U> {
        let rpc_account_provider = RpcAccountProvider::new(config);
        AccountChainStateProvider::with_provider_and_parser(
            rpc_account_provider,
            delegation_record_parser,
        )
    }

    pub fn with_provider_and_parser(
        account_provider: T,
        delegation_record_parser: U,
    ) -> Self {
        Self {
            account_provider,
            delegation_record_parser,
        }
    }

    pub async fn try_fetch_chain_state_snapshot_of_pubkey(
        &self,
        pubkey: &Pubkey,
    ) -> LockboxResult<AccountChainStateSnapshot> {
        let delegation_pda = pda::delegation_record_pda_from_pubkey(pubkey);
        // Fetch the current chain state for revelant accounts (all at once)
        let (from_slot, mut fetched_accounts) = self
            .account_provider
            .get_multiple_accounts(&[delegation_pda, *pubkey])
            .await?;
        // Parse the result into an AccountChainState
        self.try_parse_chain_state_of_fetched_accounts(
            pubkey,
            delegation_pda,
            &mut fetched_accounts,
        )
        .map(|chain_state| AccountChainStateSnapshot {
            from_slot,
            chain_state,
        })
    }

    fn try_parse_chain_state_of_fetched_accounts(
        &self,
        pubkey: &Pubkey,
        delegation_pda: Pubkey,
        fetched_accounts: &mut Vec<Option<Account>>,
    ) -> LockboxResult<AccountChainState> {
        // If something went wrong in the fetch we stop, we should receive 2 accounts exactly every time
        if fetched_accounts.len() != 2 {
            return Err(LockboxError::InvalidFetch {
                fetched_pubkeys: vec![*pubkey, delegation_pda],
                fetched_accounts: fetched_accounts.clone(),
            });
        }
        // Check if the base account exists (it should always be account at index[1])
        let base_account = match fetched_accounts.remove(1) {
            Some(account) => account,
            None => return Ok(AccountChainState::NewAccount),
        };
        // Check if the base account is locked by the delegation program
        if !is_owned_by_delegation_program(&base_account) {
            return Ok(AccountChainState::Undelegated {
                account: base_account,
            });
        }
        // Verify the delegation account exists and is owned by the delegation program
        match DelegationAccount::try_from_fetched_account(
            fetched_accounts.remove(0),
            &self.delegation_record_parser,
        )? {
            DelegationAccount::Valid(DelegationRecord {
                commit_frequency,
                owner,
            }) => Ok(AccountChainState::Delegated {
                account: base_account,
                delegated_id: *pubkey,
                delegation_pda,
                config: LockConfig {
                    commit_frequency,
                    owner,
                },
            }),
            DelegationAccount::Invalid(inconsistencies) => {
                Ok(AccountChainState::Inconsistent {
                    account: base_account,
                    delegated_id: *pubkey,
                    delegation_pda,
                    inconsistencies,
                })
            }
        }
    }
}
