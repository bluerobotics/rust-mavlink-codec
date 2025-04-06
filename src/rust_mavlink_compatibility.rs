use bytes::Bytes;

use crate::{v1::V1Packet, v2::V2Packet, Packet};

impl From<mavlink::MAVLinkV1MessageRaw> for Packet {
    fn from(value: mavlink::MAVLinkV1MessageRaw) -> Self {
        Self::V1(V1Packet::from(value))
    }
}

impl From<mavlink::MAVLinkV2MessageRaw> for Packet {
    fn from(value: mavlink::MAVLinkV2MessageRaw) -> Self {
        Self::V2(V2Packet::from(value))
    }
}

impl TryFrom<Packet> for mavlink::MAVLinkV1MessageRaw {
    type Error = mavlink::error::MessageReadError;

    /// A convenient rust-mavlink compatibility layer
    /// warning: this has a bad performance because we don't have access to the mutable internal buffer of rust-mavlink's raw messages    fn try_from(value: Packet) -> Result<Self, Self::Error> {
    fn try_from(value: Packet) -> Result<Self, Self::Error> {
        match value {
            Packet::V1(v1_packet) => mavlink::MAVLinkV1MessageRaw::try_from(v1_packet),
            Packet::V2(_) => Err(mavlink::error::MessageReadError::Io(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Expected V1 Message",
            ))),
        }
    }
}

impl TryFrom<Packet> for mavlink::MAVLinkV2MessageRaw {
    type Error = mavlink::error::MessageReadError;

    /// A convenient rust-mavlink compatibility layer
    /// warning: this has a bad performance because we don't have access to the mutable internal buffer of rust-mavlink's raw messages    fn try_from(value: Packet) -> Result<Self, Self::Error> {
    fn try_from(value: Packet) -> Result<Self, Self::Error> {
        match value {
            Packet::V1(_) => Err(mavlink::error::MessageReadError::Io(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Expected V2 Message",
            ))),
            Packet::V2(v2_packet) => mavlink::MAVLinkV2MessageRaw::try_from(v2_packet),
        }
    }
}

impl From<mavlink::MAVLinkV1MessageRaw> for V1Packet {
    fn from(value: mavlink::MAVLinkV1MessageRaw) -> Self {
        Self {
            buffer: Bytes::copy_from_slice(value.raw_bytes()),
        }
    }
}

impl TryFrom<V1Packet> for mavlink::MAVLinkV1MessageRaw {
    type Error = mavlink::error::MessageReadError;

    /// A convenient rust-mavlink compatibility layer
    fn try_from(value: V1Packet) -> Result<Self, Self::Error> {
        let src_s = value.as_slice();
        let src_s_ptr = src_s.as_ptr();
        let src_s_len = src_s.len();

        let mut message = std::mem::MaybeUninit::<mavlink::MAVLinkV1MessageRaw>::uninit();
        let dst_s_ptr = message.as_mut_ptr() as *mut u8;

        unsafe {
            let remaining_len = 280 - src_s_len;
            if remaining_len > 0 {
                std::ptr::write_bytes(dst_s_ptr.add(src_s_len), 0, remaining_len);
            }

            std::ptr::copy_nonoverlapping(src_s_ptr, dst_s_ptr, src_s_len);
            Ok(message.assume_init())
        }
    }
}

impl From<mavlink::MAVLinkV2MessageRaw> for V2Packet {
    fn from(value: mavlink::MAVLinkV2MessageRaw) -> Self {
        Self {
            buffer: Bytes::copy_from_slice(value.raw_bytes()),
        }
    }
}

impl TryFrom<V2Packet> for mavlink::MAVLinkV2MessageRaw {
    type Error = mavlink::error::MessageReadError;

    /// A convenient rust-mavlink compatibility layer
    fn try_from(value: V2Packet) -> Result<Self, Self::Error> {
        let src_s = value.as_slice();
        let src_s_ptr = src_s.as_ptr();
        let src_s_len = src_s.len();

        let mut message = std::mem::MaybeUninit::<mavlink::MAVLinkV2MessageRaw>::uninit();
        let dst_s_ptr = message.as_mut_ptr() as *mut u8;

        unsafe {
            let remaining_len = 280 - src_s_len;
            if remaining_len > 0 {
                std::ptr::write_bytes(dst_s_ptr.add(src_s_len), 0, remaining_len);
            }

            std::ptr::copy_nonoverlapping(src_s_ptr, dst_s_ptr, src_s_len);
            Ok(message.assume_init())
        }
    }
}
