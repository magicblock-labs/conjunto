use conjunto_core::delegation_inconsistency::DelegationInconsistency;
use conjunto_lockbox::{
    account_chain_snapshot::AccountChainSnapshot,
    account_chain_state::AccountChainState,
};
use conjunto_test_tools::accounts::{
    account_owned_by_delegation_program, account_with_data,
};
use conjunto_transwise::{
    transaction_accounts_snapshot::TransactionAccountsSnapshot,
    transaction_accounts_validator::{
        TransactionAccountsValidator, TransactionAccountsValidatorImpl,
    },
    AccountChainSnapshotShared, CommitFrequency, DelegationRecord,
};
use solana_sdk::{pubkey::Pubkey, system_program};

fn transaction_accounts_validator() -> TransactionAccountsValidatorImpl {
    TransactionAccountsValidatorImpl {}
}

fn chain_snapshot_wallet() -> AccountChainSnapshotShared {
    AccountChainSnapshot {
        pubkey: Pubkey::new_unique(),
        at_slot: 42,
        chain_state: AccountChainState::Wallet {
            lamports: 42,
            owner: system_program::ID,
        },
    }
    .into()
}
fn chain_snapshot_data() -> AccountChainSnapshotShared {
    AccountChainSnapshot {
        pubkey: Pubkey::new_unique(),
        at_slot: 42,
        chain_state: AccountChainState::Data {
            account: account_with_data(),
            delegation_inconsistency:
                DelegationInconsistency::AccountInvalidOwner,
        },
    }
    .into()
}
fn chain_snapshot_delegated() -> AccountChainSnapshotShared {
    AccountChainSnapshot {
        pubkey: Pubkey::new_unique(),
        at_slot: 42,
        chain_state: AccountChainState::Delegated {
            account: account_owned_by_delegation_program(),
            delegation_record: DelegationRecord {
                authority: Pubkey::new_unique(),
                owner: Pubkey::new_unique(),
                delegation_slot: 0,
                commit_frequency: CommitFrequency::Millis(1_000),
            },
        },
    }
    .into()
}

#[test]
fn test_two_readonly_data_and_two_writable_delegated_and_wallets() {
    let readonly_data1 = chain_snapshot_data();
    let readonly_data2 = chain_snapshot_data();
    let readonly_wallet = chain_snapshot_wallet();
    let writable_delegated1 = chain_snapshot_delegated();
    let writable_delegated2 = chain_snapshot_delegated();
    let writable_wallet = chain_snapshot_wallet();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_wallet.pubkey,
                readonly: vec![readonly_data1, readonly_data2, readonly_wallet],
                writable: vec![
                    writable_delegated1,
                    writable_delegated2,
                    writable_wallet,
                ],
            },
        );

    // This is a fairly typical case that should work (wallet and writables are in good condition)
    assert!(result.is_ok());
}

#[test]
fn test_empty_transaction_accounts() {
    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![],
                writable: vec![],
            },
        );

    // No data writables, so it's fine (solana will deny the transaction tho, because no payer)
    assert!(result.is_ok());
}

#[test]
fn test_only_one_readonly_data() {
    let readonly_data = chain_snapshot_data();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![readonly_data],
                writable: vec![],
            },
        );

    // No data writables, so it's fine (solana will deny the transaction tho, because no payer)
    assert!(result.is_ok());
}

#[test]
fn test_only_one_writable_delegated() {
    let writable_delegated = chain_snapshot_delegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![],
                writable: vec![writable_delegated],
            },
        );

    // No data writables, so it's fine (solana will deny the transaction tho, because no payer)
    assert!(result.is_ok());
}

#[test]
fn test_only_one_writable_wallet() {
    let writable_wallet = chain_snapshot_wallet();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![],
                writable: vec![writable_wallet],
            },
        );

    // No data writables, so it's fine (solana will deny the transaction tho, because no payer)
    assert!(result.is_ok());
}

#[test]
fn test_only_one_readable_data_as_payer() {
    let readable_data = chain_snapshot_data();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: readable_data.pubkey,
                readonly: vec![readable_data],
                writable: vec![],
            },
        );

    // No data writables, so it's fine (solana will deny the transaction tho, because invalid payer)
    assert!(result.is_ok());
}

#[test]
fn test_only_one_writable_data_as_payer_fail() {
    let writable_data = chain_snapshot_data();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_data.pubkey,
                readonly: vec![],
                writable: vec![writable_data],
            },
        );

    // This transaction's payer is data, that's not good, we should NOT allow this
    assert!(result.is_err());
}

#[test]
fn test_only_one_writable_delegated_as_payer() {
    let writable_delegated = chain_snapshot_delegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_delegated.pubkey,
                readonly: vec![],
                writable: vec![writable_delegated],
            },
        );

    // No data writables, so it's fine (solana will deny the transaction tho, because invalid payer)
    assert!(result.is_ok());
}

#[test]
fn test_only_one_writable_wallet_as_payer() {
    let writable_wallet = chain_snapshot_wallet();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_wallet.pubkey,
                readonly: vec![],
                writable: vec![writable_wallet],
            },
        );

    // Because there is a payer a wallet, this should work fine
    assert!(result.is_ok());
}

#[test]
fn test_one_readonly_data_and_writable_wallet_as_payer() {
    let readonly_data = chain_snapshot_data();
    let writable_wallet = chain_snapshot_wallet();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_wallet.pubkey,
                readonly: vec![readonly_data],
                writable: vec![writable_wallet],
            },
        );

    // This should work, this is a fairly common case
    assert!(result.is_ok());
}

#[test]
fn test_one_readonly_data_and_one_writable_delegated_and_wallet() {
    let readonly_data = chain_snapshot_data();
    let writable_delegated = chain_snapshot_delegated();
    let writable_wallet = chain_snapshot_wallet();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![readonly_data],
                writable: vec![writable_delegated, writable_wallet],
            },
        );

    // This should work, this is a fairly common case
    assert!(result.is_ok());
}

#[test]
fn test_one_readonly_data_and_one_writable_data_and_payer_fail() {
    let readonly_data = chain_snapshot_data();
    let writable_data = chain_snapshot_data();
    let writable_wallet = chain_snapshot_wallet();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![readonly_data],
                writable: vec![writable_data, writable_wallet],
            },
        );

    // Any writable data should fail
    assert!(result.is_err());
}

#[test]
fn test_one_readonly_data_and_one_writable_data_as_payer_fail() {
    let readonly_data = chain_snapshot_data();
    let writable_data = chain_snapshot_data();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_data.pubkey,
                readonly: vec![readonly_data],
                writable: vec![writable_data],
            },
        );

    // Payer is data and writable, which is wrong
    assert!(result.is_err());
}

#[test]
fn test_one_writable_data_and_writable_wallet_as_payer_fail() {
    let writable_data = chain_snapshot_data();
    let writable_wallet = chain_snapshot_wallet();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_wallet.pubkey,
                readonly: vec![],
                writable: vec![writable_data, writable_wallet],
            },
        );

    // Even if the payer is correct, we have a data account as writable so this should not work
    assert!(result.is_err());
}

#[test]
fn test_one_of_each_valid_type() {
    let readonly_data = chain_snapshot_data();
    let readonly_delegated = chain_snapshot_delegated();
    let readonly_wallet = chain_snapshot_wallet();

    let writable_delegated = chain_snapshot_delegated();
    let writable_wallet = chain_snapshot_wallet();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_wallet.pubkey,
                readonly: vec![
                    readonly_data,
                    readonly_delegated,
                    readonly_wallet,
                ],
                writable: vec![writable_delegated, writable_wallet],
            },
        );

    // This should work just right in strict mode
    assert!(result.is_ok());
}
