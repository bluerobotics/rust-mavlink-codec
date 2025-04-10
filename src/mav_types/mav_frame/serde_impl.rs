use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use serde_derive::{Deserialize as DeriveDeserialize, Serialize as DeriveSerialize};

use crate::mav_types::mav_message::MavMessage;
use crate::{error::DecoderError, Packet};

use super::MavFrame;

// #[repr(C, packed)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MavFrameSemanticModel {
    pub header: MavFrame,
    pub message: MavMessage,
}

impl Serialize for MavFrame {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let model = MavFrameSemanticModel {
            header: todo!(),
            message: todo!(),
        };

        model.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MavFrame {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // let model = MavFrameSemanticModel::deserialize(deserializer)?;

        // let mut bytes = BytesMut::with_capacity(MavFrame::LEN as usize);
        // bytes.put_u32_le(model.custom_mode);
        // bytes.put_u8(model.mavtype as u8);
        // bytes.put_u8(model.autopilot as u8);
        // bytes.put_u8(model.base_mode.bits);
        // bytes.put_u8(model.system_status as u8);
        // bytes.put_u8(model.mavlink_version);

        // Ok(MavFrame::new(bytes.freeze()))

        todo!()
    }
}
