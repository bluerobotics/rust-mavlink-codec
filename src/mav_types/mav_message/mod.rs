pub mod heartbeat;
pub mod rc_channels;

use std::any::Any;

use heartbeat::{serde_impl::HeartbeatMessageSemanticModel, HeartbeatMessage};
use rc_channels::{serde_impl::RcChannelsMessageSemanticModel, RcChannelsMessage};
use serde_derive::{Deserialize, Serialize};

use crate::{error::DecoderError, Packet};

pub trait MavMessageDef: MavMessageFields {
    const ID: u32;
    const LEN: u16;
    const CRC: u32;

    fn new(bytes: bytes::Bytes) -> Self;

    fn bytes(&self) -> &bytes::Bytes;

    fn as_slice(&self) -> &[u8];
}

pub trait MavMessageFields: Any {
    fn fields(
        &self,
    ) -> &'static phf::Map<&'static str, fn(&dyn MavMessageFields) -> serde_json::Value>;
    fn as_any(&self) -> &dyn Any;
}

#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MavMessage {
    Heartbeat(HeartbeatMessage) = HeartbeatMessage::ID,
    RcChannels(RcChannelsMessage) = RcChannelsMessage::ID,
    // ... and thousands of others
}

impl MavMessageFields for MavMessage {
    fn fields(
        &self,
    ) -> &'static phf::Map<&'static str, fn(&dyn MavMessageFields) -> serde_json::Value> {
        match self {
            MavMessage::Heartbeat(heartbeat_message) => heartbeat_message.fields(),
            MavMessage::RcChannels(rc_channels_message) => rc_channels_message.fields(),
        }
    }

    fn as_any(&self) -> &dyn Any {
        match self {
            MavMessage::Heartbeat(heartbeat_message) => heartbeat_message,
            MavMessage::RcChannels(rc_channels_message) => rc_channels_message,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MavMessageSemanticModel {
    Heartbeat(HeartbeatMessageSemanticModel) = HeartbeatMessage::ID,
    RcChannels(RcChannelsMessageSemanticModel) = RcChannelsMessage::ID,
}

impl MavMessage {
    pub fn bytes(&self) -> &bytes::Bytes {
        match self {
            MavMessage::Heartbeat(heartbeat_message) => heartbeat_message.bytes(),
            MavMessage::RcChannels(rc_channels_message) => rc_channels_message.bytes(),
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            MavMessage::Heartbeat(_) => HeartbeatMessage::ID,
            MavMessage::RcChannels(_) => RcChannelsMessage::ID,
        }
    }
}

impl TryFrom<&Packet> for MavMessage {
    type Error = DecoderError;

    fn try_from(value: &Packet) -> Result<Self, Self::Error> {
        crate::parser::parse(value)
    }
}

impl From<&MavMessage> for MavMessageSemanticModel {
    fn from(value: &MavMessage) -> Self {
        match value {
            MavMessage::Heartbeat(heartbeat_message) => MavMessageSemanticModel::Heartbeat(
                HeartbeatMessageSemanticModel::from(heartbeat_message),
            ),
            MavMessage::RcChannels(rc_channels_message) => MavMessageSemanticModel::RcChannels(
                RcChannelsMessageSemanticModel::from(rc_channels_message),
            ),
        }
    }
}
