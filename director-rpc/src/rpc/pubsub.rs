use jsonrpsee::{core::RegisterMethodError, RpcModule};

use super::{params::RawParams, DirectorRpc};

pub fn register_subscription_methods(
    module: &mut RpcModule<DirectorRpc>,
) -> Result<(), RegisterMethodError> {
    module.register_subscription(
        "slotSubscribe",
        "slotNotification",
        "slotUnsubscribe",
        |params, pending, rpc| async move {
            let params = RawParams(params);
        },
    )?;

    Ok(())
}

impl DirectorRpc {
    async fn slotSubscribe(&self, params: RawParams) {
        self.rpc_chain_client.subscribe("slotSubscribe", params)
    }
}
