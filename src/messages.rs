use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::RequestId;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[repr(u8)]
pub enum BinaryOpcode {
    MessageData = 1,
    Time = 2,
    ServiceCallResponse = 3,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[repr(u8)]
pub enum ClientBinaryOpcode {
    MessageData = 1,
    ServiceCallRequest = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Capability {
    ClientPublish,
    Parameters,
    ParametersSubscribe,
    Time,
    Services,
    ConnectionGraph,
    Assets,
}

// Server Info
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub op: String,
    pub name: String,
    pub capabilities: Vec<Capability>,
    pub supported_encodings: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub session_id: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[repr(u8)]
pub enum StatusLevel {
    Info = 0,
    Warning = 1,
    Error = 2,
}

// Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub op: String,
    pub level: u8,
    pub message: String,
    pub id: Option<String>,
}

// Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientChannel {
    pub op: String,
    pub level: u8,
    pub message: String,
    pub id: Option<String>,
}

// Remove Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveStatus {
    pub op: String,
    pub status_ids: Vec<String>,
}

// Advertise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advertise {
    pub op: String, // advertise
    pub channels: Vec<Channel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: u32,
    pub topic: String,
    pub encoding: String,
    pub schema_name: String,
    pub schema: String,
    pub schema_encoding: Option<String>,
}

// Unadvertise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unadvertise {
    pub op: String, // unadvertise
    pub channel_ids: Vec<u32>,
}

// Parameter Values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValues {
    pub op: String, // parameterValues
    pub parameters: Vec<Parameter>,
    pub id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub value: serde_json::Value,
    pub type_: Option<String>,
    pub id: Option<String>,
}

// Advertise Services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvertiseServices {
    pub op: String,
    pub services: Vec<Service>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: u32,
    pub name: String,
    pub type_: String,
    pub request: Option<ServiceSchema>,
    pub response: Option<ServiceSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSchema {
    pub encoding: String,
    pub schema_name: String,
    pub schema_encoding: String,
    pub schema: String,
}

// Unadvertise Services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnadvertiseServices {
    pub op: String, // unadvertiseServices
    pub service_ids: Vec<u32>,
}

// Connection Graph Update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionGraphUpdate {
    pub op: String,
    pub published_topics: Vec<Topic>,
    pub subscribed_topics: Vec<Topic>,
    pub advertised_services: Vec<Service>,
    pub removed_topics: Vec<String>,
    pub removed_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub publisher_ids: Vec<String>,
    pub subscriber_ids: Vec<String>,
}

// Service Call Failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCallFailure {
    pub op: String,
    pub service_id: u32,
    pub call_id: u32,
    pub message: String,
}

// Subscribe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscribe {
    pub op: String,
    pub subscriptions: Vec<Subscription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: u32,
    pub channel_id: u32,
}

// Unsubscribe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unsubscribe {
    pub op: String,
    pub subscription_ids: Vec<u32>,
}

// Client Advertise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAdvertise {
    pub op: String,
    pub channels: Vec<Channel>,
}

// Client Unadvertise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientUnadvertise {
    pub op: String,
    pub channel_ids: Vec<RequestId>,
}

// Get Parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetParameters {
    pub op: String,
    pub parameter_names: Vec<String>,
    pub id: Option<u32>,
}

// Set Parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParameters {
    pub op: String,
    pub parameters: Vec<Parameter>,
    pub id: Option<RequestId>,
}

// Subscribe Parameter Update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeParameterUpdate {
    pub op: String,
    pub parameter_names: Vec<String>,
}

// Unsubscribe Parameter Update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeParameterUpdate {
    pub op: String,
    pub parameter_names: Vec<String>,
}

// Subscribe Connection Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubscribeConnectionGraph {
    op: String,
}

// Unsubscribe Connection Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UnsubscribeConnectionGraph {
    op: String,
}

// Fetch Asset
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FetchAsset {
    op: String,
    uri: String,
    request_id: u32,
}

// Fetch Asset Response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FetchAssetResponse {
    op: String,
    request_id: u32,
    status: u8,
    error_message: Option<String>,
    asset_data: Vec<u8>,
}

// Binary message types

// Message Data
struct MessageData {
    opcode: u8,
    subscription_id: u32,
    timestamp: u64,
    payload: Vec<u8>,
}

// Time
struct Time {
    opcode: u8,
    timestamp: u64,
}

// Service Call Response
struct ServiceCallResponse {
    opcode: u8,
    service_id: u32,
    call_id: u32,
    encoding_length: u32,
    encoding: Vec<u8>,
    response_payload: Vec<u8>,
}

// Client Message Data
struct ClientMessageData {
    opcode: u8,
    channel_id: u32,
    payload: Vec<u8>,
}

// Service Call Request
struct ServiceCallRequest {
    opcode: u8,
    service_id: u32,
    call_id: u32,
    encoding_length: u32,
    encoding: Vec<u8>,
    request_payload: Vec<u8>,
}
