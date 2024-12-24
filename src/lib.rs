mod client_channel;
mod client_state;
mod foxglove_server;
mod foxglove_server_listener;
mod messages;

pub type ChannelId = u32;
pub type CallId = u32;
pub type RequestId = u32;
pub type SubscriptionId = u32;
pub type ServiceId = u32;
pub type ClientChannelId = u32;

pub use client_channel::ClientChannel;
pub use client_state::ClientState;
pub use foxglove_server::FoxgloveServer;
pub use foxglove_server_listener::FoxgloveServerListener;
pub use messages::Advertise;
pub use messages::Capability;
pub use messages::Channel;
pub use messages::Parameter;
pub use messages::ParameterValues;
pub use messages::ServerInfo;
pub use messages::Service;
pub use messages::Status;
pub use messages::Unadvertise;
pub use messages::UnadvertiseServices;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
