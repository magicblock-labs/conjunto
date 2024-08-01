use conjunto_lockbox::{
    AccountChainSnapshot, AccountChainState, CommitFrequency, DelegationRecord,
};
use conjunto_test_tools::accounts::{
    account_owned_by_delegation_program, account_owned_by_system_program,
};
use conjunto_transwise::transaction_accounts_snapshot::TransactionAccountsSnapshot;
use solana_sdk::pubkey::Pubkey;

use crate::errors::TranswiseResult;

fn transaction_accounts_validator() -> TransactionAccountsValidator {
    TransactionAccountsValidatorImpl {}
}

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
        pubkey: Pubkey::new_unique(),
        at_slot: 42,
        chain_state: AccountChainState::Delegated {
            account: account_owned_by_delegation_program(),
            delegated_id: Pubkey::new_unique(),
            delegation_pda: Pubkey::new_unique(),
            delegation_record: DelegationRecord {
                commit_frequency: CommitFrequency::Millis(1_000),
                owner: Pubkey::new_unique(),
            },
        },
    }
}
fn chain_snapshot_undelegated() -> AccountChainSnapshot {
    AccountChainSnapshot {
        pubkey: Pubkey::new_unique(),
        at_slot: 42,
        chain_state: AccountChainState::Undelegated {
            account: account_owned_by_system_program(),
        },
    }
}
fn chain_snapshot_new_account() -> AccountChainSnapshot {
    AccountChainSnapshot {
        pubkey: Pubkey::new_unique(),
        at_slot: 42,
        chain_state: AccountChainState::NewAccount,
    }
}
fn chain_snapshot_inconsistent() -> AccountChainSnapshot {
    AccountChainSnapshot {
        pubkey: Pubkey::new_unique(),
        at_slot: 42,
        chain_state: AccountChainState::Inconsistent {
            account: account_owned_by_system_program(),
            delegated_id: Pubkey::new_unique(),
            delegation_pda: Pubkey::new_unique(),
            delegation_inconsistencies: vec![],
        },
    }
}

#[test]
fn test_two_readonly_undelegated_and_two_writable_delegated_and_payer() {
    let undelegated1 = chain_snapshot_undelegated();
    let undelegated2 = chain_snapshot_undelegated();
    let delegated1 = chain_snapshot_delegated();
    let delegated2 = chain_snapshot_delegated();
    let undelegated_payer = chain_snapshot_undelegated();

    let transaction_accounts = TransactionAccountsSnapshot {
        readonly: vec![undelegated1, undelegated2],
        writable: vec![delegated1, delegated2, undelegated_payer],
        payer: writable_undelegated_payer.pubkey,
    };

    let result = transaction_accounts_validator()
        .validate_accounts(transaction_accounts, &config_strict());

    assert!(result.is_ok());
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
        TransactionAccountsSnapshot(vec![meta1, meta2]),
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
        TransactionAccountsSnapshot(vec![meta1, meta2]),
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
        TransactionAccountsSnapshot(vec![meta1, meta2]),
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
        TransactionAccountsSnapshot(vec![meta1, meta2]),
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
        TransactionAccountsSnapshot(vec![meta1, meta2]),
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
        TransactionAccountsSnapshot(vec![meta1, meta2, meta3]),
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
        TransactionAccountsSnapshot(vec![meta1, meta2, meta3, meta4, meta5]),
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
        TransactionAccountsSnapshot(vec![
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
