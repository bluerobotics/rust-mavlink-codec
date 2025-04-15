use bytes::{BufMut, BytesMut};
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_derive::{Deserialize as DeriveDeserialize, Serialize as DeriveSerialize};

use crate::v2::V2Packet;

use super::MavHeader;

#[derive(Debug, Default, Clone, PartialEq, DeriveSerialize, DeriveDeserialize)]
pub struct MavHeaderSemanticModel {
    pub sequence: u8,
    pub system_id: u8,
    pub component_id: u8,
    pub message_id: Option<u32>,
}

impl Serialize for MavHeader {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        MavHeaderSemanticModel::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MavHeader {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let model = MavHeaderSemanticModel::deserialize(deserializer)?;

        let mut bytes = BytesMut::with_capacity(V2Packet::HEADER_SIZE);
        bytes.put_u8(model.sequence);
        bytes.put_u8(model.system_id);
        bytes.put_u8(model.component_id);
        bytes.put_u32(model.message_id.unwrap_or_default());

        Ok(Self::new(bytes.freeze()))
    }
}

impl From<&MavHeader> for MavHeaderSemanticModel {
    fn from(header: &MavHeader) -> Self {
        MavHeaderSemanticModel {
            sequence: header.sequence(),
            system_id: header.system_id(),
            component_id: header.component_id(),
            message_id: header.message_id(),
        }
    }
}
