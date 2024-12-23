mod foxglove_server_listener;
mod foxglove_server;
mod messages;
mod client_channel;
mod client_state;

pub type ChannelId = u32;
pub type SubscriptionId = u32;
pub type ServiceId = u32;
pub type ClientChannelId = u32;

pub use messages::ServerInfo;
pub use messages::Status;
pub use messages::Parameter;
pub use messages::Service;
pub use messages::Channel;
pub use foxglove_server::FoxgloveServer;
pub use client_channel::ClientChannel;
pub use client_state::ClientState;
pub use foxglove_server_listener::FoxgloveServerListener;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
