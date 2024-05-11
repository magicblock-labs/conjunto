use jsonrpsee::core::client::Subscription;
use jsonrpsee::core::{
    client::SubscriptionClientT, error::Error as RegisterMethodError,
};
use jsonrpsee::{RpcModule, SubscriptionMessage, SubscriptionSink};
use log::*;
use solana_rpc_client_api::response::SlotInfo;

use crate::errors::DirectorRpcResult;

use super::params::RawParams;
use super::DirectorPubsub;

pub fn register_subscription_methods(
    module: &mut RpcModule<DirectorPubsub>,
) -> Result<(), RegisterMethodError> {
    module.register_subscription(
        "slotSubscribe",
        "slotNotification",
        "slotUnsubscribe",
        |_params, pending, rpc| async move {
            debug!("slotSubscribe");
            let conn_id = pending.connection_id();
            if let Ok(sink) = pending.accept().await {
                if let Err(err) = rpc.try_slot_subscribe(sink).await {
                    error!(
                        "Failed to accept subscription with connection id {}: {:#?}",
                        conn_id, err
                    );
                }
            } else {
                warn!(
                    "Failed to accept subscription with connection id {}",
                    conn_id
                );
            }
        },
    )?;

    Ok(())
}

impl DirectorPubsub {
    async fn try_slot_subscribe(
        &self,
        sink: SubscriptionSink,
    ) -> DirectorRpcResult<()> {
        let mut sub: Subscription<SlotInfo> = self
            .pubsub_chain_client
            .subscribe("slotSubscribe", RawParams::new(None), "slotUnsubscribe")
            .await?;

        // This solution kind of works, however a while after unsubscribing we get the following:
        // [backend]: Failed to read message: Networking or low-level protocol error: WebSocket connection error: connection closed
        // and when we then try to subscribe with the same connection again:
        // Failed to accept subscription with connection id 0: JsonRpcRegisterMethodError(
        //     RestartNeeded(
        //         "Networking or low-level protocol error: WebSocket connection error: connection closed",
        //     ),
        // )
        // This state does not resolve itself even after disconnecting the upstream client and
        // reconnecting it.
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = sink.closed() => {
                        let _ = sub.unsubscribe().await.map_err(|err| {
                            warn!("Failed to unsubscribe from slotSubscribe: {:#?}", err)
                        });
                        break;
                    }
                    res = sub.next() => {
                        match res {
                            Some(Ok(slot_info)) => {
                                match SubscriptionMessage::new(
                                    sink.method_name(),
                                    sink.subscription_id(),
                                    &slot_info,
                                ) {
                                    Ok(notif) => if (sink.send(notif).await).is_err() {
                                        // This only happens if the sink was closed
                                        break;
                                    }
                                    Err(err) => warn!(
                                        "Got invalid slot notification: {:#?} from backend",
                                        err
                                    ),
                                }
                            }
                            Some(Err(err)) => {
                                warn!("Got invalid slot notification: {:#?} from backend", err)
                            }
                            None => {
                                break;
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }
}
