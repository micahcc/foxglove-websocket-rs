use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

//use futures::stream::SplitSink;
//use futures::{SinkExt, StreamExt};
//use tokio::net::TcpListener;
//use tokio::sync::mpsc;
//use tokio_stream::wrappers::TcpListenerStream;
//use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
//use warp::Filter;
//
//type ChannelId = u32;
//type SubscriptionId = u32;
//type ServiceId = u32;
//type ClientChannelId = u32;

use crate::messages::AdvertiseServices;
use crate::Advertise;
use crate::Channel;
use crate::ChannelId;
use crate::ClientChannelId;
use crate::ClientState;
use crate::FoxgloveServerListener;
use crate::Parameter;
use crate::ParameterValues;
use crate::ServerInfo;
use crate::Service;
use crate::ServiceId;
use crate::SubscriptionId;
use crate::Unadvertise;
use crate::UnadvertiseServices;

pub struct FoxgloveServer {
    server_info: ServerInfo,

    clients: Arc<Mutex<HashMap<ChannelId, ClientState>>>,

    next_channel_id: Arc<AtomicU32>,
    channels: Arc<Mutex<HashMap<ChannelId, Channel>>>,

    next_service_id: Arc<AtomicU32>,
    services: Arc<Mutex<HashMap<ServiceId, Service>>>,

    listener: Box<dyn FoxgloveServerListener>,
}

async fn handle_connection(stream: tokio::net::TcpStream) {
    if let Ok(ws_stream) = tokio_tungstenite::accept_async(stream).await {
        // Use ws_stream to send and receive WebSocket messages
    }
}

impl FoxgloveServer {
    pub fn new(listener: Box<dyn FoxgloveServerListener>) -> Self {
        let server_info = ServerInfo {
            op: "serverInfo".to_string(),
            session_id: None,
            name: listener.name(),
            capabilities: listener.capabilities(),
            supported_encodings: listener.supported_encodings(),
            metadata: listener.metadata(),
        };

        FoxgloveServer {
            server_info,
            next_channel_id: Arc::new(AtomicU32::new(1)),
            next_service_id: Arc::new(AtomicU32::new(1)),
            clients: Arc::new(Mutex::new(HashMap::new())),
            channels: Arc::new(Mutex::new(HashMap::new())),
            services: Arc::new(Mutex::new(HashMap::new())),
            listener,
        }
    }

    pub async fn start(&self, host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        // Create a TCP listener on the specified host and port
        let addr = format!("{}:{}", host, port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        log::info!("Server listening on: {}", addr);

        // Accept incoming TCP connections and upgrade them to WebSocket
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(handle_connection(stream));
        }

        return Ok(());
    }

    // Assuming you have an async function to send JSON messages to all connected clients
    async fn broadcast(&self, message: Vec<u8>) {
        let clients = self.clients.lock().expect("lock");
        for client in clients.values() {
            let _ = client.sender.send(message.clone());
        }
    }

    pub async fn add_channel(&self, mut channel: Channel) -> ChannelId {
        let mut channels = self.channels.lock().expect("lock");
        let new_id = self.next_channel_id.fetch_add(1, Ordering::Relaxed);
        channel.id = new_id;

        channels.insert(new_id, channel.clone());

        let msg = Advertise {
            op: "advertise".to_string(),
            channels: vec![channel],
        };

        // Broadcast the new channel to all connected clients
        let _ = self
            .broadcast(serde_json::to_vec(&msg).expect("serializing advertise"))
            .await;

        return new_id;
    }

    pub async fn remove_channel(&self, channel_id: ChannelId) -> Result<(), String> {
        let mut channels = self.channels.lock().expect("lock");

        // Check if the channel exists before attempting to remove it
        if channels.remove(&channel_id).is_none() {
            return Err(format!("Channel with ID {} does not exist.", channel_id));
        }

        // Broadcast the new channel to all connected clients
        let msg = Unadvertise {
            op: "unadvertise".to_string(),
            channel_ids: vec![channel_id],
        };

        let _ = self
            .broadcast(serde_json::to_vec(&msg).expect("serializing unadvertise"))
            .await;
        return Ok(());
    }

    pub async fn add_service(&self, mut service: Service) -> ServiceId {
        let new_id = self.next_channel_id.fetch_add(1, Ordering::Relaxed);
        service.id = new_id;
        let mut services = self.services.lock().expect("lock");

        services.insert(new_id, service.clone());

        let msg = AdvertiseServices {
            op: "advertiseServices".to_string(),
            services: vec![service],
        };

        // Broadcast the new channel to all connected clients
        let _ = self
            .broadcast(serde_json::to_vec(&msg).expect("serializing advertise services"))
            .await;
        return new_id;
    }

    pub async fn remove_service(&self, service_id: ServiceId) -> Result<(), String> {
        let mut services = self.services.lock().expect("lock");

        // Check if the channel exists before attempting to remove it
        if services.remove(&service_id).is_none() {
            return Err(format!("Service with ID {} does not exist.", service_id));
        }

        // Broadcast the new channel to all connected clients
        let msg = UnadvertiseServices {
            op: "unadvertiseServices".to_string(),
            service_ids: vec![service_id],
        };

        let _ = self
            .broadcast(serde_json::to_vec(&msg).expect("serializing unadvertise"))
            .await;
        return Ok(());
    }

    pub async fn update_parameters(&self, parameters: Vec<Parameter>) {
        let clients = self.clients.lock().expect("lock");
        for client in clients.values() {
            let to_send: Vec<Parameter> = parameters
                .iter()
                .filter(|p| client.subscribed_params.contains(&p.name))
                .map(|p| p.clone())
                .collect();
            if !to_send.is_empty() {
                let msg = ParameterValues {
                    op: "parameterValues".to_string(),
                    parameters: to_send,
                };
                let msg = serde_json::to_vec(&msg).expect("serializing params");
                let _ = client.sender.send(msg);
            }
        }
    }

    pub async fn send_message(&self, chan_id: ChannelId, timestamp_nanos: u64, payload: Vec<u8>) {
        let clients = self.clients.lock().expect("lock");
        for client in clients.values() {
            if let Some(&sub_id) = client.subscriptions_by_channel.get(&chan_id) {
                let sub_id = sub_id as u32;
                let timestamp_nanos = timestamp_nanos as u64;
                let mut message = Vec::new();
                message.push(1u8); // opcode 0x1
                message.extend_from_slice(&sub_id.to_le_bytes()); // Subscription ID
                message.extend_from_slice(&timestamp_nanos.to_le_bytes()); // Timestamp
                message.extend_from_slice(&payload); // Payload
                let _ = client.sender.send(message);
            }
        }
    }

    /// Reset session Id and send new server info to clients.
    pub async fn reset_session_id(&mut self, new_session_id: Option<String>) {
        self.server_info.session_id = new_session_id;

        let clients = self.clients.lock().expect("lock");
        for client in clients.values() {
            let _ = client
                .sender
                .send(serde_json::to_vec(&self.server_info).expect("encoding server_info"));
        }
    }

    // Define the methods for adding/removing channels, services, etc.
    // Define methods to send messages to clients
}
