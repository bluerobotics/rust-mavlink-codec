use bytes::BufMut;
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};

use crate::mav_types::mav_header::MavHeader;
use crate::mav_types::mav_message::MavMessage;
use crate::v1::{V1Packet, V1_STX};
use crate::v2::{V2Packet, V2_STX};

use super::MavFrame;

// #[repr(C, packed)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MavFrameSemanticModel {
    pub header: MavHeader,
    pub message: MavMessage,
}

impl Serialize for MavFrame {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        MavFrameSemanticModel::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MavFrame {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(MavFrame::from(&MavFrameSemanticModel::deserialize(
            deserializer,
        )?))
    }
}

impl From<&MavFrame> for MavFrameSemanticModel {
    fn from(frame: &MavFrame) -> Self {
        Self {
            header: frame.header(),
            message: frame.message(),
        }
    }
}

impl From<&MavFrameSemanticModel> for MavFrame {
    fn from(model: &MavFrameSemanticModel) -> Self {
        MavFrame::new(model, crate::PacketVersion::V2)
    }
}

impl From<&MavFrameSemanticModel> for V1Packet {
    fn from(frame: &MavFrameSemanticModel) -> Self {
        let payload = frame.message.bytes();
        let payload_len = payload.len();

        let mut buffer = bytes::BytesMut::with_capacity(
            V1Packet::STX_SIZE + V1Packet::HEADER_SIZE + payload_len + V1Packet::CHECKSUM_SIZE,
        );

        buffer.extend_from_slice(&[
            V1_STX,                      // packet start
            payload_len as u8,           // payload len
            0,                           // packet sequence
            frame.header.system_id(),    // system id
            frame.header.component_id(), // component id
        ]);
        buffer.put_u32_le(
            frame
                .header
                .message_id()
                .unwrap_or_else(|| frame.message.id()),
        );
        buffer.extend_from_slice(payload);
        buffer.put_u16_le(
            0, // TODO: compute CRC
        );

        V1Packet::new(buffer.freeze())
    }
}

impl From<&MavFrameSemanticModel> for V2Packet {
    fn from(frame: &MavFrameSemanticModel) -> Self {
        let payload = frame.message.bytes();
        let payload_len = payload.len();

        let mut buffer = bytes::BytesMut::with_capacity(
            V2Packet::STX_SIZE + V2Packet::HEADER_SIZE + payload_len + V2Packet::CHECKSUM_SIZE,
        );
        buffer.extend_from_slice(&[
            V2_STX,                      // packet start
            payload_len as u8,           // payload len
            0,                           // incompat flags
            0,                           // compat flags
            0,                           // packet sequence
            frame.header.system_id(),    // system id
            frame.header.component_id(), // component id
        ]);
        buffer.put_u32_le(
            frame
                .header
                .message_id()
                .unwrap_or_else(|| frame.message.id()),
        );
        buffer.extend_from_slice(payload);
        buffer.put_u16_le(
            0, // TODO: compute CRC
        );

        V2Packet::new(buffer.freeze())
    }
}
