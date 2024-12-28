use std::collections::HashMap;
use std::time::Duration;
use std::time::SystemTime;

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

struct ExampleFoxgloveServerListener {
    parameters: HashMap<String, Parameter>,
}

#[async_trait]
impl FoxgloveServerListener for ExampleFoxgloveServerListener {
    async fn on_subscribe(&mut self, _server: FoxgloveServer, _channel_id: ChannelId) {}

    async fn on_unsubscribe(&mut self, _server: FoxgloveServer, _channel_id: ChannelId) {}

    async fn on_client_advertise(&mut self, _server: FoxgloveServer, _channel: Channel) {}

    async fn on_client_unadvertise(
        &mut self,
        _server: FoxgloveServer,
        _channel_id: ClientChannelId,
    ) {
    }

    async fn on_client_message(
        &mut self,
        _server: FoxgloveServer,
        _channel_id: ClientChannelId,
        _payload: Vec<u8>,
    ) {
    }

    async fn on_service_request(
        &mut self,
        _server: FoxgloveServer,
        _service_id: ServiceId,
        _call_id: CallId,
        _encoding: String,
        _payload: Vec<u8>,
    ) -> Vec<u8> {
        return vec![];
    }

    async fn on_get_parameters(
        &mut self,
        _server: FoxgloveServer,
        param_names: Vec<String>,
        _request_id: Option<RequestId>,
    ) -> Vec<Parameter> {
        let mut out = vec![];
        for n in param_names {
            if let Some(p) = self.parameters.get(&n) {
                out.push(p.clone());
            }
        }
        return out;
    }

    async fn on_set_parameters(
        &mut self,
        _server: FoxgloveServer,
        params: Vec<Parameter>,
        _request_id: Option<RequestId>,
    ) -> Vec<Parameter> {
        for p in params {
            self.parameters.insert(p.name.clone(), p);
        }
        return self.parameters.values().map(|p| p.clone()).collect();
    }

    async fn on_parameters_subscribe(
        &mut self,
        _server: FoxgloveServer,
        _param_name: Vec<String>,
        _subscribe: bool,
    ) {
    }
}

// Example of a simple WebSocket server using warp and tokio_tungstenite
#[tokio::main]
async fn main() {
    env_logger::init();

    let listener = Box::new(ExampleFoxgloveServerListener {
        parameters: Default::default(),
    });
    let server = FoxgloveServer::new_with_listener("hello".to_string(), listener);
    server
        .start("127.0.0.1", 8765)
        .await
        .expect("Failed to start");

    let cid = server
        .add_channel(Channel {
            id: 0,
            topic: "/hello".to_string(),
            encoding: "json".to_string(),
            schema_name: "json".to_string(),
            schema: "json".to_string(),
            schema_encoding: None,
        })
        .await;
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let dt = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_nanos();
        server
            .send_message(cid, dt as u64, b"{\"hello\":\"world\"}".to_vec())
            .await;
    }
}

// Define other structs and enums here
