use bytes::{BufMut, BytesMut};
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_derive::{Deserialize as DeriveDeserialize, Serialize as DeriveSerialize};

use crate::mav_types::mav_message::MavMessageDef;

use super::RcChannelsMessage;

#[derive(Debug, Clone, PartialEq, DeriveSerialize, DeriveDeserialize)]
#[serde(rename_all = "snake_case")]
pub struct RcChannelsMessageSemanticModel {
    pub time_boot_ms: u32,
    pub chan1_raw: u16,
    pub chan2_raw: u16,
    pub chan3_raw: u16,
    pub chan4_raw: u16,
    pub chan5_raw: u16,
    pub chan6_raw: u16,
    pub chan7_raw: u16,
    pub chan8_raw: u16,
    pub chan9_raw: u16,
    pub chan10_raw: u16,
    pub chan11_raw: u16,
    pub chan12_raw: u16,
    pub chan13_raw: u16,
    pub chan14_raw: u16,
    pub chan15_raw: u16,
    pub chan16_raw: u16,
    pub chan17_raw: u16,
    pub chan18_raw: u16,
    pub chancount: u8,
    pub rssi: u8,
}

impl From<&RcChannelsMessage> for RcChannelsMessageSemanticModel {
    fn from(message: &RcChannelsMessage) -> Self {
        RcChannelsMessageSemanticModel {
            time_boot_ms: message.time_boot_ms(),
            chan1_raw: message.chan1_raw(),
            chan2_raw: message.chan2_raw(),
            chan3_raw: message.chan3_raw(),
            chan4_raw: message.chan4_raw(),
            chan5_raw: message.chan5_raw(),
            chan6_raw: message.chan6_raw(),
            chan7_raw: message.chan7_raw(),
            chan8_raw: message.chan8_raw(),
            chan9_raw: message.chan9_raw(),
            chan10_raw: message.chan10_raw(),
            chan11_raw: message.chan11_raw(),
            chan12_raw: message.chan12_raw(),
            chan13_raw: message.chan13_raw(),
            chan14_raw: message.chan14_raw(),
            chan15_raw: message.chan15_raw(),
            chan16_raw: message.chan16_raw(),
            chan17_raw: message.chan17_raw(),
            chan18_raw: message.chan18_raw(),
            chancount: message.chancount(),
            rssi: message.rssi(),
        }
    }
}

impl From<&RcChannelsMessageSemanticModel> for RcChannelsMessage {
    fn from(model: &RcChannelsMessageSemanticModel) -> Self {
        let mut buffer = BytesMut::with_capacity(RcChannelsMessage::LEN as usize);

        buffer.put_u32_le(model.time_boot_ms);
        buffer.put_u16_le(model.chan1_raw);
        buffer.put_u16_le(model.chan2_raw);
        buffer.put_u16_le(model.chan3_raw);
        buffer.put_u16_le(model.chan4_raw);
        buffer.put_u16_le(model.chan5_raw);
        buffer.put_u16_le(model.chan6_raw);
        buffer.put_u16_le(model.chan7_raw);
        buffer.put_u16_le(model.chan8_raw);
        buffer.put_u16_le(model.chan9_raw);
        buffer.put_u16_le(model.chan10_raw);
        buffer.put_u16_le(model.chan11_raw);
        buffer.put_u16_le(model.chan12_raw);
        buffer.put_u16_le(model.chan13_raw);
        buffer.put_u16_le(model.chan14_raw);
        buffer.put_u16_le(model.chan15_raw);
        buffer.put_u16_le(model.chan16_raw);
        buffer.put_u16_le(model.chan17_raw);
        buffer.put_u16_le(model.chan18_raw);
        buffer.put_u8(model.chancount);
        buffer.put_u8(model.rssi);

        RcChannelsMessage::new(buffer.freeze())
    }
}

impl Serialize for RcChannelsMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        RcChannelsMessageSemanticModel::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RcChannelsMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(RcChannelsMessage::from(
            &RcChannelsMessageSemanticModel::deserialize(deserializer)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use serde_json::{from_value, json, to_value};

    const RC_CHANNELS_BYTES: &[u8] = &[
        0xe8, 0x03, 0x00, 0x00, // time_boot_ms (u32_le)
        0xdc, 0x05, // chan1_raw (u16_le)
        0xdc, 0x05, // chan2_raw (u16_le)
        0xdc, 0x05, // chan3_raw (u16_le)
        0xdc, 0x05, // chan4_raw (u16_le)
        0xdc, 0x05, // chan5_raw (u16_le)
        0xdc, 0x05, // chan6_raw (u16_le)
        0xdc, 0x05, // chan7_raw (u16_le)
        0xdc, 0x05, // chan8_raw (u16_le)
        0xe8, 0x03, // chan9_raw (u16_le)
        0xe8, 0x03, // chan10_raw (u16_le)
        0xe8, 0x03, // chan11_raw (u16_le)
        0xe8, 0x03, // chan12_raw (u16_le)
        0xe8, 0x03, // chan13_raw (u16_le)
        0xe8, 0x03, // chan14_raw (u16_le)
        0xe8, 0x03, // chan15_raw (u16_le)
        0xe8, 0x03, // chan16_raw (u16_le)
        0xe8, 0x03, // chan17_raw (u16_le)
        0xe8, 0x03, // chan18_raw (u16_le)
        18,   // chancount (u8)
        210,  // rssi (u8)
    ];

    fn rc_channels_json() -> serde_json::Value {
        json!({
            "time_boot_ms": 1000,
            "chan1_raw": 1500,
            "chan2_raw": 1500,
            "chan3_raw": 1500,
            "chan4_raw": 1500,
            "chan5_raw": 1500,
            "chan6_raw": 1500,
            "chan7_raw": 1500,
            "chan8_raw": 1500,
            "chan9_raw": 1000,
            "chan10_raw": 1000,
            "chan11_raw": 1000,
            "chan12_raw": 1000,
            "chan13_raw": 1000,
            "chan14_raw": 1000,
            "chan15_raw": 1000,
            "chan16_raw": 1000,
            "chan17_raw": 1000,
            "chan18_raw": 1000,
            "chancount": 18,
            "rssi": 210
        })
    }

    #[test]
    fn test_rc_channels_serialize() {
        let expected_json = rc_channels_json();
        dbg!(&expected_json);

        let message = RcChannelsMessage {
            buffer: Bytes::from(RC_CHANNELS_BYTES),
        };
        dbg!(&message);

        let serialized_json = to_value(&message).unwrap();
        dbg!(&serialized_json);

        assert_eq!(serialized_json, expected_json);
    }

    #[test]
    fn test_rc_channels_deserialize() {
        let expected_message = RcChannelsMessage {
            buffer: Bytes::from(RC_CHANNELS_BYTES),
        };
        dbg!(&expected_message);

        let json = rc_channels_json();
        dbg!(&json);

        let deserialized_message: RcChannelsMessage = from_value(json).unwrap();
        dbg!(&deserialized_message);

        assert_eq!(deserialized_message, expected_message);
    }
}
