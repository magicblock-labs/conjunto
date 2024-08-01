use std::vec;

use conjunto_lockbox::{
    account_chain_snapshot::AccountChainSnapshotProvider,
    delegation_record::DelegationRecord,
};
use conjunto_test_tools::{
    account_provider_stub::AccountProviderStub,
    accounts::{
        account_owned_by_delegation_program, account_owned_by_system_program,
        delegated_account_ids, program_account,
    },
    delegation_record_parser_stub::DelegationRecordParserStub,
    transaction_accounts_holder_stub::TransactionAccountsHolderStub,
};
use conjunto_transwise::{
    endpoint::{Endpoint, UnroutableReason},
    transaction_accounts_snapshot::TransactionAccountsSnapshot,
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
    AccountChainSnapshotProvider::with_provider_and_parser(
        account_provider,
        delegation_record_parser,
    )
}

#[tokio::test]
async fn test_one_new_account_readonly_and_one_delegated_writable() {
    let (writable_delegated_id, delegation_pda) = delegated_account_ids();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_delegation_program()),
        ],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );
    let readonly_new_account_id = Pubkey::new_from_array([4u8; 32]);
    let payer_id = Pubkey::new_unique();

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![readonly_new_account_id],
        writable: vec![writable_delegated_id],
        payer: payer_id,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 1);
    assert_eq!(acc_snapshot.writable.len(), 1);

    assert_eq!(acc_snapshot.readonly[0].pubkey, readonly_new_account_id);
    assert_eq!(acc_snapshot.writable[0].pubkey, writable_delegated_id);

    assert!(acc_snapshot.readonly[0].chain_state.is_new());
    assert!(acc_snapshot.writable[0].chain_state.is_delegated());

    assert_eq!(acc_snapshot.payer, payer_id);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    eprintln!("{:#?}", endpoint);
    assert_eq!(
        endpoint,
        Endpoint::Ephemeral {
            transaction_accounts_snapshot: acc_snapshot,
            writable_delegated_pubkeys: vec![writable_delegated_id]
        }
    );
}

#[tokio::test]
async fn test_one_writable_delegated_and_one_writable_undelegated() {
    let (writable_delegated_id, delegation_pda) = delegated_account_ids();
    let writable_undelegated_id = Pubkey::new_from_array([4u8; 32]);
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_delegation_program()),
            (writable_undelegated_id, account_owned_by_system_program()),
        ],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );
    let payer_id = Pubkey::new_unique();

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![],
        writable: vec![writable_delegated_id, writable_undelegated_id],
        payer: payer_id,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 0);
    assert_eq!(acc_snapshot.writable.len(), 2);

    assert_eq!(acc_snapshot.writable[0].pubkey, writable_delegated_id);
    assert_eq!(acc_snapshot.writable[1].pubkey, writable_undelegated_id);

    assert!(acc_snapshot.writable[0].chain_state.is_delegated());
    assert!(acc_snapshot.writable[1].chain_state.is_undelegated());

    assert_eq!(acc_snapshot.payer, payer_id);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    eprintln!("{:#?}", endpoint);
    assert_eq!(endpoint, Endpoint::Unroutable {
        transaction_accounts_snapshot: acc_snapshot,
        reason: UnroutableReason::ContainsWritableDelegatedAndWritableUndelegated {
            writable_delegated_pubkeys: vec![writable_delegated_id],
            writable_undelegated_non_payer_pubkeys: vec![writable_undelegated_id]
        }
    });
}

#[tokio::test]
async fn test_one_writable_inconsistent_with_missing_delegation_account() {
    let (writable_inconsistent_id, _) = delegated_account_ids();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (
                writable_inconsistent_id,
                account_owned_by_delegation_program(),
            ),
            // Missing delegation account
        ],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );
    let payer_id = Pubkey::new_unique();

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![],
        writable: vec![writable_inconsistent_id],
        payer: payer_id,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 0);
    assert_eq!(acc_snapshot.writable.len(), 1);

    assert_eq!(acc_snapshot.writable[0].pubkey, writable_inconsistent_id);
    assert!(acc_snapshot.writable[0].chain_state.is_inconsistent());

    assert_eq!(acc_snapshot.payer, payer_id);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    eprintln!("{:#?}", endpoint);
    assert_eq!(
        endpoint,
        Endpoint::Unroutable {
            transaction_accounts_snapshot: acc_snapshot,
            reason: UnroutableReason::WritablesIncludeInconsistentAccounts {
                writable_inconsistent_pubkeys: vec![writable_inconsistent_id]
            }
        }
    );
}

#[tokio::test]
async fn test_one_writable_inconsistent_with_invalid_delegation_record() {
    let (writable_inconsistent_id, delegation_pda) = delegated_account_ids();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (
                writable_inconsistent_id,
                account_owned_by_delegation_program(),
            ),
            (delegation_pda, account_owned_by_delegation_program()),
        ],
        None, // invalid delegation record for delegated account
    );
    let payer_id = Pubkey::new_unique();

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![],
        writable: vec![writable_inconsistent_id],
        payer: payer_id,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 0);
    assert_eq!(acc_snapshot.writable.len(), 1);

    assert_eq!(acc_snapshot.writable[0].pubkey, writable_inconsistent_id);
    assert!(acc_snapshot.writable[0].chain_state.is_inconsistent());

    assert_eq!(acc_snapshot.payer, payer_id);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    eprintln!("{:#?}", endpoint);
    assert_eq!(
        endpoint,
        Endpoint::Unroutable {
            transaction_accounts_snapshot: acc_snapshot,
            reason: UnroutableReason::WritablesIncludeInconsistentAccounts {
                writable_inconsistent_pubkeys: vec![writable_inconsistent_id]
            }
        }
    );
}

#[tokio::test]
async fn test_one_writable_delegated_and_one_writable_new_account() {
    let (writable_delegated_id, delegation_pda) = delegated_account_ids();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_delegation_program()),
        ],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );
    let writable_new_account_id = Pubkey::new_from_array([4u8; 32]);
    let payer_id = Pubkey::new_unique();

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![],
        writable: vec![writable_delegated_id, writable_new_account_id],
        payer: payer_id,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 0);
    assert_eq!(acc_snapshot.writable.len(), 2);

    assert_eq!(acc_snapshot.writable[0].pubkey, writable_delegated_id);
    assert_eq!(acc_snapshot.writable[1].pubkey, writable_new_account_id);

    assert!(acc_snapshot.writable[0].chain_state.is_delegated());
    assert!(acc_snapshot.writable[1].chain_state.is_new());

    assert_eq!(acc_snapshot.payer, payer_id);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    eprintln!("{:#?}", endpoint);
    assert_eq!(
        endpoint,
        Endpoint::Unroutable {
            transaction_accounts_snapshot: acc_snapshot,
            reason: UnroutableReason::ContainsWritableDelegatedAndWritableUndelegated {
                writable_delegated_pubkeys: vec![writable_delegated_id],
                writable_undelegated_non_payer_pubkeys: vec![writable_new_account_id],
            }
        }
    );
}

#[tokio::test]
async fn test_one_writable_new_account() {
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );

    let writable_new_account_id = Pubkey::new_from_array([4u8; 32]);
    let payer_id = Pubkey::new_unique();

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![],
        writable: vec![writable_new_account_id],
        payer: payer_id,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 0);
    assert_eq!(acc_snapshot.writable.len(), 1);

    assert_eq!(acc_snapshot.writable[0].pubkey, writable_new_account_id);
    assert!(acc_snapshot.writable[0].chain_state.is_new());

    assert_eq!(acc_snapshot.payer, payer_id);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    eprintln!("{:#?}", endpoint);
    assert_eq!(
        endpoint,
        Endpoint::Chain {
            transaction_accounts_snapshot: acc_snapshot
        }
    );
}

#[tokio::test]
async fn test_one_writable_undelegated_that_is_payer() {
    // NOTE: it is very rare to encounter a transaction which would only have
    //       write to one account (same as payer) and we don't expect a
    //       transaction like this to make sense inside the ephemeral validator.
    //       That is the main reason we send it to chain
    let writable_undelegated_id = Pubkey::new_from_array([4u8; 32]);
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![(writable_undelegated_id, account_owned_by_system_program())],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![],
        writable: vec![writable_undelegated_id],
        payer: writable_undelegated_id,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 0);
    assert_eq!(acc_snapshot.writable.len(), 1);

    assert_eq!(acc_snapshot.writable[0].pubkey, writable_undelegated_id);
    assert!(acc_snapshot.writable[0].chain_state.is_undelegated());

    assert_eq!(acc_snapshot.payer, writable_undelegated_id);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    eprintln!("{:#?}", endpoint);
    assert_eq!(
        endpoint,
        Endpoint::Chain {
            transaction_accounts_snapshot: acc_snapshot
        }
    );
}

#[tokio::test]
async fn test_one_writable_undelegated_that_is_payer_and_one_writable_delegated(
) {
    let (writable_delegated_id, delegation_pda) = delegated_account_ids();
    let writable_undelegated_id = Pubkey::new_from_array([4u8; 32]);
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_delegation_program()),
            (writable_undelegated_id, account_owned_by_system_program()),
        ],
        Some(DelegationRecord::default_with_owner(writable_delegated_id)),
    );

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![],
        writable: vec![writable_undelegated_id, writable_delegated_id],
        payer: writable_undelegated_id,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 0);
    assert_eq!(acc_snapshot.writable.len(), 2);

    assert_eq!(acc_snapshot.writable[0].pubkey, writable_undelegated_id);
    assert_eq!(acc_snapshot.writable[1].pubkey, writable_delegated_id);

    assert!(acc_snapshot.readonly[0].chain_state.is_undelegated());
    assert!(acc_snapshot.writable[1].chain_state.is_delegated());

    assert_eq!(acc_snapshot.payer, writable_undelegated_id);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    eprintln!("{:#?}", endpoint);
    assert_eq!(
        endpoint,
        Endpoint::Ephemeral {
            transaction_accounts_snapshot: acc_snapshot,
            writable_delegated_pubkeys: vec![writable_delegated_id]
        }
    )
}

#[tokio::test]
async fn test_account_meta_one_writable_undelegated_that_is_payer_and_writable_undelegated(
) {
    let writable_undelegated_id = Pubkey::new_from_array([3u8; 32]);
    let writable_undelegated_payer_id = Pubkey::new_from_array([4u8; 32]);
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_undelegated_id, account_owned_by_system_program()),
            (
                writable_undelegated_payer_id,
                account_owned_by_system_program(),
            ),
        ],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![],
        writable: vec![writable_undelegated_payer_id, writable_undelegated_id],
        payer: writable_undelegated_payer_id,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 0);
    assert_eq!(acc_snapshot.writable.len(), 2);

    assert_eq!(
        acc_snapshot.writable[0].pubkey,
        writable_undelegated_payer_id
    );
    assert_eq!(acc_snapshot.writable[1].pubkey, writable_undelegated_id);

    assert!(acc_snapshot.writable[0].chain_state.is_undelegated());
    assert!(acc_snapshot.writable[1].chain_state.is_undelegated());

    assert_eq!(acc_snapshot.payer, writable_undelegated_payer_id);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    eprintln!("{:#?}", endpoint);
    assert_eq!(
        endpoint,
        Endpoint::Chain {
            transaction_accounts_snapshot: acc_snapshot
        }
    );
}

#[tokio::test]
async fn test_one_writable_undelegated_and_two_readonly_new_accounts() {
    let writable_undelegated_id = Pubkey::new_from_array([4u8; 32]);
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![(writable_undelegated_id, account_owned_by_system_program())],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );
    let readonly1_new_account_id = Pubkey::new_from_array([4u8; 32]);
    let readonly2_new_account_id = Pubkey::new_from_array([5u8; 32]);
    let payer_id = Pubkey::new_unique();

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![readonly1_new_account_id, readonly2_new_account_id],
        writable: vec![writable_undelegated_id],
        payer: payer_id,
    };

    let acc_snapshot = TransactionAccountsSnapshot::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    assert_eq!(acc_snapshot.readonly.len(), 2);
    assert_eq!(acc_snapshot.writable.len(), 1);

    assert_eq!(acc_snapshot.readonly[0].pubkey, readonly1_new_account_id);
    assert_eq!(acc_snapshot.readonly[1].pubkey, readonly2_new_account_id);
    assert_eq!(acc_snapshot.writable[0].pubkey, writable_undelegated_id);

    assert!(acc_snapshot.readonly[0].chain_state.is_new());
    assert!(acc_snapshot.readonly[1].chain_state.is_new());
    assert!(acc_snapshot.writable[0].chain_state.is_undelegated());

    assert_eq!(acc_snapshot.payer, payer_id);

    let endpoint = Endpoint::from(acc_snapshot.clone());

    eprintln!("{:#?}", endpoint);
    assert_eq!(
        endpoint,
        Endpoint::Chain {
            transaction_accounts_snapshot: acc_snapshot
        }
    );
}

#[tokio::test]
async fn test_account_meta_two_readonlys() {
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );

    let readonly1 = Pubkey::new_from_array([4u8; 32]);
    let readonly2 = Pubkey::new_from_array([5u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![readonly1, readonly2],
        ..Default::default()
    };

    let endpoint = Endpoint::from(
        TransactionAccountsSnapshot::from_accounts_holder(
            &acc_holder,
            &chain_snapshot_provider,
        )
        .await
        .unwrap(),
    );

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_chain());
}

#[tokio::test]
async fn test_account_meta_two_readonlys_one_program_and_one_writable_undelegated(
) {
    let readonly1 = Pubkey::new_from_array([4u8; 32]);
    let readonly2 = Pubkey::new_from_array([5u8; 32]);
    let writable_undelegated = Pubkey::new_from_array([6u8; 32]);
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (readonly1, account_owned_by_system_program()),
            (readonly2, program_account()),
        ],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![readonly1, readonly2],
        writable: vec![writable_undelegated],
        payer: Pubkey::new_unique(),
    };

    let endpoint = Endpoint::from(
        TransactionAccountsSnapshot::from_accounts_holder(
            &acc_holder,
            &chain_snapshot_provider,
        )
        .await
        .unwrap(),
    );
    assert!(endpoint.is_chain());

    let transaction_metas = endpoint.into_transaction_accounts_snapshot();
    assert_eq!(transaction_metas.len(), 3);
    assert_eq!(*transaction_metas[0].pubkey(), readonly1);
    assert_eq!(*transaction_metas[1].pubkey(), readonly2);
    assert_eq!(*transaction_metas[2].pubkey(), writable_undelegated);
}
