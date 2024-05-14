use std::sync::Arc;

use crate::{
    director::DirectorPubsub, errors::DirectorPubsubResult, BackendWebSocket,
};
use conjunto_core::{AccountProvider, RequestEndpoint};
use futures_util::{SinkExt, StreamExt};
use log::*;
use tokio::net::TcpStream;

pub(crate) async fn accept_connection<T: AccountProvider>(
    director: Arc<DirectorPubsub<T>>,
    chain_socket: BackendWebSocket,
    ephem_socket: BackendWebSocket,
    incoming_stream: TcpStream,
) -> DirectorPubsubResult<()> {
    let addr = incoming_stream.peer_addr()?;
    debug!("Peer address: {}", addr);

    let client_stream =
        tokio_tungstenite::accept_async(incoming_stream).await?;

    let (mut write_client, mut read_client) = client_stream.split();
    let (mut write_chain, mut read_chain) = chain_socket.split();
    let (mut write_ephem, mut read_ephem) = ephem_socket.split();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                // We pipe both chain and ephemeral messages to the client
                next = read_chain.next() => {
                    match next {
                        Some(msg) => {
                            trace!("Chain message: {:?}", msg);
                            write_client.send(msg.unwrap()).await.unwrap();
                        }
                        None => {
                            debug!("Chain stream ended");
                            break;
                        }
                    }
                }
                next = read_ephem.next() => {
                    match next {
                        Some(msg) => {
                            trace!("Ephem message: {:?}", msg);
                            write_client.send(msg.unwrap()).await.unwrap();
                        }
                        None => {
                            debug!("Ephem stream ended");
                            break;
                        }
                    }
                }
                // For client messages we decide by message content if to send it
                // to chain or ephem socket
                next = read_client.next() => {
                    match next {
                        Some(Ok(msg)) => {
                            trace!("Client message: {:?}", msg);
                            use RequestEndpoint::*;
                            match director.guide_msg(&msg).await {
                                Some(Chain) => {
                                    trace!("Sending message to chain: {:?}", msg);
                                    write_chain.send(msg).await.unwrap()
                                },
                                Some(Ephemeral) => {
                                    trace!("Sending message to ephemeral: {:?}", msg);
                                    write_ephem.send(msg).await.unwrap();
                                }
                                Some(Both) => {
                                    trace!("Sending message to chain and ephemeral: {:?}", msg);
                                    write_chain.send(msg.clone()).await.unwrap();
                                    write_ephem.send(msg).await.unwrap();
                                }
                                // If client sends a "close" message we return None as endpoint
                                None => break
                            }
                        }
                        Some(Err(err)) => {
                            error!("Error reading client message: {:?}", err);
                            break;
                        }
                        None => {
                            debug!("Client stream ended");
                            break;
                        }
                    }
                }
            };
        }
    });
    Ok(())
}
