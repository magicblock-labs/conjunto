use jsonrpsee::types::{ErrorCode, ErrorObject, ErrorObjectOwned};

pub fn invalid_params(msg: String) -> ErrorObjectOwned {
    ErrorObject::owned(ErrorCode::InvalidParams.code(), msg, None::<String>)
}
