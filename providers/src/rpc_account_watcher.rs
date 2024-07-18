use std::{collections::HashMap, time::Instant};

use crate::rpc_provider_config::RpcProviderConfig;
use async_trait::async_trait;
use conjunto_core::{
    errors::{CoreError, CoreResult},
    AccountWatcher,
};
use solana_pubsub_client::nonblocking::pubsub_client::PubsubClient;
use solana_rpc_client_api::config::RpcAccountInfoConfig;
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
};

pub struct RpcAccountWatcher {
    commitment: Option<CommitmentLevel>,
    pubsub_client: PubsubClient,
    subscribed_accounts: HashMap<Pubkey, Instant>,
}

impl RpcAccountWatcher {
    pub async fn new(config: RpcProviderConfig) -> CoreResult<Self> {
        let pubsub_client = PubsubClient::new(config.ws_url())
            .await
            .map_err(CoreError::PubsubClientError)?;
        Ok(Self {
            commitment: config.commitment(),
            pubsub_client,
            subscribed_accounts: Default::default(),
        })
    }
}

#[async_trait]
impl AccountWatcher for RpcAccountWatcher {
    async fn test(&self) -> Option<bool> {
        let pubkey = &Pubkey::default();
        let config = Some(RpcAccountInfoConfig {
            commitment: self
                .commitment
                .map(|commitment| CommitmentConfig { commitment }),
            encoding: None,
            data_slice: None,
            min_context_slot: None,
        });
        let dudu = self.pubsub_client.account_subscribe(pubkey, config).await;
        let val = match dudu {
            Ok(value) => value,
            Err(_) => return None,
        };
        val.0.as_mut().poll_next()
    }
}

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
