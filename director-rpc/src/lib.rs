use async_trait::async_trait;
use jsonrpsee::proc_macros::rpc;
use solana_rpc_client_api::config::RpcSendTransactionConfig;

pub type Result<T> = std::result::Result<T, jsonrpsee::types::ErrorObjectOwned>;

#[rpc(server)]
pub trait DirectorRpc {
    #[method(name = "sendTransaction")]
    async fn send_transaction(
        &self,
        tx: String,
        send_transaction_config: Option<RpcSendTransactionConfig>,
    ) -> Result<String>;
}

pub struct DirectorRpcImpl;

#[async_trait]
impl DirectorRpcServer for DirectorRpcImpl {
    async fn send_transaction(
        &self,
        data: String,
        send_transaction_config: Option<RpcSendTransactionConfig>,
    ) -> Result<String> {
        eprintln!(
            "send_transaction: tx: {}, config: {:?}",
            data, send_transaction_config
        );
        Ok("send_transaction".to_string())
    }
}
