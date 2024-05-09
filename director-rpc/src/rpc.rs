use async_trait::async_trait;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use log::*;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::UiTransactionEncoding;

use crate::{decoders::decode_and_deserialize, utils::invalid_params};

#[rpc(server)]
pub trait DirectorRpc {
    #[method(name = "sendTransaction")]
    async fn send_transaction(
        &self,
        tx: String,
        send_transaction_config: Option<RpcSendTransactionConfig>,
    ) -> RpcResult<String>;
}

pub struct DirectorRpcImpl;

#[async_trait]
impl DirectorRpcServer for DirectorRpcImpl {
    async fn send_transaction(
        &self,
        data: String,
        config: Option<RpcSendTransactionConfig>,
    ) -> RpcResult<String> {
        debug!("send_transaction rpc request received");
        let RpcSendTransactionConfig {
            skip_preflight: _,
            preflight_commitment: _,
            encoding,
            max_retries: _,
            min_context_slot: _,
        } = config.unwrap_or_default();

        let tx_encoding = encoding.unwrap_or(UiTransactionEncoding::Base58);

        let binary_encoding = tx_encoding.into_binary_encoding().ok_or_else(|| {
                invalid_params(format!(
                    "unsupported encoding: {tx_encoding}. Supported encodings: base58, base64"
                ))
            })?;
        let (_, unsanitized_tx) = decode_and_deserialize::<VersionedTransaction>(
            data,
            binary_encoding,
        )?;

        info!("tx: {:#?}", unsanitized_tx);

        Ok("send_transaction".to_string())
    }
}
