pub mod codec;
pub mod error;
pub mod mav_types;
pub mod parser;
pub mod rust_mavlink_compatibility;
pub mod v1;
pub mod v2;

use bytes::Bytes;

use v1::{V1Packet, V1_STX};
use v2::{V2Packet, V2_STX};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Packet {
    V1(V1Packet) = V1_STX,
    V2(V2Packet) = V2_STX,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PacketVersion {
    V1 = V1_STX,
    V2 = V2_STX,
}

impl Packet {
    #[inline(always)]
    pub fn bytes(&self) -> &Bytes {
        match self {
            Packet::V1(v1_packet) => v1_packet.bytes(),
            Packet::V2(v2_packet) => v2_packet.bytes(),
        }
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        match self {
            Packet::V1(v1_packet) => v1_packet.as_slice(),
            Packet::V2(v2_packet) => v2_packet.as_slice(),
        }
    }

    #[inline(always)]
    pub fn header(&self) -> &[u8] {
        match self {
            Packet::V1(v1_packet) => v1_packet.header(),
            Packet::V2(v2_packet) => v2_packet.header(),
        }
    }

    #[inline(always)]
    pub fn payload(&self) -> &[u8] {
        match self {
            Packet::V1(v1_packet) => v1_packet.payload(),
            Packet::V2(v2_packet) => v2_packet.payload(),
        }
    }

    #[inline(always)]
    pub fn checksum(&self) -> u16 {
        match self {
            Packet::V1(v1_packet) => v1_packet.checksum(),
            Packet::V2(v2_packet) => v2_packet.checksum(),
        }
    }

    #[inline(always)]
    pub fn checksum_data(&self) -> &[u8] {
        match self {
            Packet::V1(v1_packet) => v1_packet.checksum_data(),
            Packet::V2(v2_packet) => v2_packet.checksum_data(),
        }
    }

    #[inline(always)]
    pub fn packet_size(&self) -> usize {
        match self {
            Packet::V1(v1_packet) => v1_packet.packet_size(),
            Packet::V2(v2_packet) => v2_packet.packet_size(),
        }
    }

    #[inline(always)]
    pub fn stx(&self) -> u8 {
        match self {
            Packet::V1(v1_packet) => v1_packet.stx(),
            Packet::V2(v2_packet) => v2_packet.stx(),
        }
    }

    #[inline(always)]
    pub fn payload_length(&self) -> u8 {
        match self {
            Packet::V1(v1_packet) => v1_packet.payload_length(),
            Packet::V2(v2_packet) => v2_packet.payload_length(),
        }
    }

    #[inline(always)]
    pub fn sequence(&self) -> u8 {
        match self {
            Packet::V1(v1_packet) => v1_packet.sequence(),
            Packet::V2(v2_packet) => v2_packet.sequence(),
        }
    }

    #[inline(always)]
    pub fn system_id(&self) -> u8 {
        match self {
            Packet::V1(v1_packet) => v1_packet.system_id(),
            Packet::V2(v2_packet) => v2_packet.system_id(),
        }
    }

    #[inline(always)]
    pub fn component_id(&self) -> u8 {
        match self {
            Packet::V1(v1_packet) => v1_packet.component_id(),
            Packet::V2(v2_packet) => v2_packet.component_id(),
        }
    }

    #[inline(always)]
    pub fn message_id(&self) -> u32 {
        match self {
            Packet::V1(v1_packet) => v1_packet.message_id() as u32,
            Packet::V2(v2_packet) => v2_packet.message_id(),
        }
    }

    fn header_bytes(&self) -> Bytes {
        match self {
            Packet::V1(v1_packet) => v1_packet.frame_header_bytes(),
            Packet::V2(v2_packet) => v2_packet.frame_header_bytes(),
        }
    }

    fn payload_bytes(&self) -> Bytes {
        match self {
            Packet::V1(v1_packet) => v1_packet.payload_bytes(),
            Packet::V2(v2_packet) => v2_packet.payload_bytes(),
        }
    }

    // // TODO: Maybe change this to from_frame!
    // pub fn try_from_frame(frame: MavFrame, version: PacketVersion) -> Result<Self, DecoderError> {
    //     match version {
    //         PacketVersion::V1 => V1Packet::try_from(frame).map(Self::V1),
    //         PacketVersion::V2 => V2Packet::try_from(frame).map(Self::V2),
    //     }
    // }
}

/// Creates a `MavlinkCodec` with compile-time configuration.
///
/// # Parameters
///
/// - `accept_v1`: Whether to accept MAVLink V1 messages.
/// - `accept_v2`: Whether to accept MAVLink V2 messages.
/// - `drop_invalid_sysid`: Whether to drop messages with zeroed System ID
/// - `drop_invalid_compid`: Whether to drop messages with zeroed Component ID
/// - `skip_crc_validation`: Whether to skip the CRC validation
/// - `drop_incompatible`: Whether to drop messages with unknown Incompatibility Flags
///
/// # Example
///
/// ```
/// use mavlink_codec::{mavlink_codec, codec::MavlinkCodec};
///
/// let codec = mavlink_codec! {
///     accept_v1: true,
///     accept_v2: true,
///     drop_invalid_sysid: false,
///     drop_invalid_compid: false,
///     skip_crc_validation: false,
///     drop_incompatible: false,
/// };
///
/// // Which is equivallent to:
/// let codec = MavlinkCodec::<true, true, false, false, false, false>::default();
/// ```
#[macro_export]
macro_rules! mavlink_codec {
    (
        // Whether to accept MAVLink V1 messages
        accept_v1: $accept_v1:expr,
        // Whether to accept MAVLink V2 messages
        accept_v2: $accept_v2:expr,
        /// Whether to drop messages with zeroed System ID
        drop_invalid_sysid: $drop_invalid_sysid:expr,
        /// Whether to drop messages with zeroed Component ID
        drop_invalid_compid: $drop_invalid_compid:expr,
        /// Whether to skip the CRC validation
        skip_crc_validation: $skip_crc_validation:expr,
        /// Whether to drop messages with unknown Incompatibility Flags
        drop_incompatible: $drop_incompatible:expr,
    ) => {
        $crate::codec::MavlinkCodec::<
            { $accept_v1 },
            { $accept_v2 },
            { $drop_invalid_sysid },
            { $drop_invalid_compid },
            { $skip_crc_validation },
            { $drop_incompatible },
        > {
            state: $crate::codec::CodecState::default(),
        }
    };
}
