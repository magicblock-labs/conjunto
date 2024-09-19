use conjunto_core::{
    delegation_inconsistency::DelegationInconsistency,
    delegation_record_parser::DelegationRecordParser, AccountProvider,
};
use dlp::{consts::DELEGATION_PROGRAM_ID, pda};
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::{
    account_chain_snapshot::AccountChainSnapshot,
    account_chain_state::AccountChainState,
    errors::{LockboxError, LockboxResult},
};

pub struct AccountChainSnapshotProvider<
    T: AccountProvider,
    U: DelegationRecordParser,
> {
    account_provider: T,
    delegation_record_parser: U,
}

impl<T: AccountProvider, U: DelegationRecordParser>
    AccountChainSnapshotProvider<T, U>
{
    pub fn new(account_provider: T, delegation_record_parser: U) -> Self {
        Self {
            account_provider,
            delegation_record_parser,
        }
    }

    pub async fn try_fetch_chain_snapshot_of_pubkey(
        &self,
        pubkey: &Pubkey,
    ) -> LockboxResult<AccountChainSnapshot> {
        if pubkey.is_on_curve() {
            self.try_fetch_chain_snapshot_of_wallet(pubkey).await
        } else {
            self.try_fetch_chain_snapshot_of_pda(pubkey).await
        }
    }

    async fn try_fetch_chain_snapshot_of_wallet(
        &self,
        pubkey: &Pubkey,
    ) -> LockboxResult<AccountChainSnapshot> {
        let (at_slot, account) =
            self.account_provider.get_account(pubkey).await?;
        Ok(AccountChainSnapshot {
            pubkey: *pubkey,
            at_slot,
            chain_state: AccountChainState::Wallet {
                account: account.unwrap_or_default(),
            },
        })
    }

    async fn try_fetch_chain_snapshot_of_pda(
        &self,
        pubkey: &Pubkey,
    ) -> LockboxResult<AccountChainSnapshot> {
        let delegation_pda = pda::delegation_record_pda_from_pubkey(pubkey);
        // Fetch the current chain state for revelant accounts (all at once)
        let (at_slot, mut fetched_accounts) = self
            .account_provider
            .get_multiple_accounts(&[*pubkey, delegation_pda])
            .await?;
        // If something went wrong in the fetch we stop, we should receive 2 accounts exactly every time
        if fetched_accounts.len() != 2 {
            return Err(LockboxError::InvalidFetch {
                fetched_pubkeys: vec![*pubkey, delegation_pda],
                fetched_accounts,
            });
        }
        // Extract the accounts we just fetched
        let account = fetched_accounts.swap_remove(0);
        let delegation_record_account = fetched_accounts.swap_remove(0);
        // Parse the result into an AccountChainState
        let chain_state = self.try_into_chain_state_of_pda_fetched_accounts(
            account,
            delegation_record_account,
        )?;
        // Build the AccountChainSnapshot
        Ok(AccountChainSnapshot {
            pubkey: *pubkey,
            at_slot,
            chain_state,
        })
    }

    fn try_into_chain_state_of_pda_fetched_accounts(
        &self,
        account: Option<Account>,
        delegation_record_account: Option<Account>,
    ) -> LockboxResult<AccountChainState> {
        // Check if the base account exists
        let account = match account {
            None => {
                return Ok(AccountChainState::Undelegated {
                    account: Account::default(),
                    delegation_inconsistency:
                        DelegationInconsistency::AccountNotFound,
                })
            }
            Some(account) => account,
        };
        // Check if the base account is locked by the delegation program
        if !is_owned_by_delegation_program(&account) {
            return Ok(AccountChainState::Undelegated {
                account,
                delegation_inconsistency:
                    DelegationInconsistency::AccountInvalidOwner,
            });
        }
        // Check if the delegation record exists
        let delegation_record_account = match delegation_record_account {
            None => {
                return Ok(AccountChainState::Undelegated {
                    account,
                    delegation_inconsistency:
                        DelegationInconsistency::DelegationRecordNotFound,
                })
            }
            Some(account) => account,
        };
        // Check if the delegation record is owned by the delegation program
        if !is_owned_by_delegation_program(&delegation_record_account) {
            return Ok(AccountChainState::Undelegated {
                account,
                delegation_inconsistency:
                    DelegationInconsistency::DelegationRecordInvalidOwner,
            });
        }
        // Try to parse the delegation record's data
        match self
            .delegation_record_parser
            .try_parse(&delegation_record_account.data)
        {
            Ok(delegation_record) => Ok(AccountChainState::Delegated {
                account,
                delegation_record,
            }),
            Err(err) => Ok(AccountChainState::Undelegated {
                account,
                delegation_inconsistency:
                    DelegationInconsistency::DelegationRecordDataInvalid(
                        err.to_string(),
                    ),
            }),
        }
    }
}

fn is_owned_by_delegation_program(account: &Account) -> bool {
    account.owner == DELEGATION_PROGRAM_ID
}
