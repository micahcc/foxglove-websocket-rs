use std::collections::HashMap;

use async_trait::async_trait;

use foxglove_websocket_rs::CallId;
use foxglove_websocket_rs::Capability;
use foxglove_websocket_rs::Channel;
use foxglove_websocket_rs::ChannelId;
use foxglove_websocket_rs::ClientChannelId;
use foxglove_websocket_rs::FoxgloveServer;
use foxglove_websocket_rs::FoxgloveServerListener;
use foxglove_websocket_rs::Parameter;
use foxglove_websocket_rs::RequestId;
use foxglove_websocket_rs::ServerInfo;
use foxglove_websocket_rs::ServiceId;

struct ExampleFoxgloveServerListener {}

#[async_trait]
impl FoxgloveServerListener for ExampleFoxgloveServerListener {
    async fn on_subscribe(&self, server: FoxgloveServer, channel_id: ChannelId) {
        todo!();
    }

    async fn on_unsubscribe(&self, server: FoxgloveServer, channel_id: ChannelId) {
        todo!();
    }

    async fn on_client_advertise(&self, server: FoxgloveServer, channel: Channel) {
        todo!();
    }

    async fn on_client_unadvertise(&self, server: FoxgloveServer, channel_id: ClientChannelId) {
        todo!();
    }

    async fn on_client_message(
        &self,
        server: FoxgloveServer,
        channel_id: ClientChannelId,
        payload: Vec<u8>,
    ) {
        todo!();
    }

    async fn on_service_request(
        &self,
        server: FoxgloveServer,
        service_id: ServiceId,
        call_id: CallId,
        encoding: String,
        payload: Vec<u8>,
    ) -> Vec<u8> {
        todo!();
    }

    async fn on_get_parameters(
        &self,
        server: FoxgloveServer,
        param_names: Vec<String>,
        request_id: Option<RequestId>,
    ) -> Vec<Parameter> {
        todo!();
    }

    async fn on_set_parameters(
        &self,
        server: FoxgloveServer,
        params: Vec<Parameter>,
        request_id: Option<RequestId>,
    ) -> Vec<Parameter> {
        todo!();
    }

    async fn on_parameters_subscribe(
        &self,
        server: FoxgloveServer,
        param_name: Vec<String>,
        subscribe: bool,
    ) {
        todo!();
    }
}

// Example of a simple WebSocket server using warp and tokio_tungstenite
#[tokio::main]
async fn main() {
    let listener = Box::new(ExampleFoxgloveServerListener {});
    let server = FoxgloveServer::new_with_listener("hello".to_string(), listener);
    server
        .start("127.0.0.1", 8323)
        .await
        .expect("Failed to start");
}

// Define other structs and enums here
