use conjunto_addresses::cluster::RpcCluster;
use conjunto_core::{AccountProvider, RequestEndpoint};
use conjunto_guidepoint::GuideStrategyResolver;
use conjunto_providers::rpc_account_provider::{
    RpcAccountProvider, RpcAccountProviderConfig,
};
use log::*;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use crate::{
    errors::DirectorPubsubResult,
    guide_strategy::guide_strategy_from_pubsub_msg, BackendWebSocket,
};

pub struct DirectorPubsubConfig {
    pub chain_cluster: RpcCluster,
    pub ephemeral_cluster: RpcCluster,
}

impl Default for DirectorPubsubConfig {
    fn default() -> Self {
        Self {
            chain_cluster: Default::default(),
            ephemeral_cluster: RpcCluster::Development,
        }
    }
}

pub struct DirectorPubsub<T: AccountProvider> {
    config: DirectorPubsubConfig,
    guide_strategy_resolver: GuideStrategyResolver<T>,
}

impl<T: AccountProvider> DirectorPubsub<T> {
    pub fn new(
        config: DirectorPubsubConfig,
    ) -> DirectorPubsub<RpcAccountProvider> {
        let acc_provider_config = RpcAccountProviderConfig::new(
            config.ephemeral_cluster.clone(),
            None,
        );
        let ephemeral_account_provider =
            RpcAccountProvider::new(acc_provider_config);
        DirectorPubsub::with_providers(config, ephemeral_account_provider)
    }

    pub fn with_providers(
        config: DirectorPubsubConfig,
        ephemeral_account_provider: T,
    ) -> Self {
        let guide_strategy_resolver =
            GuideStrategyResolver::new(ephemeral_account_provider);
        Self {
            config,
            guide_strategy_resolver,
        }
    }

    pub(super) async fn guide_msg(
        &self,
        msg: &Message,
    ) -> Option<RequestEndpoint> {
        use Message::*;
        let msg = match msg {
            Text(txt) => txt,
            Close(code) => {
                debug!("Close client: {:?}", code);
                return None;
            }
            // If in doubt just pass on to chain
            Binary(_) => return Some(RequestEndpoint::Chain),
            Ping(_) => return Some(RequestEndpoint::Chain),
            Pong(_) => return Some(RequestEndpoint::Chain),
            Frame(_) => return Some(RequestEndpoint::Chain),
        };
        let strategy = guide_strategy_from_pubsub_msg(msg.as_str());
        let endpoint = self.guide_strategy_resolver.resolve(&strategy).await;
        debug!("Guiding message to: {:?}", endpoint);
        Some(endpoint)
    }

    pub async fn try_chain_client(
        &self,
    ) -> DirectorPubsubResult<BackendWebSocket> {
        let url = self.config.chain_cluster.ws_url();
        let (socket, _) = connect_async(Url::parse(url)?).await?;
        Ok(socket)
    }

    pub async fn try_ephemeral_client(
        &self,
    ) -> DirectorPubsubResult<BackendWebSocket> {
        let url = self.config.ephemeral_cluster.ws_url();
        let (socket, _) = connect_async(Url::parse(url)?).await?;
        Ok(socket)
    }
}
