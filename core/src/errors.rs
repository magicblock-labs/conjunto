use thiserror::Error;

pub type CoreResult<T> = std::result::Result<T, CoreError>;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("RpcClientError")]
    RpcClientError(#[from] solana_rpc_client_api::client_error::Error),
    #[error("PubsubClientError")]
    PubsubClientError(
        #[from]
        solana_pubsub_client::nonblocking::pubsub_client::PubsubClientError,
    ),
    #[error("Failed to get account from cluster")]
    FailedToGetAccountFromCluster,
    #[error("Failed to parse account data")]
    FailedToParseDelegationRecord(String),
}
