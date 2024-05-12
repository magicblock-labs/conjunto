mod decoders;
pub mod errors;
pub mod rpc;
mod utils;

use errors::DirectorRpcResult;
use jsonrpsee::server::{Server, ServerHandle};
use rpc::{create_rpc_module, DirectorConfig};

pub const DEFAULT_DIRECTOR_RPC_URL: &str = "127.0.0.1:9899";

pub async fn start_rpc_server(
    config: DirectorConfig,
    url: Option<&str>,
) -> DirectorRpcResult<(String, ServerHandle)> {
    let url = url.unwrap_or(DEFAULT_DIRECTOR_RPC_URL);
    let server = Server::builder().build(url).await?;

    let rpc_module = create_rpc_module(config)?;
    let handle = server.start(rpc_module);
    Ok((url.to_string(), handle))
}
