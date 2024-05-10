use conjunto_lockbox::accounts::RpcAccountProviderConfig;
use conjunto_transwise::Transwise;
use jsonrpsee::{
    core::RegisterMethodError,
    http_client::{HttpClient, HttpClientBuilder},
    RpcModule,
};

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
) -> Result<RpcModule<DirectorRpc>, RegisterMethodError> {
    let url = config.rpc_account_provider_config.url().to_string();
    let transwise = Transwise::new(config.rpc_account_provider_config);

    let rpc_client = HttpClientBuilder::default()
        .build(url)
        // TODO(thlorenz): thiserror crate to provide more generic error here?
        .expect("Failed to build HttpClient");

    let director = DirectorRpc {
        transwise,
        rpc_chain_client: rpc_client,
    };

    let mut module = RpcModule::new(director);

    register_guide_methods(&mut module)?;
    register_passthrough_methods(&mut module)?;

    Ok(module)
}
