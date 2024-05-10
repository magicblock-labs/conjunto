use conjunto_lockbox::accounts::RpcAccountProviderConfig;
use conjunto_transwise::Transwise;
use jsonrpsee::{
    http_client::{HttpClient, HttpClientBuilder},
    RpcModule,
};

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
}

pub fn create_rpc_module(
    config: DirectorConfig,
) -> DirectorRpcResult<RpcModule<DirectorRpc>> {
    let url = config.rpc_account_provider_config.url().to_string();
    let transwise = Transwise::new(config.rpc_account_provider_config);

    let rpc_client = HttpClientBuilder::default().build(url)?;

    let director = DirectorRpc {
        transwise,
        rpc_chain_client: rpc_client,
    };

    let mut module = RpcModule::new(director);

    register_guide_methods(&mut module)?;
    register_passthrough_methods(&mut module)?;

    Ok(module)
}
