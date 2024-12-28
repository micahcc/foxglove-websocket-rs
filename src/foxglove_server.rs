use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use futures_util::SinkExt;
use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::Message;

use crate::messages;
use crate::Advertise;
use crate::Channel;
use crate::ChannelId;
use crate::ClientState;
use crate::FoxgloveServerListener;
use crate::FoxgloveState;
use crate::Parameter;
use crate::ParameterValues;
use crate::ServerInfo;
use crate::Service;
use crate::ServiceId;
use crate::SubscriptionId;
use crate::Unadvertise;
use crate::UnadvertiseServices;

#[derive(Clone)]
pub struct FoxgloveServer {
    state: Arc<Mutex<FoxgloveState>>,
    listener: Option<Arc<tokio::sync::Mutex<Box<dyn FoxgloveServerListener + Send>>>>,
}

fn make_status(level: messages::StatusLevel, message: String) -> Message {
    let s = messages::Status {
        op: "status".to_string(),
        level: level as u8,
        message,
        id: None,
    };

    return Message::Text(serde_json::to_string(&s).expect("Failed to serialize status"));
}

async fn handle_subscribe(
    client_id: u32,
    state: Arc<Mutex<FoxgloveState>>,
    listener: Option<Arc<tokio::sync::Mutex<Box<dyn FoxgloveServerListener + Send>>>>,
    msg: messages::Subscribe,
) {
    let events = {
        let mut state = state.lock().expect("lock");

        let mut events = vec![];
        for sub in msg.subscriptions {
            let chan_id = sub.channel_id;
            let sub_id = sub.id;

            let client = state
                .clients
                .get(&client_id)
                .expect("must be a client because we own the object");
            if client.subscriptions.contains_key(&sub_id) {
                let _ = client.sender.send(make_status(
                    messages::StatusLevel::Error,
                    format!(
                        "Client subscription id {sub_id} was already used; ignoring subscription"
                    ),
                ));
                continue;
            }

            if !state.channels.contains_key(&chan_id) {
                let _ = client.sender.send(make_status(
                    messages::StatusLevel::Warning,
                    format!("Channel {chan_id} is not available; ignoring subscription"),
                ));
                continue;
            };

            if client.subscriptions_by_channel.contains_key(&chan_id) {
                let _ = client.sender.send(make_status(
                    messages::StatusLevel::Warning,
                    format!(
                        "Client is already subscribed to channel {chan_id}; ignoring subscription"
                    ),
                ));
                continue;
            }

            let prev_sub_count: usize = state
                .clients
                .values()
                .flat_map(|c| {
                    c.subscriptions_by_channel
                        .keys()
                        .map(|cid| (*cid == chan_id) as usize)
                })
                .sum();

            if prev_sub_count == 0 {
                events.push(chan_id);
            }

            let client = state
                .clients
                .get_mut(&client_id)
                .expect("must be a client because we own the object");

            let _ = client.subscriptions.insert(sub_id, chan_id);
            let _ = client.subscriptions_by_channel.insert(chan_id, sub_id);
        }
        events
    };

    // hold listener lock separately from state lock
    let interface = FoxgloveServer::new_from_state(state);
    if let Some(listener) = listener {
        let mut listener = listener.lock().await;
        for chan_id in events {
            listener.on_subscribe(interface.clone(), chan_id).await;
        }
    }
}

async fn handle_unsubscribe(
    client_id: u32,
    state: Arc<Mutex<FoxgloveState>>,
    listener: Option<Arc<tokio::sync::Mutex<Box<dyn FoxgloveServerListener + Send>>>>,
    msg: messages::Unsubscribe,
) {
    let events = {
        let mut removed_chan_ids = vec![];
        let mut state = state.lock().expect("lock");
        for sub_id in msg.subscription_ids {
            let client = state
                .clients
                .get_mut(&client_id)
                .expect("must be a client because we own the object");

            match client.subscriptions.remove(&sub_id) {
                None => {
                    let _ = client.sender.send(make_status(
                        messages::StatusLevel::Warning,
                        format!(
                        "Client subscription id {sub_id} did not exist; ignoring unsubscription"
                    ),
                    ));
                    continue;
                }
                Some(chan_id) => {
                    log::debug!(
                        "Client {} unsubscribed from channel {}",
                        client.remote,
                        chan_id
                    );

                    removed_chan_ids.push(chan_id);
                }
            }
        }

        let mut events = vec![];
        for chan_id in removed_chan_ids {
            let sub_count: usize = state
                .clients
                .values()
                .flat_map(|c| {
                    c.subscriptions_by_channel
                        .keys()
                        .map(|cid| (*cid == chan_id) as usize)
                })
                .sum();
            if sub_count == 0 {
                events.push(chan_id);
            }
        }

        events
    };

    // hold listener lock separately from state lock
    let interface = FoxgloveServer::new_from_state(state);
    if let Some(listener) = listener {
        let mut listener = listener.lock().await;
        for chan_id in events {
            listener.on_unsubscribe(interface.clone(), chan_id).await;
        }
    }
}

async fn handle_advertise(
    client_id: u32,
    state: Arc<Mutex<FoxgloveState>>,
    listener: Option<Arc<tokio::sync::Mutex<Box<dyn FoxgloveServerListener + Send>>>>,
    message: messages::ClientAdvertise,
) {
    let events = {
        let mut state = state.lock().expect("lock");
        let client = state
            .clients
            .get_mut(&client_id)
            .expect("must be a client because we own the object");
        let mut events = vec![];
        for channel in message.channels {
            if client.advertisements_by_channel.contains_key(&channel.id) {
                log::error!("Failed to add client channel {}", channel.id);
                let _ = client.sender.send(make_status(
                    messages::StatusLevel::Warning,
                    format!("Failed to add client channel {}", channel.id),
                ));
                continue;
            }

            let _ = client
                .advertisements_by_channel
                .insert(channel.id, channel.clone());

            log::debug!(
                "Client {} advertised channel {} ({})",
                client.remote,
                channel.id,
                channel.topic
            );

            events.push(channel);
        }
        events
    };

    // hold listener lock separately from state lock
    let interface = FoxgloveServer::new_from_state(state);
    if let Some(listener) = listener {
        let mut listener = listener.lock().await;
        for c in events {
            listener.on_client_advertise(interface.clone(), c).await;
        }
    }
}

async fn handle_unadvertise(
    client_id: u32,
    state: Arc<Mutex<FoxgloveState>>,
    listener: Option<Arc<tokio::sync::Mutex<Box<dyn FoxgloveServerListener + Send>>>>,
    msg: messages::ClientUnadvertise,
) {
    let events = {
        let mut state = state.lock().expect("lock");
        let client = state
            .clients
            .get_mut(&client_id)
            .expect("must be a client because we own the object");
        let mut events = vec![];

        for channel_id in msg.channel_ids {
            if !client.advertisements_by_channel.contains_key(&channel_id) {
                log::error!("Failed to remove client channel {channel_id}");
                let _ = client.sender.send(make_status(
                    messages::StatusLevel::Warning,
                    format!("Failed to remove client channel {channel_id}"),
                ));
                continue;
            }

            let _ = client.advertisements_by_channel.remove(&channel_id);

            log::debug!(
                "Client {} unadvertised channel {}",
                client.remote,
                channel_id,
            );

            events.push(channel_id);
        }
        events
    };

    // hold listener lock separately from state lock
    let interface = FoxgloveServer::new_from_state(state);
    if let Some(listener) = listener {
        let mut listener = listener.lock().await;
        for cid in events {
            listener.on_client_unadvertise(interface.clone(), cid).await;
        }
    }
}

async fn handle_get_parameters(
    client_id: u32,
    state: Arc<Mutex<FoxgloveState>>,
    listener: Option<Arc<tokio::sync::Mutex<Box<dyn FoxgloveServerListener + Send>>>>,
    msg: messages::GetParameters,
) {
    let request_id = msg.id.clone();
    let interface = FoxgloveServer::new_from_state(state.clone());
    let params = {
        if let Some(listener) = listener {
            let mut listener = listener.lock().await;
            listener
                .on_get_parameters(interface, msg.parameter_names, msg.id)
                .await
        } else {
            vec![]
        }
    };

    {
        let out = messages::ParameterValues {
            op: "ParameterValues".to_string(),
            parameters: params,
            id: request_id,
        };
        let out = Message::Text(
            serde_json::to_string(&out).expect("Failed to serialize ParameterValues"),
        );

        let mut state = state.lock().expect("lock");
        let client = state
            .clients
            .get_mut(&client_id)
            .expect("must be a client because we own the object");
        let _ = client.sender.send(out);
    }
}

async fn handle_set_parameters(
    client_id: u32,
    state: Arc<Mutex<FoxgloveState>>,
    listener: Option<Arc<tokio::sync::Mutex<Box<dyn FoxgloveServerListener + Send>>>>,
    msg: messages::SetParameters,
) {
    let request_id = msg.id.clone();
    let interface = FoxgloveServer::new_from_state(state.clone());
    let params = {
        if let Some(listener) = listener {
            let mut listener = listener.lock().await;
            listener
                .on_set_parameters(interface, msg.parameters, msg.id)
                .await
        } else {
            vec![]
        }
    };

    {
        let out = messages::ParameterValues {
            op: "ParameterValues".to_string(),
            parameters: params,
            id: request_id,
        };
        let out = Message::Text(
            serde_json::to_string(&out).expect("Failed to serialize ParameterValues"),
        );

        let mut state = state.lock().expect("lock");
        let client = state
            .clients
            .get_mut(&client_id)
            .expect("must be a client because we own the object");
        let _ = client.sender.send(out);
    }
}

async fn handle_subscribe_parameter_updates(
    client_id: u32,
    state: Arc<Mutex<FoxgloveState>>,
    listener: Option<Arc<tokio::sync::Mutex<Box<dyn FoxgloveServerListener + Send>>>>,
    msg: messages::SubscribeParameterUpdate,
) {
    let new_param_subscriptions = {
        let mut state = state.lock().expect("lock");
        let client = state
            .clients
            .get_mut(&client_id)
            .expect("must be a client because we own the object");
        let mut added = vec![];
        for p in &msg.parameter_names {
            if client.subscribed_params.insert(p.to_owned()) {
                added.push(p.to_owned());
            }
        }
        added
    };

    // hold listener lock separately from state lock
    let interface = FoxgloveServer::new_from_state(state.clone());
    if let Some(listener) = listener {
        let mut listener = listener.lock().await;
        listener
            .on_parameters_subscribe(interface, new_param_subscriptions, true)
            .await;
    }
}

async fn handle_unsubscribe_parameter_updates(
    client_id: u32,
    state: Arc<Mutex<FoxgloveState>>,
    listener: Option<Arc<tokio::sync::Mutex<Box<dyn FoxgloveServerListener + Send>>>>,
    msg: messages::UnsubscribeParameterUpdate,
) {
    let removed = {
        let mut state = state.lock().expect("lock");
        let client = state
            .clients
            .get_mut(&client_id)
            .expect("must be a client because we own the object");
        let mut removed = vec![];
        for p in &msg.parameter_names {
            if client.subscribed_params.remove(p) {
                removed.push(p.to_string());
            }
        }
        removed
    };

    // hold listener lock separately from state lock
    let interface = FoxgloveServer::new_from_state(state.clone());
    if let Some(listener) = listener {
        let mut listener = listener.lock().await;
        listener
            .on_parameters_subscribe(interface, removed, false)
            .await;
    }
}

async fn handle_text_input(
    client_id: u32,
    state: Arc<Mutex<FoxgloveState>>,
    listener: Option<Arc<tokio::sync::Mutex<Box<dyn FoxgloveServerListener + Send>>>>,
    msg: String,
) {
    let unparsed = match serde_json::from_str(&msg) {
        Err(err) => {
            log::error!("Received unparsable json: {err}");
            return;
        }
        Ok(serde_json::Value::Object(m)) => m,
        Ok(m) => {
            log::error!("Received non-object json: {m}");
            return;
        }
    };

    let op = match unparsed.get("op") {
        Some(serde_json::Value::String(op)) => op,
        _ => {
            log::error!("No op received");
            return;
        }
    };

    match op.as_str() {
        "subscribe" => match serde_json::from_str(&msg) {
            Ok(parsed) => {
                handle_subscribe(client_id, state, listener, parsed).await;
            }
            Err(err) => {
                log::error!("Failed to parse subscription request, err: {err}");
            }
        },
        "unsubscribe" => match serde_json::from_str(&msg) {
            Ok(parsed) => {
                handle_unsubscribe(client_id, state, listener, parsed).await;
            }
            Err(err) => {
                log::error!("Failed to parse unsubscribe request, err: {err}");
            }
        },
        "advertise" => match serde_json::from_str(&msg) {
            Ok(parsed) => {
                handle_advertise(client_id, state, listener, parsed).await;
            }
            Err(err) => {
                log::error!("Failed to parse advertise request, err: {err}");
            }
        },
        "unadvertise" => match serde_json::from_str(&msg) {
            Ok(parsed) => {
                handle_unadvertise(client_id, state, listener, parsed).await;
            }
            Err(err) => {
                log::error!("Failed to parse unadvertise request, err: {err}");
            }
        },
        "getParameters" => match serde_json::from_str(&msg) {
            Ok(parsed) => {
                handle_get_parameters(client_id, state, listener, parsed).await;
            }
            Err(err) => {
                log::error!("Failed to parse getParameters request, err: {err}");
            }
        },
        "setParameters" => match serde_json::from_str(&msg) {
            Ok(parsed) => {
                handle_set_parameters(client_id, state, listener, parsed).await;
            }
            Err(err) => {
                log::error!("Failed to parse getParameters request, err: {err}");
            }
        },
        "subscribeParameterUpdates" => match serde_json::from_str(&msg) {
            Ok(parsed) => {
                handle_subscribe_parameter_updates(client_id, state, listener, parsed).await;
            }
            Err(err) => {
                log::error!("Failed to parse getParameters request, err: {err}");
            }
        },
        "unsubscribeParameterUpdates" => match serde_json::from_str(&msg) {
            Ok(parsed) => {
                handle_unsubscribe_parameter_updates(client_id, state, listener, parsed).await;
            }
            Err(err) => {
                log::error!("Failed to parse getParameters request, err: {err}");
            }
        },
        unknown => {
            log::error!("unhandled op: {unknown}");
        }
    }
}

async fn handle_binary_input(
    client_id: u32,
    state: Arc<Mutex<FoxgloveState>>,
    listener: Option<Arc<tokio::sync::Mutex<Box<dyn FoxgloveServerListener + Send>>>>,
    msg: Vec<u8>,
) {
    if msg.len() < 5 {
        let mut state = state.lock().expect("lock");
        let client = state
            .clients
            .get_mut(&client_id)
            .expect("must be a client because we own the object");
        log::error!("Received invalid binary message of size {}", msg.len());
        let _ = client.sender.send(make_status(
            messages::StatusLevel::Error,
            format!("Received invalid binary message of size {}", msg.len()),
        ));
        return;
    }

    if msg[0] == messages::ClientBinaryOpcode::MessageData as u8 {
        // Bytes            Type        Description
        // 1                opcode      0x01
        // 4                uint32      channel id
        // remaining bytes  uint8[]     message payload

        let channel_id = u32::from_le_bytes((&msg[1..5]).try_into().expect("4 bytes"));
        let payload = &msg[5..];

        {
            let mut state = state.lock().expect("lock");
            let client = state
                .clients
                .get_mut(&client_id)
                .expect("must be a client because we own the object");
            if !client.advertisements_by_channel.contains_key(&channel_id) {
                log::error!(
                    "Channel {channel_id} not registered by client {}",
                    client.remote
                );
                let _ = client.sender.send(make_status(
                    messages::StatusLevel::Error,
                    format!(
                        "Channel {channel_id} not registered by client {}",
                        client.remote
                    ),
                ));
                return;
            }
        }

        // make sure this lock doens't overlap with state lock
        let interface = FoxgloveServer::new_from_state(state.clone());

        if let Some(listener) = listener {
            let mut listener = listener.lock().await;
            listener
                .on_client_message(interface, channel_id, payload.to_vec())
                .await;
        }
    } else if msg[0] == messages::ClientBinaryOpcode::ServiceCallRequest as u8 {
        let (sender, service_id, call_id, encoding, payload) = {
            let mut state = state.lock().expect("lock");
            let client = state
                .clients
                .get_mut(&client_id)
                .expect("must be a client because we own the object");
            let sender = client.sender.clone();

            if msg.len() < 13 {
                log::error!("Not enough bytes in message: expect 13 got: {}", msg.len());
                let _ = client.sender.send(make_status(
                    messages::StatusLevel::Error,
                    format!("Not enough bytes in message: expect 13 got: {}", msg.len()),
                ));
                return;
            }

            // Service Call Request
            // Request to call a service that has been advertised by the server.
            // Only supported if the server previously declared the services capability.
            // Bytes        Type        Description
            // 1                    opcode      0x02
            // 4                    uint32      service id
            // 4                    uint32      call id, a unique number to identify the corresponding service response
            // 4                    uint32      encoding length
            // encoding length      char[]      encoding, one of the encodings supported by the server
            // remaining bytes      uint8[]     request payload
            let service_id = u32::from_le_bytes((&msg[1..5]).try_into().expect("4 bytes"));
            let call_id = u32::from_le_bytes((&msg[5..9]).try_into().expect("4 bytes"));
            let encoding_length = u32::from_le_bytes((&msg[9..13]).try_into().expect("4 bytes"));
            let encoding_start: usize = 13;
            let encoding_end: usize = encoding_start + encoding_length as usize;

            let encoding = if encoding_length == 0 {
                "json".to_string()
            } else if encoding_length as usize + 13 < msg.len() {
                log::error!(
                    "Not enough bytes in message: expect {} got: {}",
                    13 + encoding_length,
                    msg.len()
                );
                let _ = client.sender.send(make_status(
                    messages::StatusLevel::Error,
                    format!(
                        "Not enough bytes in message: expect {} got: {}",
                        13 + encoding_length,
                        msg.len()
                    ),
                ));
                return;
            } else {
                String::from_utf8_lossy(&msg[encoding_start..encoding_end]).into_owned()
            };
            let payload = &msg[encoding_end..];

            if !state.services.contains_key(&service_id) {
                log::error!("Unknown service {service_id}");
                let _ = sender.send(make_status(
                    messages::StatusLevel::Error,
                    format!("Unknown service {service_id}"),
                ));
                return;
            }
            (sender, service_id, call_id, encoding, payload.to_vec())
        };

        // make sure this lock doens't overlap with state lock

        // pack back into binary
        let mut response_bytes = vec![messages::BinaryOpcode::ServiceCallResponse as u8];
        response_bytes.extend_from_slice(&service_id.to_le_bytes());
        response_bytes.extend_from_slice(&call_id.to_le_bytes());

        let encoding_bytes = encoding.as_bytes();
        response_bytes.extend_from_slice(&(encoding_bytes.len() as u32).to_le_bytes());
        response_bytes.extend_from_slice(&encoding_bytes);

        let interface = FoxgloveServer::new_from_state(state.clone());
        if let Some(listener) = listener {
            let mut listener = listener.lock().await;
            let resp = listener
                .on_service_request(interface, service_id, call_id, encoding, payload)
                .await;

            // finally append response
            response_bytes.extend_from_slice(&resp);
            let _ = sender.send(Message::Binary(response_bytes));
        } else {
            log::error!("No implementation of {service_id}");
            let _ = sender.send(make_status(
                messages::StatusLevel::Error,
                format!("No implementation of {service_id}"),
            ));
        }
    } else {
        let mut state = state.lock().expect("lock");
        let client = state
            .clients
            .get_mut(&client_id)
            .expect("must be a client because we own the object");
        log::error!(
            "Received binary message with invalid operation {:?}",
            msg[0]
        );
        let _ = client.sender.send(make_status(
            messages::StatusLevel::Error,
            format!(
                "Received binary message with invalid operation {:?}",
                msg[0]
            ),
        ));
    }
}

struct ConnectionHandler {}

impl tokio_tungstenite::tungstenite::handshake::server::Callback for ConnectionHandler {
    fn on_request(
        self,
        request: &tokio_tungstenite::tungstenite::handshake::server::Request,
        response: tokio_tungstenite::tungstenite::handshake::server::Response,
    ) -> Result<
        tokio_tungstenite::tungstenite::handshake::server::Response,
        tokio_tungstenite::tungstenite::handshake::server::ErrorResponse,
    > {
        log::info!("Got Req: {request:?}, Res: {response:?}");
        return Ok(response);
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    listener: Option<Arc<tokio::sync::Mutex<Box<dyn FoxgloveServerListener + Send>>>>,
    state: Arc<Mutex<FoxgloveState>>,
) {
    log::info!("handle_connection");
    let remote = match stream.peer_addr() {
        Ok(a) => format!("{a}"),
        Err(_) => format!("unknown"),
    };

    let handler = ConnectionHandler {};
    let connection = match tokio_tungstenite::accept_hdr_async(stream, handler).await {
        Err(err) => {
            log::error!("Failed to upgrade to websocket, reason: {err}");
            return;
        }
        Ok(stream) => stream,
    };

    let (to_client_tx, mut to_client_rx) = tokio::sync::mpsc::unbounded_channel();
    let (mut conn_out, mut conn_in) = connection.split();
    let client = ClientState {
        remote,
        subscribed_params: HashSet::new(),
        subscriptions: HashMap::new(),
        subscriptions_by_channel: HashMap::new(),
        sender: to_client_tx,
        advertisements_by_channel: HashMap::new(),
    };

    let client_id = {
        let mut state = state.lock().expect("lock");

        // service_info
        let _ = client.sender.send(Message::Text(
            serde_json::to_string(&state.server_info).expect("encoding server_info"),
        ));

        let client_id = state.next_client_id;
        state.next_client_id += 1;

        let services_msg = messages::AdvertiseServices {
            op: "advertiseServices".to_string(),
            services: state.services.values().map(|c| c.clone()).collect(),
        };
        let _ = client.sender.send(Message::Text(
            serde_json::to_string(&services_msg).expect("encoding services"),
        ));

        let channels_msg = Advertise {
            op: "advertise".to_string(),
            channels: state.channels.values().map(|c| c.clone()).collect(),
        };
        let _ = client.sender.send(Message::Text(
            serde_json::to_string(&channels_msg).expect("encoding advertisement"),
        ));

        state.clients.insert(client_id, client);
        client_id
    };

    loop {
        tokio::select! {
            from_client = conn_in.next() => {
                if let Some(status) = from_client {
                    match status {
                        Ok(Message::Ping(_)) => {
                            let _ = conn_out.send(Message::Pong(Default::default())).await;
                        }
                        Ok(Message::Pong(_)) => { }
                        Ok(Message::Close(_)) => {
                            log::info!("Connection closed");
                            break;
                        }
                        Ok(Message::Binary(bin)) => handle_binary_input(
                            client_id, state.clone(), listener.clone(), bin
                        ).await,
                        Ok(Message::Text(txt)) => handle_text_input(
                            client_id, state.clone(), listener.clone(), txt
                        ).await,
                        Err(err) => {
                            log::info!("Connection broken: {err}");
                            break;
                        }
                    }
                }
            }
            to_client = to_client_rx.recv() => {
                if let Some(msg) = to_client {
                    let _ = conn_out.send(msg).await;
                }
            }
        }
    }

    // send unsubusbscribe notification for everything that only subscribed by this client
    let to_unsubscribe = {
        let mut state = state.lock().expect("lock");

        let all_subscriptions: HashMap<SubscriptionId, usize> = {
            let mut all_subscriptions = HashMap::new();
            for client in state.clients.values() {
                for sub in client.subscriptions.keys() {
                    all_subscriptions
                        .entry(*sub)
                        .and_modify(|x| *x += 1)
                        .or_insert(1);
                }
            }
            all_subscriptions
        };

        let client = state
            .clients
            .remove(&client_id)
            .expect("must be in clients");

        let mut to_unsubscribe = vec![];
        for (sub_id, count) in all_subscriptions.iter() {
            if *count == 1 {
                if let Some(channel_id) = client.subscriptions.get(sub_id) {
                    to_unsubscribe.push(*channel_id)
                }
            }
        }

        to_unsubscribe
    };

    let interface = FoxgloveServer::new_from_state(state.clone());
    if let Some(listener) = listener {
        let mut listener = listener.lock().await;
        for channel_id in to_unsubscribe {
            listener.on_unsubscribe(interface.clone(), channel_id).await;
        }
    }
}

impl FoxgloveServer {
    fn new_from_state(state: Arc<Mutex<FoxgloveState>>) -> FoxgloveServer {
        return FoxgloveServer {
            state,
            listener: None,
        };
    }

    pub fn new_with_listener(
        name: String,
        listener: Box<dyn FoxgloveServerListener + Send>,
    ) -> Self {
        let server_info = ServerInfo {
            op: "serverInfo".to_string(),
            session_id: None,
            name,
            capabilities: vec![
                messages::Capability::ClientPublish,
                messages::Capability::Parameters,
                messages::Capability::ParametersSubscribe,
                messages::Capability::Services,
                // time
                // connectionGraph
                // assets
            ],
            supported_encodings: vec![
                messages::Encoding::Json,
                messages::Encoding::Protobuf,
                messages::Encoding::Ros1,
                messages::Encoding::Ros2,
                messages::Encoding::Cdr,
            ],
            metadata: Default::default(),
        };

        let listener = Some(Arc::new(tokio::sync::Mutex::new(listener)));
        let state = Arc::new(Mutex::new(FoxgloveState {
            server_info,
            next_channel_id: 1,
            next_service_id: 1,
            next_client_id: 1,
            clients: HashMap::new(),
            channels: HashMap::new(),
            services: HashMap::new(),
        }));

        return FoxgloveServer { state, listener };
    }

    pub fn new(name: String) -> Self {
        let server_info = ServerInfo {
            op: "serverInfo".to_string(),
            session_id: None,
            name,
            capabilities: vec![
                messages::Capability::ClientPublish,
                messages::Capability::Parameters,
                messages::Capability::ParametersSubscribe,
                messages::Capability::Services,
                // time
                // connectionGraph
                // assets
            ],
            supported_encodings: vec![
                messages::Encoding::Json,
                messages::Encoding::Protobuf,
                messages::Encoding::Ros1,
                messages::Encoding::Ros2,
                messages::Encoding::Cdr,
            ],
            metadata: Default::default(),
        };

        let state = Arc::new(Mutex::new(FoxgloveState {
            server_info,
            next_channel_id: 1,
            next_service_id: 1,
            next_client_id: 1,
            clients: HashMap::new(),
            channels: HashMap::new(),
            services: HashMap::new(),
        }));

        FoxgloveServer {
            state,
            listener: None,
        }
    }

    pub async fn start(&self, host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        // Create a TCP listener on the specified host and port
        let addr = format!("{}:{}", host, port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        log::info!("Server listening on: {}", addr);

        // Accept incoming TCP connections and upgrade them to WebSocket
        while let Ok((stream, _)) = listener.accept().await {
            log::info!("Accepted");
            let state = self.state.clone();
            let handler = self.listener.clone();
            tokio::spawn(handle_connection(stream, handler, state));
        }

        return Ok(());
    }

    // Assuming you have an async function to send JSON messages to all connected clients
    async fn broadcast(&self, message: Vec<u8>) {
        let state = self.state.lock().expect("lock");
        for client in state.clients.values() {
            let _ = client.sender.send(Message::Binary(message.clone()));
        }
    }

    pub async fn add_channel(&self, mut channel: Channel) -> ChannelId {
        let new_id = {
            let mut state = self.state.lock().expect("lock");
            let new_id = state.next_channel_id;
            state.next_channel_id += 1;
            channel.id = new_id;
            state.channels.insert(new_id, channel.clone());
            new_id
        };

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
        {
            let mut state = self.state.lock().expect("lock");

            // Check if the channel exists before attempting to remove it
            if state.channels.remove(&channel_id).is_none() {
                return Err(format!("Channel with ID {} does not exist.", channel_id));
            }
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
        let new_id = {
            let mut state = self.state.lock().expect("lock");
            let new_id = state.next_channel_id;
            state.next_channel_id += 1;
            service.id = new_id;
            state.services.insert(new_id, service.clone());
            new_id
        };

        let msg = messages::AdvertiseServices {
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
        {
            let mut state = self.state.lock().expect("lock");

            // Check if the channel exists before attempting to remove it
            if state.services.remove(&service_id).is_none() {
                return Err(format!("Service with ID {} does not exist.", service_id));
            }
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
        let state = self.state.lock().expect("lock");
        for client in state.clients.values() {
            let to_send: Vec<Parameter> = parameters
                .iter()
                .filter(|p| client.subscribed_params.contains(&p.name))
                .map(|p| p.clone())
                .collect();
            if !to_send.is_empty() {
                let msg = ParameterValues {
                    op: "parameterValues".to_string(),
                    parameters: to_send,
                    id: None,
                };
                let _ = client.sender.send(Message::Text(
                    serde_json::to_string(&msg).expect("serializing params"),
                ));
            }
        }
    }

    pub async fn send_message(&self, chan_id: ChannelId, timestamp_nanos: u64, payload: Vec<u8>) {
        let state = self.state.lock().expect("lock");
        for client in state.clients.values() {
            if let Some(&sub_id) = client.subscriptions_by_channel.get(&chan_id) {
                let sub_id = sub_id as u32;
                let timestamp_nanos = timestamp_nanos as u64;
                let mut message = Vec::new();
                message.push(1u8); // opcode 0x1
                message.extend_from_slice(&sub_id.to_le_bytes()); // Subscription ID
                message.extend_from_slice(&timestamp_nanos.to_le_bytes()); // Timestamp
                message.extend_from_slice(&payload); // Payload
                let _ = client.sender.send(Message::Binary(message));
            }
        }
    }

    /// Reset session Id and send new server info to clients.
    pub async fn reset_session_id(&mut self, new_session_id: Option<String>) {
        let mut state = self.state.lock().expect("lock");
        state.server_info.session_id = new_session_id;

        for client in state.clients.values() {
            let _ = client.sender.send(Message::Text(
                serde_json::to_string(&state.server_info).expect("encoding server_info"),
            ));
        }
    }
}
