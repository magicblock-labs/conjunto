use conjunto_lockbox::AccountLockState;
pub use conjunto_lockbox::LockConfig;
use solana_sdk::pubkey::Pubkey;

use crate::{
    errors::TranswiseError,
    trans_account_meta::{TransAccountMeta, TransAccountMetas},
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

    // The logic here is that this is None if the account doesn't exist
    // If the account exists, this represents wether or not the account is executable
    pub is_program: Option<bool>,
}

impl ValidatedReadonlyAccount {
    pub fn try_from(
        meta: &TransAccountMeta,
    ) -> Option<ValidatedReadonlyAccount> {
        match meta {
            TransAccountMeta::Readonly { pubkey, lockstate } => {
                Some(ValidatedReadonlyAccount {
                    pubkey: *pubkey,
                    is_program: match lockstate {
                        AccountLockState::NewAccount => None,
                        AccountLockState::Undelegated { is_program } => {
                            Some(*is_program)
                        }
                        AccountLockState::Delegated { .. } => Some(false),
                        AccountLockState::Inconsistent { .. } => Some(false),
                    },
                })
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct ValidatedWritableAccount {
    pub pubkey: Pubkey,

    /// The config for delegated accounts.
    /// This is `None` for undelegated or new writable accounts.
    pub lock_config: Option<LockConfig>,

    /// Indicates if this account was a payer in the transaction from which
    /// it was extracted.
    pub is_payer: bool,

    /// Indicates that this account was not found on chain but was included
    /// since we allow new accounts to be created.
    pub is_new: bool,
}

impl ValidatedWritableAccount {
    pub fn try_from(
        meta: &TransAccountMeta,
    ) -> Option<ValidatedWritableAccount> {
        match meta {
            TransAccountMeta::Writable {
                pubkey,
                lockstate,
                is_payer,
            } => Some(ValidatedWritableAccount {
                pubkey: *pubkey,
                lock_config: match lockstate {
                    AccountLockState::Delegated { config, .. } => {
                        Some(config.clone())
                    }
                    _ => None,
                },
                is_payer: *is_payer,
                is_new: lockstate.is_new(),
            }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct ValidatedAccounts {
    pub readonly: Vec<ValidatedReadonlyAccount>,
    pub writable: Vec<ValidatedWritableAccount>,
}

impl TryFrom<(&TransAccountMetas, &ValidateAccountsConfig)>
    for ValidatedAccounts
{
    type Error = TranswiseError;

    fn try_from(
        (meta, config): (&TransAccountMetas, &ValidateAccountsConfig),
    ) -> Result<Self, Self::Error> {
        // The flags require_delegation and allow_new_accounts cannot be true at the same time
        // This is because a new account cannot have been delegated (so it creates all sort of edge cases)
        // TODO(vbrunet) - make sure in the validator's config this throws a warning in this case
        assert!(!config.require_delegation || !config.allow_new_accounts);

        // First, a quick guard against accounts that are inconsistently delegated
        let writable_inconsistent_pubkeys =
            meta.writable_inconsistent_pubkeys();
        let has_writable_inconsistent =
            !writable_inconsistent_pubkeys.is_empty();
        if has_writable_inconsistent {
            return Err(TranswiseError::WritablesIncludeInconsistentAccounts {
                writable_inconsistent_pubkeys,
            });
        }

        // If we require delegation:
        // We need make sure that all writables are delegated
        // Except we don't worry about the payer, because it doesn't contain data, it just need to sign
        if config.require_delegation {
            let writable_undelegated_non_payer_pubkeys =
                meta.writable_undelegated_non_payer_pubkeys();
            let has_writable_undelegated_non_payer =
                !writable_undelegated_non_payer_pubkeys.is_empty();
            if has_writable_undelegated_non_payer {
                let writable_delegated_pubkeys =
                    meta.writable_delegated_pubkeys();
                return Err(TranswiseError::NotAllWritablesDelegated {
                    writable_delegated_pubkeys,
                    writable_undelegated_non_payer_pubkeys,
                });
            }
        }

        // NOTE: when we don't require delegation then we still query the account states to
        // get the lockstate of each delegated. This causes some unnecessary overhead which we
        // could avoid if we make the lockbox aware of this, i.e. by adding an LockstateUnknown
        // variant and returning that instead of checking it.
        // However this is only the case when developing locally and thus we may not optimize for it.

        // Then, if we are not allowed to create new accounts, we need to guard against them
        if !config.allow_new_accounts {
            let writable_new_pubkeys = meta.writable_new_pubkeys();
            let has_writable_new = !writable_new_pubkeys.is_empty();
            if has_writable_new {
                return Err(TranswiseError::WritablesIncludeNewAccounts {
                    writable_new_pubkeys,
                });
            }
        }

        // Generate the validated account structs
        let validated_readonly_accounts =
            meta.iter().flat_map(ValidatedReadonlyAccount::try_from);
        let validated_writable_accounts =
            meta.iter().flat_map(ValidatedWritableAccount::try_from);

        // Done
        Ok(ValidatedAccounts {
            readonly: validated_readonly_accounts.collect(),
            writable: validated_writable_accounts.collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use conjunto_core::CommitFrequency;
    use conjunto_lockbox::{AccountLockState, LockConfig};

    use super::*;
    use crate::{
        errors::TranswiseResult, trans_account_meta::TransAccountMeta,
    };

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

    fn lockstate_delegated() -> AccountLockState {
        AccountLockState::Delegated {
            delegated_id: Pubkey::new_unique(),
            delegation_pda: Pubkey::new_unique(),
            config: LockConfig {
                commit_frequency: CommitFrequency::Millis(1_000),
                owner: Pubkey::new_unique(),
            },
        }
    }

    fn lockstate_undelegated() -> AccountLockState {
        AccountLockState::Undelegated { is_program: false }
    }

    fn lockstate_new_account() -> AccountLockState {
        AccountLockState::NewAccount
    }

    fn lockstate_inconsistent() -> AccountLockState {
        AccountLockState::Inconsistent {
            delegated_id: Pubkey::new_unique(),
            delegation_pda: Pubkey::new_unique(),
            inconsistencies: vec![],
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

        let meta1 = TransAccountMeta::Readonly {
            pubkey: readonly_undelegated_id1,
            lockstate: lockstate_undelegated(),
        };
        let meta2 = TransAccountMeta::Readonly {
            pubkey: readonly_undelegated_id2,
            lockstate: lockstate_undelegated(),
        };
        let meta3 = TransAccountMeta::Writable {
            pubkey: writable_delegated_id1,
            lockstate: lockstate_delegated(),
            is_payer: false,
        };
        let meta4 = TransAccountMeta::Writable {
            pubkey: writable_delegated_id2,
            lockstate: lockstate_delegated(),
            is_payer: false,
        };
        let meta5 = TransAccountMeta::Writable {
            pubkey: writable_undelegated_payer_id,
            lockstate: lockstate_undelegated(),
            is_payer: true,
        };

        let vas: ValidatedAccounts = (
            &TransAccountMetas(vec![meta1, meta2, meta3, meta4, meta5]),
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

        let meta1 = TransAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            lockstate: lockstate_undelegated(),
        };
        let meta2 = TransAccountMeta::Writable {
            pubkey: writable_undelegated_id,
            lockstate: lockstate_undelegated(),
            is_payer: false,
        };

        let res: TranswiseResult<ValidatedAccounts> =
            (&TransAccountMetas(vec![meta1, meta2]), &config_strict())
                .try_into();

        assert!(res.is_err());
    }

    #[test]
    fn test_one_readonly_undelegated_and_payer() {
        let readonly_undelegated_id = Pubkey::new_unique();
        let writable_undelegated_payer_id = Pubkey::new_unique();

        let meta1 = TransAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            lockstate: lockstate_undelegated(),
        };
        let meta2 = TransAccountMeta::Writable {
            pubkey: writable_undelegated_payer_id,
            lockstate: lockstate_undelegated(),
            is_payer: true,
        };

        let vas: ValidatedAccounts =
            (&TransAccountMetas(vec![meta1, meta2]), &config_strict())
                .try_into()
                .unwrap();

        assert_eq!(readonly_pubkeys(&vas), vec![readonly_undelegated_id]);
        assert_eq!(writable_pubkeys(&vas), vec![writable_undelegated_payer_id]);
    }

    #[test]
    fn test_one_readonly_undelegated_and_one_writable_inconsistent() {
        let readonly_undelegated_id = Pubkey::new_unique();
        let writable_inconsistent_id = Pubkey::new_unique();

        let meta1 = TransAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            lockstate: lockstate_undelegated(),
        };
        let meta2 = TransAccountMeta::Writable {
            pubkey: writable_inconsistent_id,
            lockstate: lockstate_inconsistent(),
            is_payer: false,
        };

        let res: TranswiseResult<ValidatedAccounts> =
            (&TransAccountMetas(vec![meta1, meta2]), &config_strict())
                .try_into();

        assert!(res.is_err());
    }

    #[test]
    fn test_one_readonly_undelegated_and_one_writable_new_account() {
        let readonly_undelegated_id = Pubkey::new_unique();
        let writable_new_account_id = Pubkey::new_unique();

        let meta1 = TransAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            lockstate: lockstate_undelegated(),
        };
        let meta2 = TransAccountMeta::Writable {
            pubkey: writable_new_account_id,
            lockstate: lockstate_new_account(),
            is_payer: false,
        };

        let res: TranswiseResult<ValidatedAccounts> =
            (&TransAccountMetas(vec![meta1, meta2]), &config_strict())
                .try_into();

        assert!(res.is_err());
    }

    #[test]
    fn test_one_readonly_undelegated_and_one_writable_new_account_and_one_writable_undelegated_while_permissive(
    ) {
        let readonly_undelegated_id1 = Pubkey::new_unique();
        let writable_new_account_id = Pubkey::new_unique();
        let writable_undelegated_id = Pubkey::new_unique();

        let meta1 = TransAccountMeta::Readonly {
            pubkey: readonly_undelegated_id1,
            lockstate: lockstate_undelegated(),
        };
        let meta2 = TransAccountMeta::Writable {
            pubkey: writable_new_account_id,
            lockstate: lockstate_new_account(),
            is_payer: false,
        };
        let meta3 = TransAccountMeta::Writable {
            pubkey: writable_undelegated_id,
            lockstate: lockstate_delegated(),
            is_payer: false,
        };

        let vas: ValidatedAccounts = (
            &TransAccountMetas(vec![meta1, meta2, meta3]),
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
}
