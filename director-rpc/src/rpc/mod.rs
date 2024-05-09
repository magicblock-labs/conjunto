use conjunto_lockbox::accounts::RpcAccountProviderConfig;
use conjunto_transwise::Transwise;
use jsonrpsee::{core::RegisterMethodError, RpcModule};

use self::{
    guide::register_guide_methods, passthrough::register_passthrough_methods,
};

pub mod guide;
pub mod passthrough;

#[derive(Default)]
pub struct DirectorConfig {
    pub rpc_account_provider_config: RpcAccountProviderConfig,
}

pub struct DirectorRpc {
    pub(super) transwise: Transwise,
}

pub fn create_rpc_module(
    config: DirectorConfig,
) -> Result<RpcModule<DirectorRpc>, RegisterMethodError> {
    let transwise = Transwise::new(config.rpc_account_provider_config);
    let director = DirectorRpc { transwise };

    let mut module = RpcModule::new(director);

    register_guide_methods(&mut module)?;
    register_passthrough_methods(&mut module)?;

    Ok(module)
}
