use bytes::{BufMut, BytesMut};
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_derive::{Deserialize as DeriveDeserialize, Serialize as DeriveSerialize};

use crate::mav_types::field_types::*;
use crate::mav_types::serde_utils::BitsField;

use super::HeartbeatMessage;

#[derive(DeriveSerialize, DeriveDeserialize)]
#[serde(rename_all = "snake_case")]
pub struct HeartbeatMessageSemanticModel {
    pub autopilot: MavAutopilot,
    pub base_mode: BitsField<u8>,
    pub custom_mode: u32,
    pub mavlink_version: u8,
    pub mavtype: MavType,
    pub system_status: MavState,
}

impl Serialize for HeartbeatMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let model = HeartbeatMessageSemanticModel {
            autopilot: self.autopilot(),
            base_mode: BitsField {
                bits: self.base_mode(),
            },
            custom_mode: self.custom_mode(),
            mavlink_version: self.mavlink_version(),
            mavtype: self.mav_type(),
            system_status: self.system_status(),
        };

        model.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HeartbeatMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let model = HeartbeatMessageSemanticModel::deserialize(deserializer)?;

        let mut bytes = BytesMut::with_capacity(HeartbeatMessage::LEN as usize);
        bytes.put_u32_le(model.custom_mode);
        bytes.put_u8(model.mavtype as u8);
        bytes.put_u8(model.autopilot as u8);
        bytes.put_u8(model.base_mode.bits);
        bytes.put_u8(model.system_status as u8);
        bytes.put_u8(model.mavlink_version);

        Ok(HeartbeatMessage::new(bytes.freeze()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bytes::Bytes;
    use serde_json::{from_value, json, to_value};

    const HEARTBEAT_BYTES: &[u8] = &[
        19,
        0,
        0,
        0, // custom_mode = 19
        MavType::MavTypeFixedWing as u8,
        MavAutopilot::MavAutopilotArdupilotmega as u8,
        81, // base_mode
        MavState::MavStateCritical as u8,
        3, // mavlink_version
    ];

    fn heartbeat_json() -> serde_json::Value {
        json!({
            "autopilot": { "type": "MAV_AUTOPILOT_ARDUPILOTMEGA" },
            "base_mode": { "bits": 81 },
            "custom_mode": 19,
            "mavlink_version": 3,
            "mavtype": { "type": "MAV_TYPE_FIXED_WING" },
            "system_status": { "type": "MAV_STATE_CRITICAL" },
        })
    }

    #[test]
    fn test_heartbeat_serialize() {
        let expected_json = heartbeat_json();
        dbg!(&expected_json);

        let message = HeartbeatMessage {
            buffer: Bytes::from(HEARTBEAT_BYTES),
        };
        dbg!(&message);

        let serialized_json = to_value(&message).unwrap();
        dbg!(&serialized_json);

        assert_eq!(serialized_json, expected_json);
    }

    #[test]
    fn test_heartbeat_deserialize() {
        let expected_message = HeartbeatMessage {
            buffer: Bytes::from(HEARTBEAT_BYTES),
        };
        dbg!(&expected_message);

        let json = heartbeat_json();
        dbg!(&json);

        let deserialized_message: HeartbeatMessage = from_value(json).unwrap();
        dbg!(&expected_message);

        assert_eq!(deserialized_message, expected_message);
    }
}
