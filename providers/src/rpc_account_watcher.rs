use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, RwLock},
};

use futures_util::{
    select, stream::FuturesUnordered, Future, FutureExt, Stream, StreamExt,
};
use log::*;
use solana_account_decoder::{UiAccount, UiDataSliceConfig};
use tokio::sync::mpsc::{
    unbounded_channel, UnboundedReceiver, UnboundedSender,
};

use crate::rpc_provider_config::RpcProviderConfig;
use async_trait::async_trait;
use conjunto_core::{
    errors::{CoreError, CoreResult},
    AccountWatcher,
};
use solana_pubsub_client::nonblocking::pubsub_client::PubsubClient;
use solana_rpc_client_api::{config::RpcAccountInfoConfig, response::Response};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
};

struct RpcAccountWatcherSubscribe {
    pub account: Pubkey,
    pub sender: UnboundedSender<u64>,
}

struct RpcAccountWatcherUnsubscribe {
    pub account: Pubkey,
}

pub struct RpcAccountWatcher {
    config: RpcProviderConfig,
    //counters: HashMap<Pubkey, u64>,
    slots: Arc<RwLock<HashMap<Pubkey, u64>>>,
    subscribe_receiver: UnboundedReceiver<RpcAccountWatcherSubscribe>,
    subscribe_sender: UnboundedSender<RpcAccountWatcherSubscribe>,
    unsubscribe_receiver: UnboundedReceiver<RpcAccountWatcherUnsubscribe>,
    unsubscribe_sender: UnboundedSender<RpcAccountWatcherUnsubscribe>,
}

impl RpcAccountWatcher {
    pub fn new(config: RpcProviderConfig) -> Self {
        let (subscribe_sender, subscribe_receiver) =
            unbounded_channel::<RpcAccountWatcherSubscribe>();
        let (unsubscribe_sender, unsubscribe_receiver) =
            unbounded_channel::<RpcAccountWatcherUnsubscribe>();
        Self {
            config,
            slots: Default::default(),
            subscribe_sender,
            subscribe_receiver,
            unsubscribe_sender,
            unsubscribe_receiver,
        }
    }

    async fn run(&mut self) -> CoreResult<()> {
        let pubsub_client = Arc::new(
            PubsubClient::new(self.config.ws_url())
                .await
                .map_err(CoreError::PubsubClientError)?,
        );
        let commitment = self.config.commitment();

        let slots = self.slots.clone();

        let mut cancel_senders = HashMap::new();
        let mut join_handles = vec![];

        tokio::select! {
            Some(subscribe) = self.subscribe_receiver.recv() => {
                if !cancel_senders.contains_key(&subscribe.account) {
                    let (cancel_sender, cancel_receiver) = unbounded_channel::<()>();
                    cancel_senders.insert(subscribe.account, cancel_sender);
                    join_handles.push((subscribe.account, tokio::spawn(async move {
                        let result = RpcAccountWatcher::monitor_account(
                            slots,
                            pubsub_client,
                            commitment,
                            subscribe.account,
                            cancel_receiver,
                            subscribe.sender
                        ).await;
                        if let Err(error) = result {
                            warn!("Failed to monitor account: {}: {:?}", subscribe.account, error);
                        }
                    })));
                }
            }
            Some(unsubscribe) = self.unsubscribe_receiver.recv() => {
                if let Some(cancel_sender) = cancel_senders.remove(&unsubscribe.account) {
                    if let Err(error) = cancel_sender.send(()) {
                        warn!("Failed to cancel monitoring of account: {}: {:?}", unsubscribe.account, error);
                    }
                }
            }
        }

        for (account, handle) in join_handles {
            //debug!("waiting on subscribe {}", account);
            if let Err(error) = handle.await {
                //debug!("subscribe {} failed: {}", account, error);
            }
        }

        Ok(())
    }

    async fn monitor_account(
        slots: Arc<RwLock<HashMap<Pubkey, u64>>>,
        pubsub_client: Arc<PubsubClient>,
        commitment: Option<CommitmentLevel>,
        account: Pubkey,
        cancel_receiver: UnboundedReceiver<()>,
        update_sender: UnboundedSender<u64>,
    ) -> CoreResult<()> {
        let config = Some(RpcAccountInfoConfig {
            commitment: commitment
                .map(|commitment| CommitmentConfig { commitment }),
            encoding: None,
            data_slice: None,
            /*
            data_slice: Some(UiDataSliceConfig {
                offset: 0,
                length: 0,
            }),
             */
            min_context_slot: None,
        });

        let (mut stream, unsubscribe) = pubsub_client
            .account_subscribe(&account, config)
            .await
            .map_err(CoreError::PubsubClientError)?;

        while let Some(update) = stream.next().await {
            let slot = update.context.slot;

            let mut slots_write = slots.write().unwrap();
            let previous_slot = slots_write.remove(&account);
            if let Some(previous_slot) = previous_slot {
                if previous_slot >= slot {
                    continue;
                }
            }
            slots_write.insert(account, slot);
        }

        /*
        loop {
            select! {
                update = stream.next() => {
                }
                _ = cancel_receiver.recv() => {
                    break;
                }
            }
        }
        */

        Ok(())
    }
}

#[async_trait]
impl AccountWatcher for RpcAccountWatcher {
    async fn start_monitoring_account(
        &self,
        pubkey: &Pubkey,
    ) -> CoreResult<()> {
        if let Err(error) =
            self.subscribe_sender.send(RpcAccountWatcherSubscribe {
                account: pubkey.clone(),
                sender,
            })
        {
            warn!("Failed to subscribe to account: {}: {:?}", pubkey, error)
        }
        Ok(())
    }

    fn test(&self, pubkey: &Pubkey, slot: u64) -> bool {
        let slots_read = self.slots.read().unwrap();
        if let Some(last_update_slot) = slots_read.get(pubkey) {
            *last_update_slot > slot
        } else {
            false
        }
    }

    async fn stop_monitoring_account(&self, pubkey: &Pubkey) -> CoreResult<()> {
        if let Err(error) =
            self.unsubscribe_sender.send(RpcAccountWatcherUnsubscribe {
                account: pubkey.clone(),
            })
        {
            warn!("Failed to unsubscribe to account: {}: {:?}", pubkey, error)
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_non_existing_account() {
        /*
          let rpc_account_provider = RpcAccountWatcher::default();
          let pubkey = Pubkey::new_from_array([5; 32]);
          let account = rpc_account_provider.get_account(&pubkey).await.unwrap();
          assert!(account.is_none());
        */
    }
}
