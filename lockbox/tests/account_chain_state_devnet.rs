use std::str::FromStr;

use conjunto_core::AccountProvider;
use conjunto_lockbox::{
    account_chain_snapshot::AccountChainSnapshotProvider,
    account_chain_state::AccountChainState,
    delegation_record::{CommitFrequency, DelegationRecord},
};
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
};
use conjunto_test_tools::delegation_record_parser_stub::DelegationRecordParserStub;
use solana_sdk::{pubkey::Pubkey, system_program};

fn default_delegation_record() -> DelegationRecord {
    DelegationRecord {
        commit_frequency: CommitFrequency::Millis(1_000),
        owner: Pubkey::new_unique(),
    }
}

#[tokio::test]
async fn test_known_delegation() {
    // NOTE: this test depends on these accounts being present on devnet and properly locked
    let rpc_account_provider =
        RpcAccountProvider::new(RpcProviderConfig::devnet());

    let pubkey =
        Pubkey::from_str("8k2V7EzQtNg38Gi9HK5ZtQYp1YpGKNGrMcuGa737gZX4")
            .unwrap();

    let (at_slot, account) =
        rpc_account_provider.get_account(&pubkey).await.unwrap();

    let delegation_pda =
        Pubkey::from_str("CkieZJmrj6dLhwteG69LSutpWwRHiDJY9S8ua7xJ7CRW")
            .unwrap();
    let delegation_record = default_delegation_record();

    let mut delegation_record_parser = DelegationRecordParserStub::default();
    delegation_record_parser.set_next_record(delegation_record.clone());

    let account_chain_snapshot_provider = AccountChainSnapshotProvider::new(
        rpc_account_provider,
        delegation_record_parser,
    );

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(pubkey)
        .await
        .unwrap();

    assert_eq!(chain_snapshot.pubkey, pubkey);
    assert!(chain_snapshot.at_slot >= at_slot);
    assert_eq!(
        chain_snapshot.chain_state,
        AccountChainState::Delegated {
            account: account.unwrap(),
            delegation_pda,
            delegation_record,
        }
    );
}

#[tokio::test]
async fn test_system_account_not_delegated() {
    // NOTE: this test depends on devnet being up
    let rpc_account_provider =
        RpcAccountProvider::new(RpcProviderConfig::devnet());

    let pubkey = system_program::id();

    let (at_slot, account) =
        rpc_account_provider.get_account(&pubkey).await.unwrap();

    let delegation_record_parser = DelegationRecordParserStub::default();

    let account_chain_snapshot_provider = AccountChainSnapshotProvider::new(
        rpc_account_provider,
        delegation_record_parser,
    );

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(pubkey)
        .await
        .unwrap();

    assert_eq!(chain_snapshot.pubkey, pubkey);
    assert!(chain_snapshot.at_slot >= at_slot);
    assert_eq!(
        chain_snapshot.chain_state,
        AccountChainState::Undelegated {
            account: account.unwrap()
        }
    );
}
