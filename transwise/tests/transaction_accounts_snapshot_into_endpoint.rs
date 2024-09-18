use std::vec;

use conjunto_lockbox::account_chain_snapshot_provider::AccountChainSnapshotProvider;
use conjunto_test_tools::{
    account_provider_stub::AccountProviderStub,
    accounts::{
        account_owned_by_delegation_program, account_owned_by_system_program,
        delegated_account_ids,
    },
    delegation_record_parser_stub::DelegationRecordParserStub,
};
use conjunto_transwise::{
    endpoint::{Endpoint, UnroutableReason},
    transaction_accounts_holder::TransactionAccountsHolder,
    transaction_accounts_snapshot::TransactionAccountsSnapshot,
    CommitFrequency, DelegationRecord,
};
use solana_sdk::{account::Account, clock::Slot, pubkey::Pubkey};

const EXPECTED_SLOT: Slot = 42;

fn setup_chain_snapshot_provider(
    accounts: Vec<(Pubkey, Account)>,
    delegation_record: Option<DelegationRecord>,
) -> AccountChainSnapshotProvider<AccountProviderStub, DelegationRecordParserStub>
{
    let mut account_provider = AccountProviderStub::default();
    account_provider.at_slot = EXPECTED_SLOT;
    for (pubkey, account) in accounts {
        account_provider.add(pubkey, account);
    }
    let delegation_record_parser =
        DelegationRecordParserStub::new(delegation_record);
    AccountChainSnapshotProvider::new(
        account_provider,
        delegation_record_parser,
    )
}

fn dummy_delegation_record_with_owner(owner: Pubkey) -> DelegationRecord {
    DelegationRecord {
        authority: Pubkey::new_unique(),
        owner,
        delegation_slot: 0,
        commit_frequency: CommitFrequency::Millis(1_000),
    }
}

fn dummy_pda() -> Pubkey {
    loop {
        let pubkey = Pubkey::new_unique();
        if !pubkey.is_on_curve() {
            return pubkey;
        }
    }
}

fn dummy_wallet() -> Pubkey {
    loop {
        let pubkey = Pubkey::new_unique();
        if pubkey.is_on_curve() {
            return pubkey;
        }
    }
}

#[tokio::test]
async fn test_one_undelegated_readonly_and_one_delegated_writable_and_payer() {
    let (writable_delegated, delegation_record) = delegated_account_ids();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_delegated, account_owned_by_delegation_program()),
            (delegation_record, account_owned_by_delegation_program()),
        ],
        Some(dummy_delegation_record_with_owner(Pubkey::new_unique())),
    );

    let readonly_undelegated = dummy_pda();
    let writable_wallet = dummy_wallet();

    let acc_holder = TransactionAccountsHolder {
        readonly: vec![readonly_undelegated],
        writable: vec![writable_delegated, writable_wallet],
        payer: writable_wallet,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 1);
    assert_eq!(acc_snapshot.writable.len(), 2);

    assert_eq!(acc_snapshot.readonly[0].pubkey, readonly_undelegated);
    assert_eq!(acc_snapshot.writable[0].pubkey, writable_delegated);
    assert_eq!(acc_snapshot.writable[1].pubkey, writable_wallet);

    assert!(acc_snapshot.readonly[0].chain_state.is_undelegated());
    assert!(acc_snapshot.writable[0].chain_state.is_delegated());
    assert!(acc_snapshot.writable[1].chain_state.is_wallet());

    assert_eq!(acc_snapshot.payer, writable_wallet);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    assert_eq!(
        endpoint,
        Endpoint::Ephemeral {
            transaction_accounts_snapshot: acc_snapshot,
        }
    );
}

#[tokio::test]
async fn test_one_writable_delegated_and_one_writable_undelegated() {
    let (writable_delegated, delegation_record) = delegated_account_ids();
    let writable_undelegated = dummy_pda();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_delegated, account_owned_by_delegation_program()),
            (delegation_record, account_owned_by_delegation_program()),
            (writable_undelegated, account_owned_by_system_program()),
        ],
        Some(dummy_delegation_record_with_owner(Pubkey::new_unique())),
    );
    let writable_wallet = dummy_wallet();

    let acc_holder = TransactionAccountsHolder {
        readonly: vec![],
        writable: vec![
            writable_delegated,
            writable_undelegated,
            writable_wallet,
        ],
        payer: writable_wallet,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 0);
    assert_eq!(acc_snapshot.writable.len(), 3);

    assert_eq!(acc_snapshot.writable[0].pubkey, writable_delegated);
    assert_eq!(acc_snapshot.writable[1].pubkey, writable_undelegated);
    assert_eq!(acc_snapshot.writable[2].pubkey, writable_wallet);

    assert!(acc_snapshot.writable[0].chain_state.is_delegated());
    assert!(acc_snapshot.writable[1].chain_state.is_undelegated());
    assert!(acc_snapshot.writable[2].chain_state.is_wallet());

    assert_eq!(acc_snapshot.payer, writable_wallet);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    assert_eq!(
        endpoint,
        Endpoint::Unroutable {
            transaction_accounts_snapshot: acc_snapshot,
            reason:
                UnroutableReason::ContainsBothDelegatedAndUndelegatedWritable {
                    writable_undelegated_pubkeys: vec![writable_undelegated],
                    writable_delegated_pubkeys: vec![writable_delegated],
                }
        }
    );
}

#[tokio::test]
async fn test_one_writable_inconsistent_with_missing_delegation_account() {
    let (writable_undelegated, _) = delegated_account_ids();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_undelegated, account_owned_by_delegation_program()),
            // Missing delegation account
        ],
        Some(dummy_delegation_record_with_owner(Pubkey::new_unique())),
    );
    let writable_wallet = dummy_wallet();

    let acc_holder = TransactionAccountsHolder {
        readonly: vec![],
        writable: vec![writable_undelegated, writable_wallet],
        payer: writable_wallet,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 0);
    assert_eq!(acc_snapshot.writable.len(), 2);

    assert_eq!(acc_snapshot.writable[0].pubkey, writable_undelegated);
    assert_eq!(acc_snapshot.writable[1].pubkey, writable_wallet);

    assert!(acc_snapshot.writable[0].chain_state.is_undelegated());
    assert!(acc_snapshot.writable[1].chain_state.is_wallet());

    assert_eq!(acc_snapshot.payer, writable_wallet);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    assert_eq!(
        endpoint,
        Endpoint::Chain {
            transaction_accounts_snapshot: acc_snapshot,
        }
    );
}

#[tokio::test]
async fn test_one_writable_inconsistent_with_invalid_delegation_record() {
    let (writable_undelegated, delegation_record) = delegated_account_ids();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_undelegated, account_owned_by_delegation_program()),
            (delegation_record, account_owned_by_delegation_program()),
        ],
        None, // invalid delegation record for delegated account
    );
    let writable_wallet = dummy_wallet();

    let acc_holder = TransactionAccountsHolder {
        readonly: vec![],
        writable: vec![writable_undelegated, writable_wallet],
        payer: writable_wallet,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 0);
    assert_eq!(acc_snapshot.writable.len(), 2);

    assert_eq!(acc_snapshot.writable[0].pubkey, writable_undelegated);
    assert_eq!(acc_snapshot.writable[1].pubkey, writable_wallet);

    assert!(acc_snapshot.writable[0].chain_state.is_undelegated());
    assert!(acc_snapshot.writable[1].chain_state.is_wallet());

    assert_eq!(acc_snapshot.payer, writable_wallet);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    assert_eq!(
        endpoint,
        Endpoint::Chain {
            transaction_accounts_snapshot: acc_snapshot,
        }
    );
}

#[tokio::test]
async fn test_one_writable_undelegated_with_writable_wallet() {
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![],
        Some(dummy_delegation_record_with_owner(Pubkey::new_unique())),
    );

    let writable_undelegated = dummy_pda();
    let writable_wallet = dummy_wallet();

    let acc_holder = TransactionAccountsHolder {
        readonly: vec![],
        writable: vec![writable_undelegated, writable_wallet],
        payer: writable_wallet,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 0);
    assert_eq!(acc_snapshot.writable.len(), 2);

    assert_eq!(acc_snapshot.writable[0].pubkey, writable_undelegated);
    assert_eq!(acc_snapshot.writable[1].pubkey, writable_wallet);

    assert!(acc_snapshot.writable[0].chain_state.is_undelegated());
    assert!(acc_snapshot.writable[1].chain_state.is_wallet());

    assert_eq!(acc_snapshot.payer, writable_wallet);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    assert_eq!(
        endpoint,
        Endpoint::Chain {
            transaction_accounts_snapshot: acc_snapshot
        }
    );
}

#[tokio::test]
async fn test_one_writable_undelegated_as_payer() {
    // NOTE: it is very rare to encounter a transaction which would only have
    //       write to one account (same as payer) and we don't expect a
    //       transaction like this to make sense inside the ephemeral validator.
    //       That is the main reason we send it to chain
    let writable_undelegated = dummy_pda();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![(writable_undelegated, account_owned_by_system_program())],
        Some(dummy_delegation_record_with_owner(Pubkey::new_unique())),
    );

    let acc_holder = TransactionAccountsHolder {
        readonly: vec![],
        writable: vec![writable_undelegated],
        payer: writable_undelegated,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 0);
    assert_eq!(acc_snapshot.writable.len(), 1);

    assert_eq!(acc_snapshot.writable[0].pubkey, writable_undelegated);
    assert!(acc_snapshot.writable[0].chain_state.is_undelegated());

    assert_eq!(acc_snapshot.payer, writable_undelegated);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    assert_eq!(
        endpoint,
        Endpoint::Chain {
            transaction_accounts_snapshot: acc_snapshot
        }
    );
}

#[tokio::test]
async fn test_one_writable_undelegated_as_payer_and_one_writable_delegated() {
    let (writable_delegated, delegation_record) = delegated_account_ids();
    let writable_undelegated = dummy_pda();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_delegated, account_owned_by_delegation_program()),
            (delegation_record, account_owned_by_delegation_program()),
            (writable_undelegated, account_owned_by_system_program()),
        ],
        Some(dummy_delegation_record_with_owner(writable_delegated)),
    );

    let acc_holder = TransactionAccountsHolder {
        readonly: vec![],
        writable: vec![writable_undelegated, writable_delegated],
        payer: writable_undelegated,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 0);
    assert_eq!(acc_snapshot.writable.len(), 2);

    assert_eq!(acc_snapshot.writable[0].pubkey, writable_undelegated);
    assert_eq!(acc_snapshot.writable[1].pubkey, writable_delegated);

    assert!(acc_snapshot.writable[0].chain_state.is_undelegated());
    assert!(acc_snapshot.writable[1].chain_state.is_delegated());

    assert_eq!(acc_snapshot.payer, writable_undelegated);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    assert_eq!(
        endpoint,
        Endpoint::Unroutable {
            transaction_accounts_snapshot: acc_snapshot,
            reason:
                UnroutableReason::ContainsBothDelegatedAndUndelegatedWritable {
                    writable_undelegated_pubkeys: vec![writable_undelegated],
                    writable_delegated_pubkeys: vec![writable_delegated],
                }
        }
    );
}

#[tokio::test]
async fn test_two_readonly_undelegateds() {
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![],
        Some(dummy_delegation_record_with_owner(Pubkey::new_unique())),
    );

    let readonly1_undelegated = dummy_pda();
    let readonly2_undelegated = dummy_pda();
    let writable_payer = dummy_wallet();

    let acc_holder = TransactionAccountsHolder {
        readonly: vec![readonly1_undelegated, readonly2_undelegated],
        writable: vec![writable_payer],
        payer: writable_payer,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 2);
    assert_eq!(acc_snapshot.writable.len(), 1);

    assert_eq!(acc_snapshot.readonly[0].pubkey, readonly1_undelegated);
    assert_eq!(acc_snapshot.readonly[1].pubkey, readonly2_undelegated);
    assert_eq!(acc_snapshot.writable[0].pubkey, writable_payer);

    assert!(acc_snapshot.readonly[0].chain_state.is_undelegated());
    assert!(acc_snapshot.readonly[1].chain_state.is_undelegated());
    assert!(acc_snapshot.writable[0].chain_state.is_wallet());

    assert_eq!(acc_snapshot.payer, writable_payer);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    assert_eq!(
        endpoint,
        Endpoint::Chain {
            transaction_accounts_snapshot: acc_snapshot
        }
    );
}

#[tokio::test]
async fn test_two_readonly_undelegated_and_one_writable_undelegated() {
    let writable_undelegated = dummy_pda();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![(writable_undelegated, account_owned_by_system_program())],
        Some(dummy_delegation_record_with_owner(Pubkey::new_unique())),
    );
    let readonly1_undelegated = dummy_pda();
    let readonly2_undelegated = dummy_pda();
    let writable_wallet = dummy_wallet();

    let acc_holder = TransactionAccountsHolder {
        readonly: vec![readonly1_undelegated, readonly2_undelegated],
        writable: vec![writable_undelegated, writable_wallet],
        payer: writable_wallet,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 2);
    assert_eq!(acc_snapshot.writable.len(), 2);

    assert_eq!(acc_snapshot.readonly[0].pubkey, readonly1_undelegated);
    assert_eq!(acc_snapshot.readonly[1].pubkey, readonly2_undelegated);
    assert_eq!(acc_snapshot.writable[0].pubkey, writable_undelegated);
    assert_eq!(acc_snapshot.writable[1].pubkey, writable_wallet);

    assert!(acc_snapshot.readonly[0].chain_state.is_undelegated());
    assert!(acc_snapshot.readonly[1].chain_state.is_undelegated());
    assert!(acc_snapshot.writable[0].chain_state.is_undelegated());
    assert!(acc_snapshot.writable[1].chain_state.is_wallet());

    assert_eq!(acc_snapshot.payer, writable_wallet);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    assert_eq!(
        endpoint,
        Endpoint::Chain {
            transaction_accounts_snapshot: acc_snapshot
        }
    );
}
