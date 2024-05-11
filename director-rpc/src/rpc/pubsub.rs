use jsonrpsee::core::client::Subscription;
use jsonrpsee::core::{
    client::SubscriptionClientT, error::Error as RegisterMethodError,
};
use jsonrpsee::RpcModule;
use log::*;
use solana_sdk::clock::Slot;

use super::params::RawParams;
use super::DirectorPubsub;

pub fn register_subscription_methods(
    module: &mut RpcModule<DirectorPubsub>,
) -> Result<(), RegisterMethodError> {
    module.register_subscription(
        "slotSubscribe",
        "slotNotification",
        "slotUnsubscribe",
        |params, _pending, rpc| async move {
            debug!("slotSubscribe");
            trace!("{:#?}", params);
            let params = RawParams(params);
            rpc.slot_subscribe(params).await;
        },
    )?;

    Ok(())
}

impl DirectorPubsub {
    async fn slot_subscribe(&self, params: RawParams) {
        let mut sub: Subscription<Slot> = self
            .pubsub_chain_client
            .subscribe("slotSubscribe", params, "slotUnsubscribe")
            .await
            .unwrap();
        while let Some(slot) = sub.next().await {
            debug!("slotNotification: {:?}", slot);
        }
    }
}
