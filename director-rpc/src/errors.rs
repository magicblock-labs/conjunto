use thiserror::Error;

pub type DirectorRpcResult<T> = Result<T, DirectorRpcError>;

#[derive(Debug, Error)]
pub enum DirectorRpcError {
    #[error("JsonRpcRegisterMethodError")]
    JsonRpcRegisterMethodError(#[from] jsonrpsee::core::error::Error),
    // #[error("JsonRpcClientError")]
    // JsonRpcClientError(#[from] jsonrpsee::core::error::Error),
}
