use async_trait::async_trait;
use conjunto_lockbox::{
    account_chain_snapshot::AccountChainSnapshotProvider,
    account_chain_snapshot_shared::AccountChainSnapshotShared,
    delegation_record_parser::DelegationRecordParserImpl,
};
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
};
use solana_sdk::pubkey::Pubkey;

use crate::errors::TranswiseResult;

#[async_trait]
pub trait AccountFetcher {
    async fn fetch_account_chain_snapshot(
        &self,
        pubkey: &Pubkey,
    ) -> TranswiseResult<AccountChainSnapshotShared>;
}

pub struct RemoteAccountFetcher {
    account_chain_snapshot_provider: AccountChainSnapshotProvider<
        RpcAccountProvider,
        DelegationRecordParserImpl,
    >,
}

impl RemoteAccountFetcher {
    pub fn new(config: RpcProviderConfig) -> Self {
        let account_chain_snapshot_provider = AccountChainSnapshotProvider::new(
            RpcAccountProvider::new(config),
            DelegationRecordParserImpl,
        );
        Self {
            account_chain_snapshot_provider,
        }
    }
}

#[async_trait]
impl AccountFetcher for RemoteAccountFetcher {
    async fn fetch_account_chain_snapshot(
        &self,
        pubkey: &Pubkey,
    ) -> TranswiseResult<AccountChainSnapshotShared> {
        Ok(self
            .account_chain_snapshot_provider
            .try_fetch_chain_snapshot_of_pubkey(pubkey)
            .await?
            .into())
    }
}
