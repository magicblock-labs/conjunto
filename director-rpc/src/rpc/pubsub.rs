use jsonrpsee::core::client::Subscription;
use jsonrpsee::core::{
    client::SubscriptionClientT, error::Error as RegisterMethodError,
};
use jsonrpsee::RpcModule;
use log::*;
use solana_rpc_client_api::response::SlotInfo;

use super::params::RawParams;
use super::DirectorPubsub;

pub fn register_subscription_methods(
    module: &mut RpcModule<DirectorPubsub>,
) -> Result<(), RegisterMethodError> {
    module.register_subscription(
        "slotSubscribe",
        "slotNotification",
        "slotUnsubscribe",
        |_params, _pending, rpc| async move {
            debug!("slotSubscribe");
            rpc.slot_subscribe().await;
        },
    )?;

    Ok(())
}

impl DirectorPubsub {
    async fn slot_subscribe(&self) {
        let mut sub: Subscription<SlotInfo> = self
            .pubsub_chain_client
            .subscribe("slotSubscribe", RawParams::new(None), "slotUnsubscribe")
            .await
            .map_err(|e| error!("Failed to subscribe to slot: {:#?}", e))
            .unwrap();

        while let Some(slot) = sub.next().await {
            debug!("slotNotification: {:?}", slot);
        }
    }
}

#[cfg(test)]
mod tests {
    use conjunto_test_tools::init_logger;
    use jsonrpsee::ws_client::WsClientBuilder;

    use super::*;

    #[tokio::test]
    async fn test_slot_subscribe_devnet() {
        init_logger!();
        let ws_url = "wss://api.devnet.solana.com";

        let ws_client = WsClientBuilder::default()
            .build(ws_url)
            .await
            .expect("Failed to build WsClient");

        let director_pubsub = DirectorPubsub {
            pubsub_chain_client: ws_client,
        };
        director_pubsub.slot_subscribe().await;
    }
}
