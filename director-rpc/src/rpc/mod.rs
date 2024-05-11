use conjunto_lockbox::accounts::RpcAccountProviderConfig;
use conjunto_transwise::Transwise;
use jsonrpsee::{
    http_client::{HttpClient, HttpClientBuilder},
    RpcModule,
};
use jsonrpsee_ws_client::{WsClient, WsClientBuilder};

use crate::errors::DirectorRpcResult;

use self::{
    guide::register_guide_methods, passthrough::register_passthrough_methods,
};

pub mod guide;
mod params;
pub mod passthrough;

#[derive(Default)]
pub struct DirectorConfig {
    pub rpc_account_provider_config: RpcAccountProviderConfig,
}

pub struct DirectorRpc {
    pub(super) transwise: Transwise,
    pub(super) rpc_chain_client: HttpClient,
    pub(super) pubsub_chain_client: WsClient,
}

pub async fn create_rpc_module(
    config: DirectorConfig,
) -> DirectorRpcResult<RpcModule<DirectorRpc>> {
    let url = config.rpc_account_provider_config.url().to_string();
    let ws_url = config.rpc_account_provider_config.ws_url().to_string();
    let transwise = Transwise::new(config.rpc_account_provider_config);

    let rpc_client = HttpClientBuilder::default().build(url)?;
    let ws_client = WsClientBuilder::default()
        .build(ws_url)
        .await
        // TODO: properly propagate
        .expect("Failed to build WsClient");

    let director = DirectorRpc {
        transwise,
        rpc_chain_client: rpc_client,
        pubsub_chain_client: ws_client,
    };

    let mut module = RpcModule::new(director);

    register_guide_methods(&mut module)?;
    register_passthrough_methods(&mut module)?;

    Ok(module)
}
