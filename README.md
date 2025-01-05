# foxglove-websocket-rs
Implementation of [foxglove websocket server](https://github.com/foxglove/ws-protocol) for rust.

## Basic Use

1. Open https://app.foxglove.dev/
1. Open a connection to ws://localhost:8765
1. Run the example server:
```
RUST_LOG=debug ./target/debug/examples/echo_server
```
1. When viewing raw messages you should see `productId` incrementing
