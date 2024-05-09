use std::net::SocketAddr;

use conjunto_director_rpc::{DirectorRpcImpl, DirectorRpcServer as _};
use jsonrpsee::server::{Server, ServerHandle};

#[tokio::main]
async fn main() {
    let (addr, handle) = run_server().await;
    let url = format!("http://{}", addr);
    eprintln!("Server running on: {}", url);
    handle.stopped().await;
}

async fn run_server() -> (SocketAddr, ServerHandle) {
    let server = Server::builder()
        .build("127.0.0.1:8899")
        .await
        .expect("Failed to build Server");

    let addr = server.local_addr().expect("Failed to get local addr");
    let handle = server.start(DirectorRpcImpl.into_rpc());
    (addr, handle)
}
