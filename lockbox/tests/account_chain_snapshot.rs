use conjunto_core::{
    delegation_inconsistency::DelegationInconsistency,
    delegation_record::{CommitFrequency, DelegationRecord},
};
use conjunto_lockbox::{
    account_chain_snapshot::AccountChainSnapshot,
    account_chain_snapshot_provider::AccountChainSnapshotProvider,
    account_chain_state::AccountChainState,
};
use conjunto_test_tools::{
    account_provider_stub::AccountProviderStub,
    accounts::{
        account_owned_by_delegation_program, account_owned_by_system_program,
        delegated_account_ids,
    },
    delegation_record_parser_stub::DelegationRecordParserStub,
};
use solana_sdk::{
    account::Account, clock::Slot, pubkey::Pubkey, signature::Keypair,
    signer::Signer,
};

const EXPECTED_SLOT: Slot = 42;

fn dummy_delegation_record() -> DelegationRecord {
    DelegationRecord {
        authority: Pubkey::new_unique(),
        owner: Pubkey::new_unique(),
        delegation_slot: 0,
        commit_frequency: CommitFrequency::Millis(1_000),
    }
}

fn setup(
    accounts: Vec<(Pubkey, Account)>,
    delegation_record: Option<DelegationRecord>,
) -> AccountChainSnapshotProvider<AccountProviderStub, DelegationRecordParserStub>
{
    let mut account_provider = AccountProviderStub::default();
    account_provider.at_slot = EXPECTED_SLOT;
    for (pubkey, account) in accounts {
        account_provider.add(pubkey, account);
    }
    let mut delegation_record_parser = DelegationRecordParserStub::default();
    if let Some(record) = delegation_record {
        delegation_record_parser.set_next_record(record);
    }
    AccountChainSnapshotProvider::new(
        account_provider,
        delegation_record_parser,
    )
}

#[tokio::test]
async fn test_snapshot_wallet_new() {
    let pubkey = Keypair::new().pubkey();
    let account = account_owned_by_system_program();

    let account_chain_snapshot_provider = setup(vec![], None);

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::Wallet { account }
        }
    );
}

#[tokio::test]
async fn test_snapshot_wallet_exists() {
    let pubkey = Keypair::new().pubkey();
    let mut account = account_owned_by_system_program();
    account.lamports = 42;

    let account_chain_snapshot_provider =
        setup(vec![(pubkey, account.clone())], None);

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::Wallet { account }
        }
    );
}

#[tokio::test]
async fn test_snapshot_delegated() {
    let account = account_owned_by_delegation_program();

    let (pubkey, delegation_record_pubkey) = delegated_account_ids();
    let delegation_record = dummy_delegation_record();
    let account_chain_snapshot_provider = setup(
        vec![
            (pubkey, account.clone()),
            (
                delegation_record_pubkey,
                account_owned_by_delegation_program(),
            ),
        ],
        Some(delegation_record.clone()),
    );

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::Delegated {
                account,
                delegation_record,
            }
        }
    );
}

#[tokio::test]
async fn test_snapshot_account_invalid_owner() {
    let account = account_owned_by_system_program();

    let (pubkey, delegation_record_pubkey) = delegated_account_ids();
    let account_chain_snapshot_provider = setup(
        vec![
            (pubkey, account.clone()),
            // The other accounts don't matter since we don't check them if no lock is present
            (
                delegation_record_pubkey,
                account_owned_by_delegation_program(),
            ),
        ],
        None,
    );

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::Undelegated {
                account,
                delegation_inconsistency:
                    DelegationInconsistency::AccountInvalidOwner,
            }
        }
    );
}

#[tokio::test]
async fn test_snapshot_account_not_found() {
    let (pubkey, delegation_record_pubkey) = delegated_account_ids();
    let account_chain_snapshot_provider = setup(
        vec![
            // The other accounts don't matter since we don't check them if delegated
            // account is missing
            (
                delegation_record_pubkey,
                account_owned_by_delegation_program(),
            ),
        ],
        None,
    );

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::Undelegated {
                account: Account::default(),
                delegation_inconsistency:
                    DelegationInconsistency::AccountNotFound
            }
        }
    );
}

#[tokio::test]
async fn test_snapshot_delegation_record_not_found() {
    let account = account_owned_by_delegation_program();

    let (pubkey, _delegation_record_pda) = delegated_account_ids();

    let account_chain_snapshot_provider =
        setup(vec![(pubkey, account.clone())], None);

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::Undelegated {
                account,
                delegation_inconsistency:
                    DelegationInconsistency::DelegationRecordNotFound,
            }
        }
    );
}

#[tokio::test]
async fn test_snapshot_delegation_record_invalid_owner() {
    let account = account_owned_by_delegation_program();

    let (pubkey, delegation_record_pubkey) = delegated_account_ids();
    let delegation_record = dummy_delegation_record();
    let account_chain_snapshot_provider = setup(
        vec![
            (pubkey, account_owned_by_delegation_program()),
            (delegation_record_pubkey, account_owned_by_system_program()),
        ],
        Some(delegation_record.clone()),
    );

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::Undelegated {
                account,
                delegation_inconsistency:
                    DelegationInconsistency::DelegationRecordInvalidOwner,
            }
        }
    );
}

#[tokio::test]
async fn test_snapshot_delegation_record_data_invalid() {
    let account = account_owned_by_delegation_program();

    let (pubkey, delegation_record_pubkey) = delegated_account_ids();
    let account_chain_snapshot_provider = setup(
        vec![
            (pubkey, account_owned_by_delegation_program()),
            (
                delegation_record_pubkey,
                account_owned_by_delegation_program(),
            ),
        ],
        None,
    );

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::Undelegated {
                account,
                delegation_inconsistency:
                    DelegationInconsistency::DelegationRecordDataInvalid(
                        "Failed to parse account data".to_string()
                    ),
            }
        }
    );
}
