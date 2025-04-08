#![doc = "This file was automatically generated, do not edit"]
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use bitflags::bitflags;
use mavlink_core::{ MavlinkVersion, Message, MessageData, bytes::Bytes, bytes_mut::BytesMut };
use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HEARTBEAT_DATA {
    pub custom_mode: u32,
    pub mavtype: MavType,
    pub autopilot: MavAutopilot,
    pub base_mode: MavModeFlag,
    pub system_status: MavState,
    pub mavlink_version: u8,
}

impl HEARTBEAT_DATA {
    pub const ENCODED_LEN: usize = 9usize;
    pub const DEFAULT: Self = Self {
        custom_mode: 0_u32,
        mavtype: MavType::DEFAULT,
        autopilot: MavAutopilot::DEFAULT,
        base_mode: MavModeFlag::DEFAULT,
        system_status: MavState::DEFAULT,
        mavlink_version: 0_u8,
    };
}

impl Default for HEARTBEAT_DATA {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

impl MessageData for HEARTBEAT_DATA {
    type Message = MavMessage;
    const ID: u32 = 0u32;
    const NAME: &'static str = "HEARTBEAT";
    const EXTRA_CRC: u8 = 50u8;
    const ENCODED_LEN: usize = 9usize;
    fn deser(
        _version: MavlinkVersion,
        __input: &[u8]
    ) -> Result<Self, ::mavlink_core::error::ParserError> {
        let avail_len = __input.len();
        let mut payload_buf = [0; Self::ENCODED_LEN];
        let mut buf = if avail_len < Self::ENCODED_LEN {
            payload_buf[0..avail_len].copy_from_slice(__input);
            Bytes::new(&payload_buf)
        } else {
            Bytes::new(__input)
        };
        let mut __struct = Self::default();
        __struct.custom_mode = buf.get_u32_le();
        let tmp = buf.get_u8();
        __struct.mavtype = FromPrimitive::from_u8(tmp).ok_or(
            ::mavlink_core::error::ParserError::InvalidEnum {
                enum_type: "MavType",
                value: tmp as u32,
            }
        )?;
        let tmp = buf.get_u8();
        __struct.autopilot = FromPrimitive::from_u8(tmp).ok_or(
            ::mavlink_core::error::ParserError::InvalidEnum {
                enum_type: "MavAutopilot",
                value: tmp as u32,
            }
        )?;
        let tmp = buf.get_u8();
        __struct.base_mode = MavModeFlag::from_bits(tmp & MavModeFlag::all().bits()).ok_or(
            ::mavlink_core::error::ParserError::InvalidFlag {
                flag_type: "MavModeFlag",
                value: tmp as u32,
            }
        )?;
        let tmp = buf.get_u8();
        __struct.system_status = FromPrimitive::from_u8(tmp).ok_or(
            ::mavlink_core::error::ParserError::InvalidEnum {
                enum_type: "MavState",
                value: tmp as u32,
            }
        )?;
        __struct.mavlink_version = buf.get_u8();
        Ok(__struct)
    }
    fn ser(&self, version: MavlinkVersion, bytes: &mut [u8]) -> usize {
        let mut __tmp = BytesMut::new(bytes);
        #[allow(clippy::absurd_extreme_comparisons)]
        #[allow(unused_comparisons)]
        if __tmp.remaining() < Self::ENCODED_LEN {
            panic!(
                "buffer is too small (need {} bytes, but got {})",
                Self::ENCODED_LEN,
                __tmp.remaining()
            )
        }
        __tmp.put_u32_le(self.custom_mode);
        __tmp.put_u8(self.mavtype as u8);
        __tmp.put_u8(self.autopilot as u8);
        __tmp.put_u8(self.base_mode.bits());
        __tmp.put_u8(self.system_status as u8);
        __tmp.put_u8(self.mavlink_version);
        if matches!(version, MavlinkVersion::V2) {
            let len = __tmp.len();
            ::mavlink_core::utils::remove_trailing_zeroes(&bytes[..len])
        } else {
            __tmp.len()
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive, ToPrimitive, Serialize, Deserialize)]
#[serde(tag = "type")]
#[repr(u32)]
pub enum MavType {
    MAV_TYPE_GENERIC = 0,
    MAV_TYPE_FIXED_WING = 1,
    MAV_TYPE_QUADROTOR = 2,
    MAV_TYPE_COAXIAL = 3,
    MAV_TYPE_HELICOPTER = 4,
    MAV_TYPE_ANTENNA_TRACKER = 5,
    MAV_TYPE_GCS = 6,
    MAV_TYPE_AIRSHIP = 7,
    MAV_TYPE_FREE_BALLOON = 8,
    MAV_TYPE_ROCKET = 9,
    MAV_TYPE_GROUND_ROVER = 10,
    MAV_TYPE_SURFACE_BOAT = 11,
    MAV_TYPE_SUBMARINE = 12,
    MAV_TYPE_HEXAROTOR = 13,
    MAV_TYPE_OCTOROTOR = 14,
    MAV_TYPE_TRICOPTER = 15,
    MAV_TYPE_FLAPPING_WING = 16,
    MAV_TYPE_KITE = 17,
    MAV_TYPE_ONBOARD_CONTROLLER = 18,
    MAV_TYPE_VTOL_TAILSITTER_DUOROTOR = 19,
    MAV_TYPE_VTOL_TAILSITTER_QUADROTOR = 20,
    MAV_TYPE_VTOL_TILTROTOR = 21,
    MAV_TYPE_VTOL_FIXEDROTOR = 22,
    MAV_TYPE_VTOL_TAILSITTER = 23,
    MAV_TYPE_VTOL_TILTWING = 24,
    MAV_TYPE_VTOL_RESERVED5 = 25,
    MAV_TYPE_GIMBAL = 26,
    MAV_TYPE_ADSB = 27,
    MAV_TYPE_PARAFOIL = 28,
    MAV_TYPE_DODECAROTOR = 29,
    MAV_TYPE_CAMERA = 30,
    MAV_TYPE_CHARGING_STATION = 31,
    MAV_TYPE_FLARM = 32,
    MAV_TYPE_SERVO = 33,
    MAV_TYPE_ODID = 34,
    MAV_TYPE_DECAROTOR = 35,
    MAV_TYPE_BATTERY = 36,
    MAV_TYPE_PARACHUTE = 37,
    MAV_TYPE_LOG = 38,
    MAV_TYPE_OSD = 39,
    MAV_TYPE_IMU = 40,
    MAV_TYPE_GPS = 41,
    MAV_TYPE_WINCH = 42,
}
impl MavType {
    pub const DEFAULT: Self = Self::MAV_TYPE_GENERIC;
}
impl Default for MavType {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive, ToPrimitive, Serialize, Deserialize)]
#[serde(tag = "type")]
#[repr(u32)]
pub enum MavAutopilot {
    MAV_AUTOPILOT_GENERIC = 0,
    MAV_AUTOPILOT_RESERVED = 1,
    MAV_AUTOPILOT_SLUGS = 2,
    MAV_AUTOPILOT_ARDUPILOTMEGA = 3,
    MAV_AUTOPILOT_OPENPILOT = 4,
    MAV_AUTOPILOT_GENERIC_WAYPOINTS_ONLY = 5,
    MAV_AUTOPILOT_GENERIC_WAYPOINTS_AND_SIMPLE_NAVIGATION_ONLY = 6,
    MAV_AUTOPILOT_GENERIC_MISSION_FULL = 7,
    MAV_AUTOPILOT_INVALID = 8,
    MAV_AUTOPILOT_PPZ = 9,
    MAV_AUTOPILOT_UDB = 10,
    MAV_AUTOPILOT_FP = 11,
    MAV_AUTOPILOT_PX4 = 12,
    MAV_AUTOPILOT_SMACCMPILOT = 13,
    MAV_AUTOPILOT_AUTOQUAD = 14,
    MAV_AUTOPILOT_ARMAZILA = 15,
    MAV_AUTOPILOT_AEROB = 16,
    MAV_AUTOPILOT_ASLUAV = 17,
    MAV_AUTOPILOT_SMARTAP = 18,
    MAV_AUTOPILOT_AIRRAILS = 19,
    MAV_AUTOPILOT_REFLEX = 20,
}
impl MavAutopilot {
    pub const DEFAULT: Self = Self::MAV_AUTOPILOT_GENERIC;
}
impl Default for MavAutopilot {
    fn default() -> Self {
        Self::DEFAULT
    }
}

bitflags! {
    #[derive (Serialize , Deserialize)]
    pub struct MavModeFlag : u8 { 
        const MAV_MODE_FLAG_SAFETY_ARMED = 128 ;
        const MAV_MODE_FLAG_MANUAL_INPUT_ENABLED = 64 ;
        const MAV_MODE_FLAG_HIL_ENABLED = 32 ;
        const MAV_MODE_FLAG_STABILIZE_ENABLED = 16 ; 
        const MAV_MODE_FLAG_GUIDED_ENABLED = 8 ; 
        const MAV_MODE_FLAG_AUTO_ENABLED = 4 ; 
        const MAV_MODE_FLAG_TEST_ENABLED = 2 ; 
        const MAV_MODE_FLAG_CUSTOM_MODE_ENABLED = 1 ;
    } 
}
impl MavModeFlag {
    pub const DEFAULT: Self = Self::MAV_MODE_FLAG_SAFETY_ARMED;
}
impl Default for MavModeFlag {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive, ToPrimitive, Serialize, Deserialize)]
#[serde(tag = "type")]
#[repr(u32)]
pub enum MavState {
    MAV_STATE_UNINIT = 0,
    MAV_STATE_BOOT = 1,
    MAV_STATE_CALIBRATING = 2,
    MAV_STATE_STANDBY = 3,
    MAV_STATE_ACTIVE = 4,
    MAV_STATE_CRITICAL = 5,
    MAV_STATE_EMERGENCY = 6,
    MAV_STATE_POWEROFF = 7,
    MAV_STATE_FLIGHT_TERMINATION = 8,
}
impl MavState {
    pub const DEFAULT: Self = Self::MAV_STATE_UNINIT;
}
impl Default for MavState {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[repr(u32)]
pub enum MavMessage {
    HEARTBEAT(HEARTBEAT_DATA),
    // ... thousands of others
}

impl Message for MavMessage {
    fn parse(
        version: MavlinkVersion,
        id: u32,
        payload: &[u8]
    ) -> Result<Self, ::mavlink_core::error::ParserError> {
        match id {
            HEARTBEAT_DATA::ID => HEARTBEAT_DATA::deser(version, payload).map(Self::HEARTBEAT),
            // ... thousands of others
            _ => { Err(::mavlink_core::error::ParserError::UnknownMessage { id }) }
        }
    }

    fn message_name(&self) -> &'static str {
        match self {
            Self::HEARTBEAT(..) => HEARTBEAT_DATA::NAME,
            // ... thousands of others
        }
    }

    fn message_id(&self) -> u32 {
        match self {
            Self::HEARTBEAT(..) => HEARTBEAT_DATA::ID,
            // ... thousands of others
        }
    }

    fn message_id_from_name(name: &str) -> Result<u32, &'static str> {
        match name {
            HEARTBEAT_DATA::NAME => HEARTBEAT_DATA::ID,
            // ... thousands of others
            _ => { Err(::mavlink_core::error::ParserError::UnknownMessage { name }) }
        }
    }
    fn default_message_from_id(id: u32) -> Result<Self, &'static str> {
        match id {
            HEARTBEAT_DATA::ID => HEARTBEAT_DATA::default(),
            // ... thousands of others
            _ => { Err(::mavlink_core::error::ParserError::UnknownMessage { id }) }
        }
    }

    fn ser(&self, version: MavlinkVersion, bytes: &mut [u8]) -> usize {
        match self {
            HEARTBEAT_DATA::ID => HEARTBEAT_DATA::ser(),
            // ... thousands of others
        }
    }

    fn extra_crc(id: u32) -> u8 {
        match id {
            HEARTBEAT_DATA::ID => HEARTBEAT_DATA::EXTRA_CRC,
            // ... thousands of others
            _ => { Err(::mavlink_core::error::ParserError::UnknownMessage { id }) }
        }
    }
}
