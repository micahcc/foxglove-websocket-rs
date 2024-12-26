use std::collections::HashMap;

use crate::Channel;
use crate::ChannelId;
use crate::ClientId;
use crate::ClientState;
use crate::ServerInfo;
use crate::Service;
use crate::ServiceId;

/// Common data, everything inside here should be shared
pub struct FoxgloveState {
    pub server_info: ServerInfo,

    pub next_client_id: u32,
    pub clients: HashMap<ClientId, ClientState>,

    pub next_channel_id: u32,
    pub channels: HashMap<ChannelId, Channel>,

    pub next_service_id: u32,
    pub services: HashMap<ServiceId, Service>,
}
