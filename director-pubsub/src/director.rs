use conjunto_addresses::cluster::RpcCluster;
use log::*;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use crate::{errors::DirectorPubsubResult, BackendWebSocket};

// TODO: this needs to live a lot lower so at least accntwise can return it
pub enum RequestEndpoint {
    Chain,
    Ephemeral,
    Unroutable,
}

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

    pub(super) fn guide_msg(&self, msg: &Message) -> Option<RequestEndpoint> {
        debug!("Message: {:#?}", msg);
        use Message::*;
        let msg = match msg {
            Text(txt) => txt,
            Close(code) => {
                debug!("Close client: {:?}", code);
                return None;
            }
            Binary(_) => todo!("Binary"),
            Ping(_) => todo!("Ping"),
            Pong(_) => todo!("Pong"),
            Frame(_) => todo!("Frame"),
        };

        Some(RequestEndpoint::Ephemeral)
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
