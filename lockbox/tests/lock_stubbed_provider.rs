use conjunto_core::{CommitFrequency, DelegationRecord};
use conjunto_lockbox::{
    AccountChainState, AccountChainStateProvider, LockInconsistency,
};
use conjunto_test_tools::{
    account_provider_stub::AccountProviderStub,
    accounts::{
        account_owned_by_delegation_program, account_owned_by_system_program,
        delegated_account_ids,
    },
    delegation_record_parser_stub::DelegationRecordParserStub,
};
use solana_sdk::{account::Account, pubkey::Pubkey};

fn default_delegation_record() -> DelegationRecord {
    DelegationRecord {
        commit_frequency: CommitFrequency::Millis(1_000),
        owner: Pubkey::new_unique(),
    }
}

fn setup(
    accounts: Vec<(Pubkey, Account)>,
    delegation_record: Option<DelegationRecord>,
) -> AccountChainStateProvider<AccountProviderStub, DelegationRecordParserStub>
{
    let mut account_provider = AccountProviderStub::default();
    for (pubkey, account) in accounts {
        account_provider.add(pubkey, account);
    }
    let mut delegation_record_parser = DelegationRecordParserStub::default();
    if let Some(record) = delegation_record {
        delegation_record_parser.set_next_record(record);
    }
    AccountChainStateProvider::with_provider_and_parser(
        account_provider,
        delegation_record_parser,
    )
}

#[tokio::test]
async fn test_delegate_properly_delegated() {
    let (delegated_id, delegation_pda) = delegated_account_ids();
    let delegation_record = default_delegation_record();
    let chain_state_provider = setup(
        vec![
            (delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_delegation_program()),
        ],
        Some(delegation_record.clone()),
    );

    let state = chain_state_provider
        .try_fetch_chain_state_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(
        state,
        AccountChainState::Delegated {
            delegated_id,
            delegation_pda,
            config: delegation_record.into(),
        }
    );
}

#[tokio::test]
async fn test_delegate_undelegated() {
    let (delegated_id, delegation_pda) = delegated_account_ids();
    let chain_state_provider = setup(
        vec![
            (delegated_id, account_owned_by_system_program()),
            // The other accounts don't matter since we don't check them if no lock is present
            (delegation_pda, account_owned_by_delegation_program()),
        ],
        None,
    );

    let state = chain_state_provider
        .try_fetch_chain_state_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert!(matches!(state, AccountChainState::Undelegated { .. }));
}

#[tokio::test]
async fn test_delegate_not_found() {
    let (delegated_id, delegation_pda) = delegated_account_ids();
    let chain_state_provider = setup(
        vec![
            // The other accounts don't matter since we don't check them if delegated
            // account is missing
            (delegation_pda, account_owned_by_delegation_program()),
        ],
        None,
    );

    let state = chain_state_provider
        .try_fetch_chain_state_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(state, AccountChainState::NewAccount);
}

#[tokio::test]
async fn test_delegate_missing_delegate_account() {
    let (delegated_id, delegation_pda) = delegated_account_ids();

    let chain_state_provider = setup(
        vec![(delegated_id, account_owned_by_delegation_program())],
        None,
    );

    let state = chain_state_provider
        .try_fetch_chain_state_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(
        state,
        AccountChainState::Inconsistent {
            delegated_id,
            delegation_pda,
            inconsistencies: vec![LockInconsistency::DelegationAccountNotFound]
        }
    );
}

#[tokio::test]
async fn test_delegate_delegation_not_owned_by_delegate_program() {
    let (delegated_id, delegation_pda) = delegated_account_ids();
    let delegation_record = default_delegation_record();
    let chain_state_provider = setup(
        vec![
            (delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_system_program()),
        ],
        Some(delegation_record.clone()),
    );

    let state = chain_state_provider
        .try_fetch_chain_state_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(
        state,
        AccountChainState::Inconsistent {
            delegated_id,
            delegation_pda,
            inconsistencies: vec![
                LockInconsistency::DelegationAccountInvalidOwner
            ]
        }
    );
}

#[tokio::test]
async fn test_delegate_delegation_not_owned_by_delegate_program_and_invalid_record(
) {
    let (delegated_id, delegation_pda) = delegated_account_ids();
    let chain_state_provider = setup(
        vec![
            (delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_system_program()),
        ],
        None,
    );

    let state = chain_state_provider
        .try_fetch_chain_state_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(
        state,
        AccountChainState::Inconsistent {
            delegated_id,
            delegation_pda,
            inconsistencies: vec![
                LockInconsistency::DelegationAccountInvalidOwner,
                LockInconsistency::DelegationRecordAccountDataInvalid(
                    "Failed to parse account data".to_string()
                )
            ]
        }
    );
}
