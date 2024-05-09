use conjunto_lockbox::AccountLockStateProvider;
use conjunto_test_tools::{
    account_provider_stub::AccountProviderStub,
    accounts::{account_owned_by_delegation_program, delegated_account_ids},
    transaction_accounts_holder_stub::TransactionAccountsHolderStub,
};
use conjunto_transwise::trans_account_meta::TransAccountMetas;
use solana_sdk::{account::Account, pubkey::Pubkey};

fn setup_lockstate_provider(
    accounts: Vec<(Pubkey, Account)>,
) -> AccountLockStateProvider<AccountProviderStub> {
    let mut account_provider = AccountProviderStub::default();
    for (pubkey, account) in accounts {
        account_provider.add(pubkey, account);
    }
    AccountLockStateProvider::with_provider(account_provider)
}

#[tokio::test]
async fn test_account_meta_one_properly_locked_writable_and_one_readonly() {
    let (delegated_id, buffer_pda, delegation_pda) = delegated_account_ids();
    let lockstate_provider = setup_lockstate_provider(vec![
        (delegated_id, account_owned_by_delegation_program()),
        (buffer_pda, account_owned_by_delegation_program()),
        (delegation_pda, account_owned_by_delegation_program()),
    ]);
    let readonly_id = Pubkey::new_from_array([4u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![readonly_id],
        writable: vec![delegated_id],
    };

    let account_metas = TransAccountMetas::from_accounts_holder(
        &acc_holder,
        &lockstate_provider,
    )
    .await
    .unwrap();
    let endpoint = account_metas.into_endpoint();

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_ephemeral());
}
