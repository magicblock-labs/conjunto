use std::net::SocketAddr;

use conjunto_director_rpc::rpc::create_rpc_module;
use jsonrpsee::server::{Server, ServerHandle};
use log::*;

#[tokio::main]
async fn main() {
    env_logger::init();

    let (addr, handle) = run_server().await;
    let url = format!("http://{}", addr);
    info!("Server running on: {}", url);
    handle.stopped().await;
}

async fn run_server() -> (SocketAddr, ServerHandle) {
    let server = Server::builder()
        .build("127.0.0.1:8899")
        .await
        .expect("Failed to build Server");

    let rpc_module = create_rpc_module(Default::default())
        .await
        .expect("Failed to create rpc module");
    let addr = server.local_addr().expect("Failed to get local addr");
    let handle = server.start(rpc_module);
    (addr, handle)
}
