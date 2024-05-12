use conjunto_director_rpc::start_rpc_server;
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();

    let (rpc_addr, rpc_handle) =
        start_rpc_server(Default::default(), None).await.unwrap();
    info!("RPC Server running on: {}", rpc_addr);

    tokio::join!(rpc_handle.stopped());
}
