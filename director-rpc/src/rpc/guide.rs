use jsonrpsee::{
    core::{RegisterMethodError, RpcResult},
    RpcModule,
};
use log::*;
use serde::Deserialize;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::UiTransactionEncoding;

use super::DirectorRpc;
use crate::{
    decoders::decode_and_deserialize,
    utils::{
        invalid_params, server_error, server_error_with_data, ServerErrorCode,
    },
};

#[derive(Debug, Deserialize)]
struct SendTransactionParams(
    String,
    #[serde(default)] Option<RpcSendTransactionConfig>,
);

pub fn register_guide_methods(
    module: &mut RpcModule<DirectorRpc>,
) -> Result<(), RegisterMethodError> {
    module.register_async_method(
        "sendTransaction",
        |params, rpc| async move {
            debug!("send_transaction rpc request received {:#?}", params);
            let SendTransactionParams(data, config) =
                params.parse::<SendTransactionParams>()?;

            rpc.send_transaction(data, config).await?;
            RpcResult::Ok("send_transaction rpc request received".to_string())
        },
    )?;

    Ok(())
}

impl DirectorRpc {
    async fn send_transaction(
        &self,
        data: String,
        config: Option<RpcSendTransactionConfig>,
    ) -> RpcResult<String> {
        debug!("send_transaction rpc request received");
        // 1. Deserialize Transaction
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
        let (_, versioned_tx) = decode_and_deserialize::<VersionedTransaction>(
            data,
            binary_encoding,
        )?;

        // 2. Determine Endpoint to be used for this Transaction
        let endpoint = match self
            .transwise
            .guide_versioned_transaction(&versioned_tx)
            .await
        {
            Ok(endpoint) => endpoint,
            Err(err) => {
                return Err(server_error(
                    format!("error: {err}"),
                    ServerErrorCode::FailedToFetchEndpointInformation,
                ));
            }
        };
        if endpoint.is_unroutable() {
            return Err(server_error_with_data(
                "Transaction is unroutable".to_string(),
                ServerErrorCode::TransactionUnroutable,
                endpoint,
            ));
        }

        // 3. Route transaction accordingly

        info!("endpoint: {:#?}", endpoint);

        Ok("send_transaction".to_string())
    }
}
