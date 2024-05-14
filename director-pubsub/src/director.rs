use conjunto_addresses::cluster::RpcCluster;
use conjunto_core::{
    AccountProvider, RequestEndpoint, SignatureStatusProvider,
};
use conjunto_guidepoint::GuideStrategyResolver;
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
    rpc_signature_status_provider::RpcSignatureStatusProvider,
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

pub struct DirectorPubsub<T: AccountProvider, U: SignatureStatusProvider> {
    config: DirectorPubsubConfig,
    guide_strategy_resolver: GuideStrategyResolver<T, U>,
}

impl<T: AccountProvider, U: SignatureStatusProvider> DirectorPubsub<T, U> {
    pub fn new(
        config: DirectorPubsubConfig,
    ) -> DirectorPubsub<RpcAccountProvider, RpcSignatureStatusProvider> {
        let rpc_provider_config =
            RpcProviderConfig::new(config.ephemeral_cluster.clone(), None);
        let ephemeral_account_provider =
            RpcAccountProvider::new(rpc_provider_config.clone());
        let ephemeral_signature_status_provider =
            RpcSignatureStatusProvider::new(rpc_provider_config);
        DirectorPubsub::with_providers(
            config,
            ephemeral_account_provider,
            ephemeral_signature_status_provider,
        )
    }

    pub fn with_providers(
        config: DirectorPubsubConfig,
        ephemeral_account_provider: T,
        ephemeral_signature_status_provider: U,
    ) -> Self {
        let guide_strategy_resolver = GuideStrategyResolver::new(
            ephemeral_account_provider,
            ephemeral_signature_status_provider,
        );
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
            // When the client is trying to close the connection we attempt to do this
            // for both endpoints to get the proper response from at least one
            Close(code) => {
                debug!("Close client: {:?}", code);
                return Some(RequestEndpoint::Both);
            }
            // We don't know which chain the ping/pong msg is responding to
            // at this point, so we send to both
            Ping(_) => return Some(RequestEndpoint::Both),
            Pong(_) => return Some(RequestEndpoint::Both),

            // If in doubt just pass on to chain
            Binary(_) => return Some(RequestEndpoint::Chain),
            Frame(_) => return Some(RequestEndpoint::Chain),
        };
        let strategy = guide_strategy_from_pubsub_msg(msg.as_str());
        let endpoint = self.guide_strategy_resolver.resolve(&strategy).await;
        trace!("Message '{}", msg);
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
