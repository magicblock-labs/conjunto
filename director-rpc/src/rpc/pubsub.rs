use jsonrpsee::{core::RegisterMethodError, RpcModule};
use log::*;

use super::{params::RawParams, DirectorRpc};

pub fn register_subscription_methods(
    module: &mut RpcModule<DirectorRpc>,
) -> Result<(), RegisterMethodError> {
    module.register_subscription(
        "slotSubscribe",
        "slotNotification",
        "slotUnsubscribe",
        |params, pending, rpc| async move {
            debug!("slotSubscribe");
            trace!("{:#?}", params);
            let params = RawParams(params);
        },
    )?;

    Ok(())
}

impl DirectorRpc {
    async fn slotSubscribe(&self, params: RawParams) {
        self.pubsub_chain_client
            .subscribe_("slotSubscribe", params)
            .await;
    }
}
