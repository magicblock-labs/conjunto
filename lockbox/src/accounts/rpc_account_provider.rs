use async_trait::async_trait;
use conjunto_addresses::cluster::RpcCluster;
use conjunto_core::{errors::CoreResult, AccountProvider};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::{client_error::ErrorKind, request::RpcError};
use solana_sdk::{
    account::Account,
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
};

#[derive(Default)]
pub struct RpcAccountProviderConfig {
    cluster: RpcCluster,
    commitment: CommitmentLevel,
}

impl RpcAccountProviderConfig {
    pub fn cluster(&self) -> &RpcCluster {
        &self.cluster
    }

    pub fn url(&self) -> &str {
        self.cluster.url()
    }

    pub fn ws_url(&self) -> &str {
        self.cluster.ws_url()
    }

    pub fn commitment(&self) -> CommitmentLevel {
        self.commitment
    }
}

pub struct RpcAccountProvider {
    rpc_client: RpcClient,
}

impl RpcAccountProvider {
    pub fn new(config: RpcAccountProviderConfig) -> Self {
        let rpc_client = RpcClient::new_with_commitment(
            config.cluster.url().to_string(),
            CommitmentConfig {
                commitment: config.commitment,
            },
        );
        Self { rpc_client }
    }
}

impl Default for RpcAccountProvider {
    fn default() -> Self {
        Self::new(RpcAccountProviderConfig::default())
    }
}

#[async_trait]
impl AccountProvider for RpcAccountProvider {
    async fn get_account(
        &self,
        pubkey: &Pubkey,
    ) -> CoreResult<Option<Account>> {
        let account = match self.rpc_client.get_account(pubkey).await {
            Ok(acc) => Some(acc),
            Err(err) => match err.kind() {
                ErrorKind::RpcError(RpcError::ForUser(msg)) => {
                    if msg.contains("AccountNotFound") {
                        None
                    } else {
                        return Err(err.into());
                    }
                }
                _ => {
                    return Err(err.into());
                }
            },
        };
        Ok(account)
    }

    async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> CoreResult<Vec<Option<Account>>> {
        Ok(self.rpc_client.get_multiple_accounts(pubkeys).await?)
    }
}

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey::Pubkey;

    use super::*;

    #[tokio::test]
    async fn test_get_non_existing_account() {
        let rpc_account_provider = RpcAccountProvider::default();
        let pubkey = Pubkey::new_from_array([5; 32]);
        let account = rpc_account_provider.get_account(&pubkey).await.unwrap();
        assert!(account.is_none());
    }

    #[tokio::test]
    async fn test_get_existing_account() {
        let rpc_account_provider = RpcAccountProvider::default();
        let pubkey = Pubkey::default();
        let account = rpc_account_provider.get_account(&pubkey).await.unwrap();
        assert!(account.is_some());
    }

    #[tokio::test]
    async fn test_get_multiple_accounts() {
        let rpc_account_provider = RpcAccountProvider::default();
        let pubkeys = vec![Pubkey::default(), Pubkey::new_from_array([5; 32])];
        let accounts = rpc_account_provider
            .get_multiple_accounts(&pubkeys)
            .await
            .unwrap();
        assert_eq!(accounts.len(), 2);

        assert!(accounts[0].is_some());
        assert!(accounts[1].is_none());
    }
}
