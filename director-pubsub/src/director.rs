use conjunto_addresses::cluster::RpcCluster;
use conjunto_core::{GuideStrategy, RequestEndpoint};
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

pub struct DirectorPubsub {
    config: DirectorPubsubConfig,
}

impl DirectorPubsub {
    pub fn new(config: DirectorPubsubConfig) -> Self {
        Self { config }
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
        use GuideStrategy::*;
        match guide_strategy_from_pubsub_msg(msg.as_str()) {
            Chain => Some(RequestEndpoint::Chain),
            Ephemeral => Some(RequestEndpoint::Ephemeral),
            Both => Some(RequestEndpoint::Both),
            // TODO: here we consult the accntwise crate to determine
            // the destination based on the strategy
            TryEphemeralForAccount(address) => {
                // TODO(thlorenz): implement correctly
                debug!("TryEphemeralForAccount: {}", address);
                Some(RequestEndpoint::Chain)
            }
            TryEphemeralForProgram(program_id) => {
                // TODO(thlorenz): implement correctly
                debug!("TryEphemeralForProgram: {}", program_id);
                Some(RequestEndpoint::Chain)
            }
            TryEphemeralForSignature(signature) => {
                debug!("TryEphemeralForSignature: {}", signature);
                // Since the subscription might come in before the transaction
                // we cannot determine 100% where to route to.
                // We could route this to just one if we do happen to find it,
                // but this implementation works for now at the cost of an extra sub.
                Some(RequestEndpoint::Both)
            }
        }
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
