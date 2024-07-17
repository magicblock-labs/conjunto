use async_trait::async_trait;
use conjunto_core::{errors::CoreResult, AccountWatcher};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::{client_error::ErrorKind, request::RpcError};
use solana_sdk::{
    account::Account, commitment_config::CommitmentConfig, pubkey::Pubkey,
};

use crate::rpc_provider_config::RpcProviderConfig;

pub struct RpcAccountWatcher {
    rpc_client: RpcClient,
}

impl RpcAccountWatcher {
    pub fn new(config: RpcProviderConfig) -> Self {
        let rpc_client = RpcClient::new_with_commitment(
            config.cluster().url().to_string(),
            CommitmentConfig {
                commitment: config.commitment().unwrap_or_default(),
            },
        );
        Self { rpc_client }
    }
}

#[async_trait]
impl AccountWatcher for RpcAccountWatcher {}

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey::Pubkey;

    use super::*;

    #[tokio::test]
    async fn test_get_non_existing_account() {
        /*
          let rpc_account_provider = RpcAccountWatcher::default();
          let pubkey = Pubkey::new_from_array([5; 32]);
          let account = rpc_account_provider.get_account(&pubkey).await.unwrap();
          assert!(account.is_none());
        */
    }
}
