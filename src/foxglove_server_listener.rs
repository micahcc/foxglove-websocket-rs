use async_trait::async_trait;
use std::collections::HashMap;

use crate::messages;
use crate::CallId;
use crate::Capability;
use crate::ChannelId;
use crate::FoxgloveInterface;
use crate::Parameter;
use crate::RequestId;
use crate::ServiceId;

#[async_trait]
pub trait FoxgloveServerListener {
    fn name(&self) -> String;
    fn capabilities(&self) -> Vec<Capability>;
    fn supported_encodings(&self) -> Vec<String>;
    fn metadata(&self) -> HashMap<String, String>;

    async fn on_subscribe(&self, server: FoxgloveInterface, channel_id: ChannelId);
    async fn on_unsubscribe(&self, server: FoxgloveInterface, channel_id: ChannelId);
    async fn on_client_advertise(&self, server: FoxgloveInterface, channel: messages::Channel);
    async fn on_client_unadvertise(&self, server: FoxgloveInterface, channel_id: ChannelId);
    async fn on_client_message(
        &self,
        server: FoxgloveInterface,
        channel_id: ChannelId,
        payload: Vec<u8>,
    );
    async fn on_service_request(
        &self,
        server: FoxgloveInterface,
        service_id: ServiceId,
        call_id: CallId,
        encoding: String,
        payload: Vec<u8>,
    ) -> Vec<u8>;
    async fn on_get_parameters(
        &self,
        server: FoxgloveInterface,
        param_names: Vec<String>,
        request_id: Option<RequestId>,
    ) -> Vec<Parameter>;
    async fn on_set_parameters(
        &self,
        server: FoxgloveInterface,
        params: Vec<Parameter>,
        request_id: Option<RequestId>,
    ) -> Vec<Parameter>;
    async fn on_parameters_subscribe(
        &self,
        server: FoxgloveInterface,
        param_name: Vec<String>,
        subscribe: bool,
    );
}
