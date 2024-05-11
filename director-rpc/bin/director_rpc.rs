use std::net::SocketAddr;

use conjunto_director_rpc::rpc::create_rpc_module;
use jsonrpsee::server::{Server, ServerHandle};
use log::*;

#[tokio::main]
async fn main() {
    env_logger::init();

    let (rpc_addr, rpc_handle, pubsub_addr, pubsub_handle) = run_server().await;
    let rpc_url = format!("http://{}", rpc_addr);
    let pubsub_url = format!("ws://{}", pubsub_addr);
    info!("Servers running on: RPC {} Pubsub {}", rpc_url, pubsub_url);
    rpc_handle.stopped().await;
    pubsub_handle.stopped().await;
}

async fn run_server() -> (SocketAddr, ServerHandle, SocketAddr, ServerHandle) {
    let rpc_server = Server::builder()
        .build("127.0.0.1:8899")
        .await
        .expect("Failed to build RPC Server");
    let pubsub_server = Server::builder()
        .build("127.0.0.1:8900")
        .await
        .expect("Failed to build Pubsub Server");

    let (rpc_module, pubsub_module) = create_rpc_module(Default::default())
        .await
        .expect("Failed to create rpc module");
    let rpc_addr = rpc_server.local_addr().expect("Failed to get local addr");
    let pubsub_addr = pubsub_server
        .local_addr()
        .expect("Failed to get local addr");
    let pubsub_handle = pubsub_server.start(pubsub_module);
    let rpc_handle = rpc_server.start(rpc_module);
    (rpc_addr, rpc_handle, pubsub_addr, pubsub_handle)
}
