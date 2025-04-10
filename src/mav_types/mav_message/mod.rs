pub mod heartbeat;

use heartbeat::HeartbeatMessage;
use serde_derive::{Deserialize, Serialize};

use crate::{error::DecoderError, Packet};

#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
/// TODO: This needs to be generated
pub enum MavMessage {
    HEARTBEAT(HeartbeatMessage) = HeartbeatMessage::ID,
    // ... and thousands of others
}

impl MavMessage {
    pub fn bytes(&self) -> &bytes::Bytes {
        match self {
            MavMessage::HEARTBEAT(heartbeat_message) => heartbeat_message.bytes(),
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            MavMessage::HEARTBEAT(_) => HeartbeatMessage::ID,
        }
    }
}

impl TryFrom<&Packet> for MavMessage {
    type Error = DecoderError;

    fn try_from(value: &Packet) -> Result<Self, Self::Error> {
        crate::parser::parse(value)
    }
}
