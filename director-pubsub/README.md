
# Summary

Start a TCP PUBSUB service on 127.0.0.1:9900.
Allows for dynamically sending requests to the validator or the L1.
Requests are routed transparently based on their content.

# Details

Uses websocket implemented using tokio_tungstenite.

Any request from the client is conditionally routed to either:
 - the websocket of the "chain" (L1 Solana)
 - the websocket of the "ephem" (Validator)
 - Sometimes both

This routing is done using some "guide" logic implemented in this crate.

Any request from "chain" or "ephem" is sent back to the client

Exploration notes for file:

 - accept_connection.rs: event req/res loop between client/chain/ephem
 - director.rs: struct that 

# Notes

Important dependencies:

 - ??: [providers](../providers/README.md)
