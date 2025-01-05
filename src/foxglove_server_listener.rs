use async_trait::async_trait;

use crate::messages;
use crate::CallId;
use crate::ChannelId;
use crate::FoxgloveServer;
use crate::Parameter;
use crate::ServiceId;

#[async_trait]
#[allow(unused)]
pub trait FoxgloveServerListener {
    /// Called whenever a client subscribes to a channel
    async fn on_subscribe(&mut self, server: FoxgloveServer, channel_id: ChannelId) {}

    /// Called whenever a client unsubscribes from a channel
    async fn on_unsubscribe(&mut self, server: FoxgloveServer, channel_id: ChannelId) {}

    /// Called whenever a client announces a new topic
    async fn on_client_advertise(&mut self, server: FoxgloveServer, channel: messages::Channel) {}

    /// Called whenever a client stops publishing a channel
    async fn on_client_unadvertise(&mut self, server: FoxgloveServer, channel_id: ChannelId) {}

    /// Called whenver a client publishes a message, the channel_id will match one of the previous
    /// on_client_advertise. To decode the message, use the encoding information provided in the
    /// advertisement
    async fn on_client_message(
        &mut self,
        server: FoxgloveServer,
        channel_id: ChannelId,
        payload: Vec<u8>,
    ) {
    }

    /// A service request is a call-response by one of the clients.
    /// To be determined how encoding is provided.
    async fn on_service_request(
        &mut self,
        server: FoxgloveServer,
        service_id: ServiceId,
        call_id: CallId,
        encoding: String,
        payload: Vec<u8>,
    ) -> Vec<u8> {
        return vec![];
    }

    /// Allows clients to query parameters. A basic implementation is just a key-value store where
    /// on_set_parameters sets parameters and on_get_parameters gets them
    async fn on_get_parameters(
        &mut self,
        server: FoxgloveServer,
        param_names: Vec<String>,
        request_id: Option<String>,
    ) -> Vec<Parameter> {
        return vec![];
    }

    /// Allows clients to set parameters. A basic implementation is just a key-value store where
    /// on_set_parameters sets parameters and on_get_parameters gets them
    async fn on_set_parameters(
        &mut self,
        server: FoxgloveServer,
        params: Vec<Parameter>,
        request_id: Option<String>,
    ) -> Vec<Parameter> {
        return vec![];
    }

    /// Provides clients with a way to request updates on a parameter. Its up to the implementer
    /// to actually do the relaying when on_set_parameters is called.
    async fn on_parameters_subscribe(
        &mut self,
        server: FoxgloveServer,
        param_name: Vec<String>,
        subscribe: bool,
    ) {
    }
}
