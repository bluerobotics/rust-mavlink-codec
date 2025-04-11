pub mod serde_impl;

use serde_impl::MavFrameSemanticModel;

use crate::{v1::V1Packet, v2::V2Packet, Packet, PacketVersion};

use super::{
    mav_header::MavHeader,
    mav_message::{
        heartbeat::HeartbeatMessage, rc_channels::RcChannelsMessage, MavMessage, MavMessageDef,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct MavFrame {
    packet: Packet,
}

impl MavFrame {
    pub fn new(model: &MavFrameSemanticModel, version: PacketVersion) -> Self {
        let packet = match version {
            PacketVersion::V1 => Packet::V1(V1Packet::from(model)),
            PacketVersion::V2 => Packet::V2(V2Packet::from(model)),
        };

        Self { packet }
    }

    pub fn header(&self) -> MavHeader {
        MavHeader::new(self.packet.header_bytes())
    }

    pub fn message(&self) -> MavMessage {
        let message_id = self.packet.message_id();
        match message_id {
            HeartbeatMessage::ID => {
                MavMessage::Heartbeat(HeartbeatMessage::new(self.packet.payload_bytes()))
            }
            RcChannelsMessage::ID => {
                MavMessage::RcChannels(RcChannelsMessage::new(self.packet.payload_bytes()))
            }
            unknown_id => {
                Err(format!("Unknown ID: {unknown_id:?}")).unwrap() // Unreachable because of the check during the MavFrame creation
            }
        }
    }
}

impl From<&Packet> for MavFrame {
    #[inline(always)]
    fn from(value: &Packet) -> Self {
        Self {
            packet: value.clone(),
        }
    }
}

impl From<&MavFrame> for Packet {
    #[inline(always)]
    fn from(value: &MavFrame) -> Self {
        value.packet.clone()
    }
}

impl From<Packet> for MavFrame {
    #[inline(always)]
    fn from(value: Packet) -> Self {
        Self { packet: value }
    }
}

impl From<MavFrame> for Packet {
    #[inline(always)]
    fn from(value: MavFrame) -> Self {
        value.packet
    }
}
