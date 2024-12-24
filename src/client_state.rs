use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;
use tokio_tungstenite::WebSocketStream;

use crate::ChannelId;
use crate::ClientChannelId;
use crate::SubscriptionId;

pub struct ClientState {
    // The WebSocket connection for the client.
    pub connection: WebSocketStream<tokio::net::TcpStream>,

    // Maps subscription IDs to channel IDs to keep track of what the client is subscribed to.
    pub subscriptions: HashMap<SubscriptionId, ChannelId>,

    // Reverse mapping of `subscriptions` to quickly look up subscription IDs by channel IDs.
    pub subscriptions_by_channel: HashMap<ChannelId, SubscriptionId>,

    // Maps advertised client channel IDs to client channels to manage channels advertised by the client.
    //pub advertisements_by_channel: HashMap<ClientChannelId, ClientChannel>,

    // A set of parameter names that the client has subscribed to.
    pub subscribed_params: HashSet<String>,

    // Channels to communicate with the client task, e.g., for sending messages.
    pub sender: tokio::sync::mpsc::Sender<Vec<u8>>,
    pub receiver: tokio::sync::mpsc::Sender<Vec<u8>>,
}

impl ClientState {
    // Define methods to add/remove subscriptions, channels, etc.
    // Define methods to send messages to this client
}
//use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
//use serde::{Deserialize, Serialize};
//use serde_json::Value as JsonValue;
//use std::collections::{HashMap, HashSet};
//use tokio::net::TcpStream;
//use tokio_tungstenite::{tungstenite::protocol::Message, WebSocketStream};
//
//// Type aliases for clarity
//type ChannelId = u32;
//type SubscriptionId = u32;
//type ClientChannelId = u32;
//
//// Define the ClientChannel struct based on the protocol spec
//#[derive(Serialize, Deserialize, Debug, Clone)]
//struct ClientChannel {
//    id: ClientChannelId,
//    topic: String,
//    encoding: String,
//    schema_name: String,
//    schema: String,
//    schema_encoding: Option<String>,
//}
//
//// Define the ClientState struct
//struct ClientState {
//    // The WebSocket connection for the client
//    connection: WebSocketStream<TcpStream>,
//
//    // Maps subscription IDs to channel IDs
//    subscriptions: HashMap<SubscriptionId, ChannelId>,
//
//    // Reverse mapping of `subscriptions` to quickly look up subscription IDs by channel ID
//    subscriptions_by_channel: HashMap<ChannelId, SubscriptionId>,
//
//    // Maps advertised client channel IDs to client channels
//    advertisements_by_channel: HashMap<ClientChannelId, ClientChannel>,
//
//    // Set of parameter names that the client has subscribed to
//    subscribed_params: HashSet<String>,
//
//    // Sender and receiver for communicating with other parts of the server
//    sender: UnboundedSender<Message>,
//    receiver: UnboundedReceiver<Message>,
//}
//
//impl ClientState {
//    // Create a new ClientState with the given WebSocket connection and mpsc channels
//    pub fn new(
//        connection: WebSocketStream<TcpStream>,
//        sender: UnboundedSender<Message>,
//        receiver: UnboundedReceiver<Message>,
//    ) -> Self {
//        ClientState {
//            connection,
//            subscriptions: HashMap::new(),
//            subscriptions_by_channel: HashMap::new(),
//            advertisements_by_channel: HashMap::new(),
//            subscribed_params: HashSet::new(),
//            sender,
//            receiver,
//        }
//    }
//
//    // Add a subscription for the client
//    pub fn add_subscription(&mut self, sub_id: SubscriptionId, chan_id: ChannelId) {
//        self.subscriptions.insert(sub_id, chan_id);
//        self.subscriptions_by_channel.insert(chan_id, sub_id);
//    }
//
//    // Remove a subscription for the client
//    pub fn remove_subscription(&mut self, sub_id: SubscriptionId) {
//        if let Some(chan_id) = self.subscriptions.remove(&sub_id) {
//            self.subscriptions_by_channel.remove(&chan_id);
//        }
//    }
//
//    // Add an advertised channel for the client
//    pub fn add_advertised_channel(&mut self, channel: ClientChannel) {
//        self.advertisements_by_channel.insert(channel.id, channel);
//    }
//
//    // Remove an advertised channel for the client
//    pub fn remove_advertised_channel(&mut self, chan_id: ClientChannelId) {
//        self.advertisements_by_channel.remove(&chan_id);
//    }
//
//    // Subscribe to a parameter for the client
//    pub fn subscribe_param(&mut self, param_name: String) {
//        self.subscribed_params.insert(param_name);
//    }
//
//    // Unsubscribe from a parameter for the client
//    pub fn unsubscribe_param(&mut self, param_name: &str) {
//        self.subscribed_params.remove(param_name);
//    }
//
//    // Handle incoming messages from the receiver channel
//    pub async fn handle_messages(&mut self) {
//        while let Some(message) = self.receiver.next().await {
//            // Process the message
//            // For example, deserialize JSON messages and handle them accordingly
//            match message {
//                Message::Text(text) => {
//                    if let Ok(json_msg) = serde_json::from_str::<JsonValue>(&text) {
//                        // Handle the JSON message based on the 'op' value
//                        // ...
//                    } else {
//                        // Handle invalid JSON
//                    }
//                }
//                Message::Binary(bin) => {
//                    // Handle binary messages
//                    // ...
//                }
//                _ => {
//                    // Handle other types of messages or errors
//                }
//            }
//        }
//    }
//
//    // ... Additional methods to interact with the client's state ...
//}
