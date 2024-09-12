use conjunto_core::{
    delegation_inconsistency::DelegationInconsistency,
    delegation_record::DelegationRecord,
    delegation_record_parser::DelegationRecordParser, AccountProvider,
};
use dlp::pda;
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::{
    account_chain_snapshot::AccountChainSnapshot,
    account_chain_state::AccountChainState,
    accounts::predicates::is_owned_by_delegation_program,
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
        let delegation_pda = pda::delegation_record_pda_from_pubkey(pubkey);
        // Fetch the current chain state for revelant accounts (all at once)
        let (at_slot, mut fetched_accounts) = self
            .account_provider
            .get_multiple_accounts(&[delegation_pda, *pubkey])
            .await?;
        // Parse the result into an AccountChainState
        self.try_parse_chain_state_of_fetched_accounts(
            pubkey,
            delegation_pda,
            &mut fetched_accounts,
        )
        .map(|chain_state| AccountChainSnapshot {
            pubkey: *pubkey,
            at_slot,
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
        let account = match fetched_accounts.remove(1) {
            Some(account) => account,
            None => return Ok(AccountChainState::NewAccount),
        };
        // Check if the base account is locked by the delegation program
        if !is_owned_by_delegation_program(&account) {
            return Ok(AccountChainState::Undelegated { account });
        }
        // Verify the delegation account exists and is owned by the delegation program
        self.read_delegated_account_from_fetched_account(
            account,
            delegation_pda,
            fetched_accounts.remove(0),
        )
    }

    fn read_delegated_account_from_fetched_account(
        &self,
        account: Account,
        delegation_pda: Pubkey,
        fetched_delegation_account: Option<Account>,
    ) -> LockboxResult<AccountChainState> {
        let delegation_account = match fetched_delegation_account {
            None => {
                return Ok(AccountChainState::Inconsistent {
                    account,
                    delegation_pda,
                    delegation_inconsistencies: vec![
                        DelegationInconsistency::AccountNotFound,
                    ],
                })
            }
            Some(account) => account,
        };
        let mut delegation_inconsistencies = vec![];
        if !is_owned_by_delegation_program(&delegation_account) {
            delegation_inconsistencies
                .push(DelegationInconsistency::AccountInvalidOwner);
        }
        match self
            .delegation_record_parser
            .try_parse(&delegation_account.data)
        {
            Ok(delegation_record) => {
                if delegation_inconsistencies.is_empty() {
                    Ok(AccountChainState::Delegated {
                        account,
                        delegation_pda,
                        delegation_record,
                    })
                } else {
                    Ok(AccountChainState::Inconsistent {
                        account,
                        delegation_pda,
                        delegation_inconsistencies,
                    })
                }
            }
            Err(err) => {
                delegation_inconsistencies.push(
                    DelegationInconsistency::RecordAccountDataInvalid(
                        err.to_string(),
                    ),
                );
                Ok(AccountChainState::Inconsistent {
                    account,
                    delegation_pda,
                    delegation_inconsistencies,
                })
            }
        }
    }
}
