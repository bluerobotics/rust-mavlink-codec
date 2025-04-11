// use bitflags::bitflags;
// use classic_bitfield::bitfield_enum;
use serde_derive::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MavType {
    #[default]
    MavTypeGeneric = 0,
    MavTypeFixedWing = 1,
    MavTypeQuadrotor = 2,
    MavTypeCoaxial = 3,
    MavTypeHelicopter = 4,
    MavTypeAntennaTracker = 5,
    MavTypeGcs = 6,
    MavTypeAirship = 7,
    MavTypeFreeBalloon = 8,
    MavTypeRocket = 9,
    MavTypeGroundRover = 10,
    MavTypeSurfaceBoat = 11,
    MavTypeSubmarine = 12,
    MavTypeHexarotor = 13,
    MavTypeOctorotor = 14,
    MavTypeTricopter = 15,
    MavTypeFlappingWing = 16,
    MavTypeKite = 17,
    MavTypeOnboardController = 18,
    MavTypeVtolTailsitterDuorotor = 19,
    MavTypeVtolTailsitterQuadrotor = 20,
    MavTypeVtolTiltrotor = 21,
    MavTypeVtolFixedrotor = 22,
    MavTypeVtolTailsitter = 23,
    MavTypeVtolTiltwing = 24,
    MavTypeVtolReserved5 = 25,
    MavTypeGimbal = 26,
    MavTypeAdsb = 27,
    MavTypeParafoil = 28,
    MavTypeDodecarotor = 29,
    MavTypeCamera = 30,
    MavTypeChargingStation = 31,
    MavTypeFlarm = 32,
    MavTypeServo = 33,
    MavTypeOdid = 34,
    MavTypeDecarotor = 35,
    MavTypeBattery = 36,
    MavTypeParachute = 37,
    MavTypeLog = 38,
    MavTypeOsd = 39,
    MavTypeImu = 40,
    MavTypeGps = 41,
    MavTypeWinch = 42,
}

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MavAutopilot {
    #[default]
    MavAutopilotGeneric = 0,
    MavAutopilotReserved = 1,
    MavAutopilotSlugs = 2,
    MavAutopilotArdupilotmega = 3,
    MavAutopilotOpenpilot = 4,
    MavAutopilotGenericWaypointsOnly = 5,
    MavAutopilotGenericWaypointsAndSimpleNavigationOnly = 6,
    MavAutopilotGenericMissionFull = 7,
    MavAutopilotInvalid = 8,
    MavAutopilotPpz = 9,
    MavAutopilotUdb = 10,
    MavAutopilotFp = 11,
    MavAutopilotPx4 = 12,
    MavAutopilotSmaccmpilot = 13,
    MavAutopilotAutoquad = 14,
    MavAutopilotArmazila = 15,
    MavAutopilotAerob = 16,
    MavAutopilotAsluav = 17,
    MavAutopilotSmartap = 18,
    MavAutopilotAirrails = 19,
    MavAutopilotReflex = 20,
}

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MavState {
    #[default]
    MavStateUninit = 0,
    MavStateBoot = 1,
    MavStateCalibrating = 2,
    MavStateStandby = 3,
    MavStateActive = 4,
    MavStateCritical = 5,
    MavStateEmergency = 6,
    MavStatePoweroff = 7,
    MavStateFlightTermination = 8,
}

// #[bitfield_enum(as u8)]
// #[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
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

// bitflags! {
//     #[derive (Debug, Clone, PartialEq, Serialize , Deserialize)]
//     pub struct MavModeFlag : u8 {
//         const MAV_MODE_FLAG_SAFETY_ARMED = 128 ;
//         const MAV_MODE_FLAG_MANUAL_INPUT_ENABLED = 64 ;
//         const MAV_MODE_FLAG_HIL_ENABLED = 32 ;
//         const MAV_MODE_FLAG_STABILIZE_ENABLED = 16 ;
//         const MAV_MODE_FLAG_GUIDED_ENABLED = 8 ;
//         const MAV_MODE_FLAG_AUTO_ENABLED = 4 ;
//         const MAV_MODE_FLAG_TEST_ENABLED = 2 ;
//         const MAV_MODE_FLAG_CUSTOM_MODE_ENABLED = 1 ;
//     }
// }
// impl Default for MavModeFlag {
//     fn default() -> Self {
//         Self::MAV_MODE_FLAG_SAFETY_ARMED
//     }
// }
