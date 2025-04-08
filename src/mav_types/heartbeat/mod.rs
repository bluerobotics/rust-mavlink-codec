pub mod serde_impl;

use bytes::Bytes;

use super::{MavAutopilot, MavState, MavType};

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HeartbeatMessage {
    pub(crate) buffer: bytes::Bytes,
}

impl HeartbeatMessage {
    pub const ID: u32 = 0;
    pub const LEN: u16 = 9;
    pub const CRC: u32 = 300;

    #[inline(always)]
    pub fn new(bytes: Bytes) -> Self {
        Self { buffer: bytes }
    }

    #[inline(always)]
    pub fn bytes(&self) -> &Bytes {
        &self.buffer
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer[..]
    }

    #[inline(always)]
    pub fn custom_mode(&self) -> u32 {
        if self.buffer.len() < 4 {
            return 0;
        }

        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.buffer[..4]);
        u32::from_le_bytes(bytes)
    }

    #[inline(always)]
    pub fn mav_type(&self) -> MavType {
        const OFFSET: usize = 4;
        if self.buffer.len() <= OFFSET {
            return MavType::default();
        }

        let raw = self.buffer[OFFSET];
        unsafe { std::mem::transmute(raw) }
    }

    #[inline(always)]
    pub fn autopilot(&self) -> MavAutopilot {
        const OFFSET: usize = 5;
        if self.buffer.len() <= OFFSET {
            return MavAutopilot::default();
        }

        let raw = self.buffer[OFFSET];
        unsafe { std::mem::transmute(raw) }
    }

    #[inline(always)]
    pub fn base_mode(&self) -> u8 {
        const OFFSET: usize = 6;
        if self.buffer.len() <= OFFSET {
            return 0;
        }

        self.buffer[OFFSET]
    }

    #[inline(always)]
    pub fn system_status(&self) -> MavState {
        const OFFSET: usize = 7;
        if self.buffer.len() <= OFFSET {
            return MavState::default();
        }

        let raw = self.buffer[OFFSET];
        unsafe { std::mem::transmute(raw) }
    }

    #[inline(always)]
    pub fn mavlink_version(&self) -> u8 {
        const OFFSET: usize = 8;
        if self.buffer.len() <= OFFSET {
            return 0;
        }

        self.buffer[OFFSET]
    }
}

impl std::fmt::Debug for HeartbeatMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HeartbeatMessage")
            .field("custom_mode", &self.custom_mode())
            .field("mav_type", &self.mav_type())
            .field("autopilot", &self.autopilot())
            .field("base_mode", &self.base_mode())
            .field("system_status", &self.system_status())
            .field("mavlink_version", &self.mavlink_version())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use mavlink::Message as _;

    #[test]
    fn test() {
        // 1. Create a heartbeat message with All non-zero values using rust-mavlink
        let original_heartbeat_message_data = mavlink::ardupilotmega::HEARTBEAT_DATA {
            custom_mode: std::u32::MAX,
            mavtype: mavlink::ardupilotmega::MavType::MAV_TYPE_FIXED_WING,
            autopilot: mavlink::ardupilotmega::MavAutopilot::MAV_AUTOPILOT_ARDUPILOTMEGA,
            base_mode: mavlink::ardupilotmega::MavModeFlag::all(),
            system_status: mavlink::ardupilotmega::MavState::MAV_STATE_STANDBY,
            mavlink_version: 0x03,
        };
        dbg!(&original_heartbeat_message_data);

        let mut slice = [0u8; mavlink::ardupilotmega::HEARTBEAT_DATA::ENCODED_LEN];
        mavlink::ardupilotmega::MavMessage::ser(
            &mavlink::ardupilotmega::MavMessage::HEARTBEAT(original_heartbeat_message_data.clone()),
            mavlink::MavlinkVersion::V2,
            &mut slice,
        );
        dbg!(&slice);

        let parsed_heartbeat_message_data = HeartbeatMessage::new(Bytes::copy_from_slice(&slice));
        dbg!(&parsed_heartbeat_message_data);

        // 4. Now, compare all fields
        assert_eq!(
            parsed_heartbeat_message_data.custom_mode(),
            original_heartbeat_message_data.custom_mode
        );
        assert_eq!(
            parsed_heartbeat_message_data.mav_type() as u32,
            original_heartbeat_message_data.mavtype as u32
        );
        assert_eq!(
            parsed_heartbeat_message_data.autopilot() as u8,
            original_heartbeat_message_data.autopilot as u8
        );
        assert_eq!(
            parsed_heartbeat_message_data.base_mode(),
            original_heartbeat_message_data.base_mode.bits()
        );
        assert_eq!(
            parsed_heartbeat_message_data.system_status() as u32,
            original_heartbeat_message_data.system_status as u32
        );
        assert_eq!(
            parsed_heartbeat_message_data.mavlink_version(),
            original_heartbeat_message_data.mavlink_version
        );
    }
}
