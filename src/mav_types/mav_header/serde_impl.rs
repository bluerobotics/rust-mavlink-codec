use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use serde_derive::{Deserialize as DeriveDeserialize, Serialize as DeriveSerialize};

use crate::{error::DecoderError, Packet};

use super::MavHeader;

// #[repr(C, packed)]
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
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
        let model = MavHeaderSemanticModel {
            sequence: self.sequence(),
            system_id: self.system_id(),
            component_id: self.component_id(),
            message_id: Some(self.message_id()),
        };

        model.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MavHeader {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // let model = MavHeaderSemanticModel::deserialize(deserializer)?;

        // let mut bytes = BytesMut::with_capacity(mav_header::LEN as usize);
        // bytes.put_u32_le(model.custom_mode);
        // bytes.put_u8(model.mavtype as u8);
        // bytes.put_u8(model.autopilot as u8);
        // bytes.put_u8(model.base_mode.bits);
        // bytes.put_u8(model.system_status as u8);
        // bytes.put_u8(model.mavlink_version);

        // Ok(mav_header::new(bytes.freeze()))

        todo!()
    }
}
