
# Summary

Start a TCP PUBSUB service on 127.0.0.1:9900.
Dynamically route requests either to the validator or the L1.
Requests are routed transparently based on their content and context.

# Details

Uses websocket implemented using tokio_tungstenite.

Any request from the client is conditionally routed to either:
- the websocket of the "chain" (L1 Solana)
- the websocket of the "ephem" (Validator)
- Sometimes both

This routing is done using some "guide" logic implemented in this crate.

Any request from "chain" or "ephem" is sent back to the client

*Important symbols:*

- `accept_connection` function
  - Basically the main loop function for the service (using tokio)
  - Takes in parameter `DirectorPubsub` and tcps/websockets
  - Read from all streams and write to appropriate stream for each messages
  - Uses the `DirectorPubsub` for routing requests and simple forward for responses

- `DirectorPubsub` struct
  - contains a `GuideStrategyResolver`
  - can convert `Message` -> `GuideStrategy` -> `RequestEndpoint`
  - using `guide_strategy_from_pubsub_msg`

- `ParsedClientMessage` enum
  - Parsed representation of a raw websocket message
  - Can be parsed from a raw message string
  - Uses serde to read the JSON

- `guide_strategy_from_pubsub_msg` function
  - Takes in parameter a message, parses it to a `ParsedClientMessage`
  - Compute the expected `GuideStrategy` based off of the message content


# Notes

*Important dependencies:*

- Provides `GuideStrategyResolver`: [guidepoint](../guidepoint/README.md)
- Provides `GuideStrategy` and `RequestEndpoint`: [core](../core/README.md) 
