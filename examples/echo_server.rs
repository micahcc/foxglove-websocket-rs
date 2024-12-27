use async_trait::async_trait;

use foxglove_websocket_rs::CallId;
use foxglove_websocket_rs::Channel;
use foxglove_websocket_rs::ChannelId;
use foxglove_websocket_rs::ClientChannelId;
use foxglove_websocket_rs::FoxgloveServer;
use foxglove_websocket_rs::FoxgloveServerListener;
use foxglove_websocket_rs::Parameter;
use foxglove_websocket_rs::RequestId;
use foxglove_websocket_rs::ServiceId;

struct ExampleFoxgloveServerListener {}

#[async_trait]
impl FoxgloveServerListener for ExampleFoxgloveServerListener {
    async fn on_subscribe(&self, _server: FoxgloveServer, _channel_id: ChannelId) {}

    async fn on_unsubscribe(&self, _server: FoxgloveServer, _channel_id: ChannelId) {}

    async fn on_client_advertise(&self, _server: FoxgloveServer, _channel: Channel) {}

    async fn on_client_unadvertise(&self, _server: FoxgloveServer, _channel_id: ClientChannelId) {}

    async fn on_client_message(
        &self,
        _server: FoxgloveServer,
        _channel_id: ClientChannelId,
        _payload: Vec<u8>,
    ) {
        todo!();
    }

    async fn on_service_request(
        &self,
        _server: FoxgloveServer,
        _service_id: ServiceId,
        _call_id: CallId,
        _encoding: String,
        _payload: Vec<u8>,
    ) -> Vec<u8> {
        todo!();
    }

    async fn on_get_parameters(
        &self,
        _server: FoxgloveServer,
        _param_names: Vec<String>,
        _request_id: Option<RequestId>,
    ) -> Vec<Parameter> {
        todo!();
    }

    async fn on_set_parameters(
        &self,
        _server: FoxgloveServer,
        _params: Vec<Parameter>,
        _request_id: Option<RequestId>,
    ) -> Vec<Parameter> {
        todo!();
    }

    async fn on_parameters_subscribe(
        &self,
        _server: FoxgloveServer,
        _param_name: Vec<String>,
        _subscribe: bool,
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
