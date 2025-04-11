use bytes::Bytes;

use crate::mav_types::mav_message::MavMessageDef;

pub mod serde_impl;

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcChannelsMessage {
    pub(crate) buffer: bytes::Bytes,
}

impl MavMessageDef for RcChannelsMessage {
    const ID: u32 = 65;
    const LEN: u16 = 42;
    // Using u32 for CRC to match the Heartbeat example's trait definition,
    // even though the source example provides u8.
    const CRC: u32 = 118;

    #[inline(always)]
    fn new(bytes: Bytes) -> Self {
        Self { buffer: bytes }
    }

    #[inline(always)]
    fn bytes(&self) -> &Bytes {
        &self.buffer
    }

    #[inline(always)]
    fn as_slice(&self) -> &[u8] {
        &self.buffer[..]
    }
}

// Helper macro to generate accessor methods for u16 channels
macro_rules! impl_chan_raw_accessor {
    ($name:ident, $offset:expr) => {
        #[inline(always)]
        pub fn $name(&self) -> u16 {
            const OFFSET: usize = $offset;
            if self.buffer.len() < OFFSET + 2 {
                return 0;
            }
            let mut bytes = [0u8; 2];
            bytes.copy_from_slice(&self.buffer[OFFSET..OFFSET + 2]);
            u16::from_le_bytes(bytes)
        }
    };
}

impl RcChannelsMessage {
    #[inline(always)]
    pub fn time_boot_ms(&self) -> u32 {
        const OFFSET: usize = 0;
        if self.buffer.len() < OFFSET + 4 {
            return 0;
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.buffer[OFFSET..OFFSET + 4]);
        u32::from_le_bytes(bytes)
    }

    impl_chan_raw_accessor!(chan1_raw, 4);
    impl_chan_raw_accessor!(chan2_raw, 6);
    impl_chan_raw_accessor!(chan3_raw, 8);
    impl_chan_raw_accessor!(chan4_raw, 10);
    impl_chan_raw_accessor!(chan5_raw, 12);
    impl_chan_raw_accessor!(chan6_raw, 14);
    impl_chan_raw_accessor!(chan7_raw, 16);
    impl_chan_raw_accessor!(chan8_raw, 18);
    impl_chan_raw_accessor!(chan9_raw, 20);
    impl_chan_raw_accessor!(chan10_raw, 22);
    impl_chan_raw_accessor!(chan11_raw, 24);
    impl_chan_raw_accessor!(chan12_raw, 26);
    impl_chan_raw_accessor!(chan13_raw, 28);
    impl_chan_raw_accessor!(chan14_raw, 30);
    impl_chan_raw_accessor!(chan15_raw, 32);
    impl_chan_raw_accessor!(chan16_raw, 34);
    impl_chan_raw_accessor!(chan17_raw, 36);
    impl_chan_raw_accessor!(chan18_raw, 38);

    #[inline(always)]
    pub fn chancount(&self) -> u8 {
        const OFFSET: usize = 40;
        if self.buffer.len() <= OFFSET {
            return 0;
        }
        self.buffer[OFFSET]
    }

    #[inline(always)]
    pub fn rssi(&self) -> u8 {
        const OFFSET: usize = 41;
        if self.buffer.len() <= OFFSET {
            return 0;
        }
        self.buffer[OFFSET]
    }
}

impl std::fmt::Debug for RcChannelsMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RcChannelsMessage")
            .field("time_boot_ms", &self.time_boot_ms())
            .field("chan1_raw", &self.chan1_raw())
            .field("chan2_raw", &self.chan2_raw())
            .field("chan3_raw", &self.chan3_raw())
            .field("chan4_raw", &self.chan4_raw())
            .field("chan5_raw", &self.chan5_raw())
            .field("chan6_raw", &self.chan6_raw())
            .field("chan7_raw", &self.chan7_raw())
            .field("chan8_raw", &self.chan8_raw())
            .field("chan9_raw", &self.chan9_raw())
            .field("chan10_raw", &self.chan10_raw())
            .field("chan11_raw", &self.chan11_raw())
            .field("chan12_raw", &self.chan12_raw())
            .field("chan13_raw", &self.chan13_raw())
            .field("chan14_raw", &self.chan14_raw())
            .field("chan15_raw", &self.chan15_raw())
            .field("chan16_raw", &self.chan16_raw())
            .field("chan17_raw", &self.chan17_raw())
            .field("chan18_raw", &self.chan18_raw())
            .field("chancount", &self.chancount())
            .field("rssi", &self.rssi())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use mavlink::Message as _;

    #[test]
    fn test_rc_channels_parsing() {
        // 1. Create an RC_CHANNELS message with non-zero values using rust-mavlink
        let original_rc_data = mavlink::ardupilotmega::RC_CHANNELS_DATA {
            time_boot_ms: 123456789,
            chan1_raw: 1100,
            chan2_raw: 1200,
            chan3_raw: 1300,
            chan4_raw: 1400,
            chan5_raw: 1500,
            chan6_raw: 1600,
            chan7_raw: 1700,
            chan8_raw: 1800,
            chan9_raw: 1900,
            chan10_raw: 1110,
            chan11_raw: 1120,
            chan12_raw: 1130,
            chan13_raw: 1140,
            chan14_raw: 1150,
            chan15_raw: 1160,
            chan16_raw: 1170,
            chan17_raw: 1180,
            chan18_raw: 1190,
            chancount: 18,
            rssi: 200,
        };
        dbg!(&original_rc_data);

        // 2. Serialize it
        let mut slice = [0u8; mavlink::ardupilotmega::RC_CHANNELS_DATA::ENCODED_LEN];
        mavlink::ardupilotmega::MavMessage::ser(
            &mavlink::ardupilotmega::MavMessage::RC_CHANNELS(original_rc_data.clone()),
            mavlink::MavlinkVersion::V2, // Or V1, shouldn't matter for fixed-size payload
            &mut slice,
        );
        dbg!(&slice);

        // 3. Parse it using the new implementation
        let parsed_rc_message = RcChannelsMessage::new(Bytes::copy_from_slice(&slice));
        dbg!(&parsed_rc_message);

        // 4. Compare all fields
        assert_eq!(
            parsed_rc_message.time_boot_ms(),
            original_rc_data.time_boot_ms
        );
        assert_eq!(parsed_rc_message.chan1_raw(), original_rc_data.chan1_raw);
        assert_eq!(parsed_rc_message.chan2_raw(), original_rc_data.chan2_raw);
        assert_eq!(parsed_rc_message.chan3_raw(), original_rc_data.chan3_raw);
        assert_eq!(parsed_rc_message.chan4_raw(), original_rc_data.chan4_raw);
        assert_eq!(parsed_rc_message.chan5_raw(), original_rc_data.chan5_raw);
        assert_eq!(parsed_rc_message.chan6_raw(), original_rc_data.chan6_raw);
        assert_eq!(parsed_rc_message.chan7_raw(), original_rc_data.chan7_raw);
        assert_eq!(parsed_rc_message.chan8_raw(), original_rc_data.chan8_raw);
        assert_eq!(parsed_rc_message.chan9_raw(), original_rc_data.chan9_raw);
        assert_eq!(parsed_rc_message.chan10_raw(), original_rc_data.chan10_raw);
        assert_eq!(parsed_rc_message.chan11_raw(), original_rc_data.chan11_raw);
        assert_eq!(parsed_rc_message.chan12_raw(), original_rc_data.chan12_raw);
        assert_eq!(parsed_rc_message.chan13_raw(), original_rc_data.chan13_raw);
        assert_eq!(parsed_rc_message.chan14_raw(), original_rc_data.chan14_raw);
        assert_eq!(parsed_rc_message.chan15_raw(), original_rc_data.chan15_raw);
        assert_eq!(parsed_rc_message.chan16_raw(), original_rc_data.chan16_raw);
        // Note: rust-mavlink might pad unused channels (like 17/18 if chancount < 18) with different values (often u16::MAX or 0).
        // Test according to actual observed behavior or spec if needed. Here we assume chancount=18 means all are valid.
        assert_eq!(parsed_rc_message.chan17_raw(), original_rc_data.chan17_raw);
        assert_eq!(parsed_rc_message.chan18_raw(), original_rc_data.chan18_raw);
        assert_eq!(parsed_rc_message.chancount(), original_rc_data.chancount);
        assert_eq!(parsed_rc_message.rssi(), original_rc_data.rssi);
    }
}
