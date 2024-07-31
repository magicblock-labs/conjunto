use conjunto_lockbox::AccountChainSnapshot;
pub use conjunto_lockbox::LockConfig;
use solana_sdk::{account::Account, clock::Slot, pubkey::Pubkey};

use crate::{
    errors::TranswiseError,
    transaction_accounts_metas::TransactionAccountsMetas,
};

#[derive(Debug)]
pub struct ValidateAccountsConfig {
    pub allow_new_accounts: bool,
    pub require_delegation: bool,
}

impl Default for ValidateAccountsConfig {
    fn default() -> Self {
        Self {
            allow_new_accounts: false,
            require_delegation: true,
        }
    }
}

#[derive(Debug)]
pub struct ValidatedReadonlyAccount {
    pub pubkey: Pubkey,
    pub account: Option<Account>,
    pub at_slot: Slot,
}

impl From<AccountChainSnapshot> for ValidatedReadonlyAccount {
    fn from(chain_snapshot: AccountChainSnapshot) -> Self {
        Self {
            pubkey: chain_snapshot.pubkey,
            account: chain_snapshot.chain_state.into_account(),
            at_slot: chain_snapshot.at_slot,
        }
    }
}

#[derive(Debug)]
pub struct ValidatedWritableAccount {
    pub pubkey: Pubkey,
    pub lock_config: Option<LockConfig>,
    pub account: Option<Account>,
    pub at_slot: Slot,
    pub is_payer: bool,
}

impl From<(AccountChainSnapshot, Pubkey)> for ValidatedWritableAccount {
    fn from((chain_snapshot, payer): (AccountChainSnapshot, Pubkey)) -> Self {
        Self {
            pubkey: chain_snapshot.pubkey,
            lock_config: chain_snapshot.chain_state.lock_config(),
            account: chain_snapshot.chain_state.into_account(),
            at_slot: chain_snapshot.at_slot,
            is_payer: chain_snapshot.pubkey == payer,
        }
    }
}

#[derive(Debug)]
pub struct ValidatedAccounts {
    pub readonly: Vec<ValidatedReadonlyAccount>,
    pub writable: Vec<ValidatedWritableAccount>,
}

impl TryFrom<(TransactionAccountsMetas, &ValidateAccountsConfig)>
    for ValidatedAccounts
{
    type Error = TranswiseError;

    fn try_from(
        (metas, config): (TransactionAccountsMetas, &ValidateAccountsConfig),
    ) -> Result<Self, Self::Error> {
        // We put the following constraint on the config:
        //
        // A) the validator CAN create new accounts and can clone ANY account from chain, even non-delegated ones (permissive mode)
        // B) the validator CANNOT create new accounts and can ONLY clone delegated accounts from chain (strict mode)
        // C) the validator CANNOT create new accounts and can clone ANY account from chain, even non-delegated ones (frozen mode)
        //
        // This means we disallow the following remaining case:
        //
        // D) the validator CAN create new accounts and can ONLY clone delegated accounts from chain
        // This edge case is difficult to handle properly and most likely not what the user intended for the following reason:
        // If a transaction has a writable account that does not exist on chain by definition that account is not delegated
        // and if we accept it as a writable it now violates the delegation requirement.
        // In short this is a conflicting requirement that we don't allow.
        if config.require_delegation && config.allow_new_accounts {
            return Err(TranswiseError::ValidateAccountsConfigIsInvalid(
                format!("{:?}", config),
            ));
        }

        // First, a quick guard against accounts that are inconsistently delegated
        let writable_inconsistent_pubkeys =
            metas.writable_inconsistent_pubkeys();
        let has_writable_inconsistent =
            !writable_inconsistent_pubkeys.is_empty();
        if has_writable_inconsistent {
            return Err(TranswiseError::WritablesIncludeInconsistentAccounts {
                writable_inconsistent_pubkeys,
            });
        }

        // If we are not allowed to create new accounts, we need to guard against them
        if !config.allow_new_accounts {
            let writable_new_pubkeys = metas.writable_new_pubkeys();
            let has_writable_new = !writable_new_pubkeys.is_empty();
            if has_writable_new {
                return Err(TranswiseError::WritablesIncludeNewAccounts {
                    writable_new_pubkeys,
                });
            }
        }

        // If we require delegation:
        // We need make sure that all writables are delegated
        // Except we don't worry about the payer, because it doesn't contain data, it just need to sign
        if config.require_delegation {
            let writable_undelegated_non_payer_pubkeys =
                metas.writable_undelegated_non_payer_pubkeys();
            let has_writable_undelegated_non_payer =
                !writable_undelegated_non_payer_pubkeys.is_empty();
            if has_writable_undelegated_non_payer {
                let writable_delegated_pubkeys =
                    metas.writable_delegated_pubkeys();
                return Err(TranswiseError::NotAllWritablesDelegated {
                    writable_delegated_pubkeys,
                    writable_undelegated_non_payer_pubkeys,
                });
            }
        }

        let validated_readonly_accounts = metas
            .readonly
            .into_iter()
            .map(ValidatedReadonlyAccount::from)
            .collect();
        let validated_writable_accounts = metas
            .writable
            .into_iter()
            .map(|chain_snapshot| {
                ValidatedWritableAccount::from((chain_snapshot, metas.payer))
            })
            .collect();

        // Done
        Ok(ValidatedAccounts {
            readonly: validated_readonly_accounts,
            writable: validated_writable_accounts,
        })
    }
}

#[cfg(test)]
mod tests {
    use conjunto_core::CommitFrequency;
    use conjunto_lockbox::{
        AccountChainSnapshot, AccountChainState, LockConfig,
    };
    use conjunto_test_tools::accounts::{
        account_owned_by_delegation_program, account_owned_by_system_program,
    };

    use super::*;
    use crate::errors::TranswiseResult;

    fn config_strict() -> ValidateAccountsConfig {
        ValidateAccountsConfig {
            allow_new_accounts: false,
            require_delegation: true,
        }
    }

    fn config_permissive() -> ValidateAccountsConfig {
        ValidateAccountsConfig {
            allow_new_accounts: true,
            require_delegation: false,
        }
    }

    fn chain_snapshot_delegated() -> AccountChainSnapshot {
        AccountChainSnapshot {
            at_slot: 42,
            chain_state: AccountChainState::Delegated {
                account: account_owned_by_delegation_program(),
                delegated_id: Pubkey::new_unique(),
                delegation_pda: Pubkey::new_unique(),
                config: LockConfig {
                    commit_frequency: CommitFrequency::Millis(1_000),
                    owner: Pubkey::new_unique(),
                },
            },
        }
    }

    fn chain_snapshot_undelegated() -> AccountChainSnapshot {
        AccountChainSnapshot {
            at_slot: 42,
            chain_state: AccountChainState::Undelegated {
                account: account_owned_by_system_program(),
            },
        }
    }

    fn chain_snapshot_new_account() -> AccountChainSnapshot {
        AccountChainSnapshot {
            at_slot: 42,
            chain_state: AccountChainState::NewAccount,
        }
    }

    fn chain_snapshot_inconsistent() -> AccountChainSnapshot {
        AccountChainSnapshot {
            at_slot: 42,
            chain_state: AccountChainState::Inconsistent {
                account: account_owned_by_system_program(),
                delegated_id: Pubkey::new_unique(),
                delegation_pda: Pubkey::new_unique(),
                inconsistencies: vec![],
            },
        }
    }

    fn readonly_pubkeys(vas: &ValidatedAccounts) -> Vec<Pubkey> {
        vas.readonly.iter().map(|x| x.pubkey).collect()
    }

    fn writable_pubkeys(vas: &ValidatedAccounts) -> Vec<Pubkey> {
        vas.writable.iter().map(|x| x.pubkey).collect()
    }

    #[test]
    fn test_two_readonly_undelegated_and_two_writable_delegated_and_payer() {
        let readonly_undelegated_id1 = Pubkey::new_unique();
        let readonly_undelegated_id2 = Pubkey::new_unique();
        let writable_delegated_id1 = Pubkey::new_unique();
        let writable_delegated_id2 = Pubkey::new_unique();
        let writable_undelegated_payer_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id1,
            chain_snapshot: chain_snapshot_undelegated(),
        };
        let meta2 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id2,
            chain_snapshot: chain_snapshot_undelegated(),
        };
        let meta3 = TransactionAccountMeta::Writable {
            pubkey: writable_delegated_id1,
            chain_snapshot: chain_snapshot_delegated(),
            is_payer: false,
        };
        let meta4 = TransactionAccountMeta::Writable {
            pubkey: writable_delegated_id2,
            chain_snapshot: chain_snapshot_delegated(),
            is_payer: false,
        };
        let meta5 = TransactionAccountMeta::Writable {
            pubkey: writable_undelegated_payer_id,
            chain_snapshot: chain_snapshot_undelegated(),
            is_payer: true,
        };

        let vas: ValidatedAccounts = (
            TransactionAccountsMetas(vec![meta1, meta2, meta3, meta4, meta5]),
            &config_strict(),
        )
            .try_into()
            .unwrap();

        assert_eq!(
            readonly_pubkeys(&vas),
            vec![readonly_undelegated_id1, readonly_undelegated_id2]
        );
        assert_eq!(
            writable_pubkeys(&vas),
            vec![
                writable_delegated_id1,
                writable_delegated_id2,
                writable_undelegated_payer_id
            ]
        );
    }

    #[test]
    fn test_one_readonly_undelegated_and_one_writable_undelegated_fail() {
        let readonly_undelegated_id = Pubkey::new_unique();
        let writable_undelegated_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            chain_snapshot: chain_snapshot_undelegated(),
        };
        let meta2 = TransactionAccountMeta::Writable {
            pubkey: writable_undelegated_id,
            chain_snapshot: chain_snapshot_undelegated(),
            is_payer: false,
        };

        let res: TranswiseResult<ValidatedAccounts> = (
            TransactionAccountsMetas(vec![meta1, meta2]),
            &config_strict(),
        )
            .try_into();

        assert!(res.is_err());
    }

    #[test]
    fn test_one_readonly_undelegated_and_payer() {
        let readonly_undelegated_id = Pubkey::new_unique();
        let writable_undelegated_payer_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            chain_snapshot: chain_snapshot_undelegated(),
        };
        let meta2 = TransactionAccountMeta::Writable {
            pubkey: writable_undelegated_payer_id,
            chain_snapshot: chain_snapshot_undelegated(),
            is_payer: true,
        };

        let vas: ValidatedAccounts = (
            TransactionAccountsMetas(vec![meta1, meta2]),
            &config_strict(),
        )
            .try_into()
            .unwrap();

        assert_eq!(readonly_pubkeys(&vas), vec![readonly_undelegated_id]);
        assert_eq!(writable_pubkeys(&vas), vec![writable_undelegated_payer_id]);
    }

    #[test]
    fn test_one_readonly_undelegated_and_one_writable_inconsistent() {
        let readonly_undelegated_id = Pubkey::new_unique();
        let writable_inconsistent_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            chain_snapshot: chain_snapshot_undelegated(),
        };
        let meta2 = TransactionAccountMeta::Writable {
            pubkey: writable_inconsistent_id,
            chain_snapshot: chain_snapshot_inconsistent(),
            is_payer: false,
        };

        let res: TranswiseResult<ValidatedAccounts> = (
            TransactionAccountsMetas(vec![meta1, meta2]),
            &config_strict(),
        )
            .try_into();

        assert!(res.is_err());
    }

    #[test]
    fn test_one_readonly_new_account_and_one_payer() {
        let readonly_new_account_id = Pubkey::new_unique();
        let writable_undelegated_payer_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_new_account_id,
            chain_snapshot: chain_snapshot_new_account(),
        };
        let meta2 = TransactionAccountMeta::Writable {
            pubkey: writable_undelegated_payer_id,
            chain_snapshot: chain_snapshot_delegated(),
            is_payer: true,
        };

        let vas: ValidatedAccounts = (
            TransactionAccountsMetas(vec![meta1, meta2]),
            &config_strict(),
        )
            .try_into()
            .unwrap();

        // While this is a new account, it's a readonly so we don't need to write to it, so it's valid
        // However it cannot be cloned, but that last bit of clone filtering will be done in the validator
        assert_eq!(readonly_pubkeys(&vas), vec![readonly_new_account_id]);
        assert_eq!(writable_pubkeys(&vas), vec![writable_undelegated_payer_id]);
    }

    #[test]
    fn test_one_readonly_undelegated_and_one_writable_new_account() {
        let readonly_undelegated_id = Pubkey::new_unique();
        let writable_new_account_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            chain_snapshot: chain_snapshot_undelegated(),
        };
        let meta2 = TransactionAccountMeta::Writable {
            pubkey: writable_new_account_id,
            chain_snapshot: chain_snapshot_new_account(),
            is_payer: false,
        };

        let res: TranswiseResult<ValidatedAccounts> = (
            TransactionAccountsMetas(vec![meta1, meta2]),
            &config_strict(),
        )
            .try_into();

        assert!(res.is_err());
    }

    #[test]
    fn test_one_readonly_undelegated_and_one_writable_new_account_and_one_writable_undelegated_while_permissive(
    ) {
        let readonly_undelegated_id1 = Pubkey::new_unique();
        let writable_new_account_id = Pubkey::new_unique();
        let writable_undelegated_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id1,
            chain_snapshot: chain_snapshot_undelegated(),
        };
        let meta2 = TransactionAccountMeta::Writable {
            pubkey: writable_new_account_id,
            chain_snapshot: chain_snapshot_new_account(),
            is_payer: false,
        };
        let meta3 = TransactionAccountMeta::Writable {
            pubkey: writable_undelegated_id,
            chain_snapshot: chain_snapshot_delegated(),
            is_payer: false,
        };

        let vas: ValidatedAccounts = (
            TransactionAccountsMetas(vec![meta1, meta2, meta3]),
            &config_permissive(),
        )
            .try_into()
            .unwrap();

        assert_eq!(readonly_pubkeys(&vas), vec![readonly_undelegated_id1]);
        assert_eq!(
            writable_pubkeys(&vas),
            vec![writable_new_account_id, writable_undelegated_id]
        );
    }

    #[test]
    fn test_one_of_each_valid_type() {
        let readonly_new_account_id = Pubkey::new_unique();
        let readonly_undelegated_id = Pubkey::new_unique();
        let readonly_delegated_id = Pubkey::new_unique();
        let readonly_inconsistent_id = Pubkey::new_unique();
        let writable_delegated_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_new_account_id,
            chain_snapshot: chain_snapshot_new_account(),
        };
        let meta2 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            chain_snapshot: chain_snapshot_undelegated(),
        };
        let meta3 = TransactionAccountMeta::Readonly {
            pubkey: readonly_delegated_id,
            chain_snapshot: chain_snapshot_delegated(),
        };
        let meta4 = TransactionAccountMeta::Readonly {
            pubkey: readonly_inconsistent_id,
            chain_snapshot: chain_snapshot_inconsistent(),
        };
        let meta5 = TransactionAccountMeta::Writable {
            pubkey: writable_delegated_id,
            chain_snapshot: chain_snapshot_delegated(),
            is_payer: false,
        };

        let vas: ValidatedAccounts = (
            TransactionAccountsMetas(vec![meta1, meta2, meta3, meta4, meta5]),
            &config_strict(),
        )
            .try_into()
            .unwrap();

        assert_eq!(vas.readonly.len(), 4);
        assert_eq!(vas.writable.len(), 1);

        assert_eq!(vas.readonly[0].pubkey, readonly_new_account_id);
        assert_eq!(vas.readonly[1].pubkey, readonly_undelegated_id);
        assert_eq!(vas.readonly[2].pubkey, readonly_delegated_id);
        assert_eq!(vas.readonly[3].pubkey, readonly_inconsistent_id);
        assert_eq!(vas.writable[0].pubkey, writable_delegated_id);

        assert!(vas.readonly[0].account.is_none());
        assert!(vas.readonly[1].account.is_some());
        assert!(vas.readonly[2].account.is_some());
        assert!(vas.readonly[3].account.is_some());
        assert!(vas.writable[0].account.is_some());

        assert_eq!(vas.readonly[0].at_slot, 42);
        assert_eq!(vas.readonly[1].at_slot, 42);
        assert_eq!(vas.readonly[2].at_slot, 42);
        assert_eq!(vas.readonly[3].at_slot, 42);
        assert_eq!(vas.writable[0].at_slot, 42);
    }

    #[test]
    fn test_one_of_each_valid_type_while_permissive() {
        let readonly_new_account_id = Pubkey::new_unique();
        let readonly_undelegated_id = Pubkey::new_unique();
        let readonly_delegated_id = Pubkey::new_unique();
        let readonly_inconsistent_id = Pubkey::new_unique();

        let writable_new_account_id = Pubkey::new_unique();
        let writable_undelegated_id = Pubkey::new_unique();
        let writable_delegated_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_new_account_id,
            chain_snapshot: chain_snapshot_new_account(),
        };
        let meta2 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            chain_snapshot: chain_snapshot_undelegated(),
        };
        let meta3 = TransactionAccountMeta::Readonly {
            pubkey: readonly_delegated_id,
            chain_snapshot: chain_snapshot_delegated(),
        };
        let meta4 = TransactionAccountMeta::Readonly {
            pubkey: readonly_inconsistent_id,
            chain_snapshot: chain_snapshot_inconsistent(),
        };

        let meta5 = TransactionAccountMeta::Writable {
            pubkey: writable_new_account_id,
            chain_snapshot: chain_snapshot_new_account(),
            is_payer: false,
        };
        let meta6 = TransactionAccountMeta::Writable {
            pubkey: writable_undelegated_id,
            chain_snapshot: chain_snapshot_undelegated(),
            is_payer: false,
        };
        let meta7 = TransactionAccountMeta::Writable {
            pubkey: writable_delegated_id,
            chain_snapshot: chain_snapshot_delegated(),
            is_payer: false,
        };

        let vas: ValidatedAccounts = (
            TransactionAccountsMetas(vec![
                meta1, meta2, meta3, meta4, meta5, meta6, meta7,
            ]),
            &config_permissive(),
        )
            .try_into()
            .unwrap();

        assert_eq!(vas.readonly.len(), 4);
        assert_eq!(vas.writable.len(), 3);

        assert_eq!(vas.readonly[0].pubkey, readonly_new_account_id);
        assert_eq!(vas.readonly[1].pubkey, readonly_undelegated_id);
        assert_eq!(vas.readonly[2].pubkey, readonly_delegated_id);
        assert_eq!(vas.readonly[3].pubkey, readonly_inconsistent_id);

        assert_eq!(vas.writable[0].pubkey, writable_new_account_id);
        assert_eq!(vas.writable[1].pubkey, writable_undelegated_id);
        assert_eq!(vas.writable[2].pubkey, writable_delegated_id);

        assert!(vas.readonly[0].account.is_none());
        assert!(vas.readonly[1].account.is_some());
        assert!(vas.readonly[2].account.is_some());
        assert!(vas.readonly[3].account.is_some());

        assert!(vas.writable[0].account.is_none());
        assert!(vas.writable[1].account.is_some());
        assert!(vas.writable[2].account.is_some());

        assert_eq!(vas.readonly[0].at_slot, 42);
        assert_eq!(vas.readonly[1].at_slot, 42);
        assert_eq!(vas.readonly[2].at_slot, 42);
        assert_eq!(vas.readonly[3].at_slot, 42);

        assert_eq!(vas.writable[0].at_slot, 42);
        assert_eq!(vas.writable[1].at_slot, 42);
        assert_eq!(vas.writable[2].at_slot, 42);
    }
}
