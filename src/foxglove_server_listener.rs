use async_trait::async_trait;

use crate::messages;
use crate::CallId;
use crate::ChannelId;
use crate::FoxgloveServer;
use crate::Parameter;
use crate::RequestId;
use crate::ServiceId;

#[async_trait]
pub trait FoxgloveServerListener {
    async fn on_subscribe(&self, server: FoxgloveServer, channel_id: ChannelId);
    async fn on_unsubscribe(&self, server: FoxgloveServer, channel_id: ChannelId);
    async fn on_client_advertise(&self, server: FoxgloveServer, channel: messages::Channel);
    async fn on_client_unadvertise(&self, server: FoxgloveServer, channel_id: ChannelId);
    async fn on_client_message(
        &self,
        server: FoxgloveServer,
        channel_id: ChannelId,
        payload: Vec<u8>,
    );
    async fn on_service_request(
        &self,
        server: FoxgloveServer,
        service_id: ServiceId,
        call_id: CallId,
        encoding: String,
        payload: Vec<u8>,
    ) -> Vec<u8>;
    async fn on_get_parameters(
        &self,
        server: FoxgloveServer,
        param_names: Vec<String>,
        request_id: Option<RequestId>,
    ) -> Vec<Parameter>;
    async fn on_set_parameters(
        &self,
        server: FoxgloveServer,
        params: Vec<Parameter>,
        request_id: Option<RequestId>,
    ) -> Vec<Parameter>;
    async fn on_parameters_subscribe(
        &self,
        server: FoxgloveServer,
        param_name: Vec<String>,
        subscribe: bool,
    );
}
