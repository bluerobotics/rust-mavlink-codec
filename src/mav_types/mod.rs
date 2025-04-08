pub mod heartbeat;
pub mod serde_utils;

use heartbeat::HeartbeatMessage;
// use classic_bitfield::bitfield_enum;
use serde_derive::{Deserialize, Serialize};

#[macro_export]
macro_rules! match_message {
    ($id:expr, $buf:expr) => {
        match $id {
            $crate::mods::generated::types::HeartbeatMessage::ID => {
                $crate::mods::generated::types::HeartbeatMessage::from_bytes($buf)
                    .map($crate::mods::generated::types::MavMessageRef::HEARTBEAT)
            }
            // add others here
            _ => None,
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct MavFrame {
    pub header: MavHeader,
    pub message: MavMessage,
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)] // TODO: maybe derive Copy?
pub struct MavHeader {
    pub sequence: u8,
    pub system_id: u8,
    pub component_id: u8,
    pub message_id: u32,
}

#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MavMessage {
    HEARTBEAT(HeartbeatMessage) = HeartbeatMessage::ID,
    // ... and thousands of others
}

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MavType {
    #[default]
    #[serde(rename = "MAV_TYPE_GENERIC")]
    MavTypeGeneric = 0,
    #[serde(rename = "MAV_TYPE_FIXED_WING")]
    MavTypeFixedWing = 1,
    #[serde(rename = "MAV_TYPE_QUADROTOR")]
    MavTypeQuadrotor = 2,
    #[serde(rename = "MAV_TYPE_COAXIAL")]
    MavTypeCoaxial = 3,
    #[serde(rename = "MAV_TYPE_SUBMARINE")]
    MavTypeSubmarine = 12,
    // ... other variants
}

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MavAutopilot {
    #[default]
    #[serde(rename = "MAV_AUTOPILOT_GENERIC")]
    MavAutopilotGeneric = 0,
    #[serde(rename = "MAV_AUTOPILOT_RESERVED")]
    MavAutopilotReserved = 1,
    #[serde(rename = "MAV_AUTOPILOT_SLUGS")]
    MavAutopilotSlugs = 2,
    #[serde(rename = "MAV_AUTOPILOT_ARDUPILOTMEGA")]
    MavAutopilotArdupilotmega = 3,
    // ...
}

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MavState {
    #[default]
    #[serde(rename = "MAV_STATE_UNINIT")]
    MavStateUninit = 0,
    #[serde(rename = "MAV_STATE_BOOT")]
    MavStateBoot = 1,
    #[serde(rename = "MAV_STATE_CALIBRATING")]
    MavStateCalibrating = 2,
    #[serde(rename = "MAV_STATE_STANDBY")]
    MavStateStandby = 3,
    #[serde(rename = "MAV_STATE_ACTIVE")]
    MavStateActive = 4,
    #[serde(rename = "MAV_STATE_CRITICAL")]
    MavStateCritical = 5,
    #[serde(rename = "MAV_STATE_EMERGENCY")]
    MavStateEmergency = 6,
    #[serde(rename = "MAV_STATE_POWEROFF")]
    MavStatePoweroff = 7,
    #[serde(rename = "MAV_STATE_FLIGHT_TERMINATION")]
    MavStateFlightTermination = 8,
}

// #[bitfield_enum(as u8)]
// #[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, , Immutable)]
// pub enum MavModeFlag {
//     MAV_MODE_FLAG_CUSTOM_MODE_ENABLED,
//     MAV_MODE_FLAG_TEST_ENABLED,
//     MAV_MODE_FLAG_AUTO_ENABLED,
//     MAV_MODE_FLAG_GUIDED_ENABLED,
//     MAV_MODE_FLAG_STABILIZE_ENABLED,
//     MAV_MODE_FLAG_HIL_ENABLED,
//     MAV_MODE_FLAG_MANUAL_INPUT_ENABLED,
//     MAV_MODE_FLAG_SAFETY_ARMED,
// }

// impl Default for MavModeFlag {
//     fn default() -> Self {
//         Self(128)
//     }
// }
