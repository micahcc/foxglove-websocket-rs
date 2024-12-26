use std::collections::{HashMap, HashSet};
use tokio_tungstenite::tungstenite::Message;

use crate::messages;
use crate::ChannelId;
use crate::ClientChannelId;
use crate::SubscriptionId;

pub struct ClientState {
    pub remote: String,

    // Maps subscription IDs to channel IDs to keep track of what the client is subscribed to.
    pub subscriptions: HashMap<SubscriptionId, ChannelId>,

    // Reverse mapping of `subscriptions` to quickly look up subscription IDs by channel IDs.
    pub subscriptions_by_channel: HashMap<ChannelId, SubscriptionId>,

    // Maps advertised client channel IDs to client channels to manage channels advertised by the client.
    pub advertisements_by_channel: HashMap<ClientChannelId, messages::Channel>,

    // A set of parameter names that the client has subscribed to.
    pub subscribed_params: HashSet<String>,
    // Channels to communicate with the client task, e.g., for sending messages.
    pub sender: tokio::sync::mpsc::UnboundedSender<Message>,
}
