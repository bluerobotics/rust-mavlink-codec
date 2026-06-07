use bytes::{Buf, BufMut, Bytes, BytesMut};
use log::trace;
use mavlink::calculate_crc;
use tokio_util::codec::{Decoder, Encoder};

use crate::{
    error::DecoderError,
    v1::{self, V1Packet, V1_STX},
    v2::{self, V2Packet, MAVLINK_SUPPORTED_IFLAGS, V2_STX},
    Packet,
};

/// MAVLink packet codec whose behavior is selected at compile time through
/// const-generic toggles.
///
/// The toggles are, in order:
///
/// * `ACCEPT_V1` -- accept MAVLink v1 frames.
/// * `ACCEPT_V2` -- accept MAVLink v2 frames.
/// * `DROP_INVALID_SYSID` -- reject frames whose system id equals zero.
/// * `DROP_INVALID_COMPID` -- reject frames whose component id equals zero.
/// * `SKIP_CRC_VALIDATION` -- skip **only** the CRC computation step.
/// * `DROP_INCOMPATIBLE` -- reject v2 frames with unsupported incompat flags.
/// * `VERIFY_SIGNATURE` -- require a valid MAVLink2 signature on accepted v2 frames; reject all v1 frames.
#[derive(Default)]
pub struct MavlinkCodec<
    const ACCEPT_V1: bool,
    const ACCEPT_V2: bool,
    const DROP_INVALID_SYSID: bool,
    const DROP_INVALID_COMPID: bool,
    const SKIP_CRC_VALIDATION: bool,
    const DROP_INCOMPATIBLE: bool,
    const VERIFY_SIGNATURE: bool,
> {
    pub state: CodecState,
    signing: Option<mavlink::SigningData>,
}

impl<
        const ACCEPT_V1: bool,
        const ACCEPT_V2: bool,
        const DROP_INVALID_SYSID: bool,
        const DROP_INVALID_COMPID: bool,
        const SKIP_CRC_VALIDATION: bool,
        const DROP_INCOMPATIBLE: bool,
        const VERIFY_SIGNATURE: bool,
    > std::fmt::Debug
    for MavlinkCodec<
        ACCEPT_V1,
        ACCEPT_V2,
        DROP_INVALID_SYSID,
        DROP_INVALID_COMPID,
        SKIP_CRC_VALIDATION,
        DROP_INCOMPATIBLE,
        VERIFY_SIGNATURE,
    >
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MavlinkCodec")
            .field("state", &self.state)
            .field("signing", &self.signing.as_ref().map(|_| ".."))
            .finish()
    }
}

impl<
        const ACCEPT_V1: bool,
        const ACCEPT_V2: bool,
        const DROP_INVALID_SYSID: bool,
        const DROP_INVALID_COMPID: bool,
        const SKIP_CRC_VALIDATION: bool,
        const DROP_INCOMPATIBLE: bool,
        const VERIFY_SIGNATURE: bool,
    >
    MavlinkCodec<
        ACCEPT_V1,
        ACCEPT_V2,
        DROP_INVALID_SYSID,
        DROP_INVALID_COMPID,
        SKIP_CRC_VALIDATION,
        DROP_INCOMPATIBLE,
        VERIFY_SIGNATURE,
    >
{
    pub fn with_signing(signing: mavlink::SigningData) -> Self {
        Self {
            state: CodecState::default(),
            signing: Some(signing),
        }
    }
}

#[derive(Debug, Default)]
pub enum CodecState {
    #[default]
    WaitingForStx,
    WaitingV1PacketHeader,
    WaitingV2PacketHeader,
    ValidatingV1Packet {
        packet_size: usize,
    },
    ValidatingV2Packet {
        packet_size: usize,
    },
    CopyV1Packet {
        packet_size: usize,
    },
    CopyV2Packet {
        packet_size: usize,
    },
}

impl<
        const ACCEPT_V1: bool,
        const ACCEPT_V2: bool,
        const DROP_INVALID_SYSID: bool,
        const DROP_INVALID_COMPID: bool,
        const SKIP_CRC_VALIDATION: bool,
        const DROP_INCOMPATIBLE: bool,
        const VERIFY_SIGNATURE: bool,
    > Decoder
    for MavlinkCodec<
        ACCEPT_V1,
        ACCEPT_V2,
        DROP_INVALID_SYSID,
        DROP_INVALID_COMPID,
        SKIP_CRC_VALIDATION,
        DROP_INCOMPATIBLE,
        VERIFY_SIGNATURE,
    >
{
    type Item = Result<Packet, DecoderError>;
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        trace!("Decoding: {:?}", &buf[..]);

        loop {
            match self.state {
                CodecState::WaitingForStx => {
                    trace!("Waitig for STX...");

                    if buf.is_empty() {
                        trace!(
                            "Not enough data, buf.len: {:?}, buf.capacity: {:?}",
                            buf.len(),
                            buf.capacity()
                        );
                        return Ok(None);
                    }

                    match buf[0] {
                        V1_STX if ACCEPT_V1 => self.state = CodecState::WaitingV1PacketHeader,
                        V2_STX if ACCEPT_V2 => self.state = CodecState::WaitingV2PacketHeader,
                        _ => {
                            trace!("Invalid STX byte: {}", buf[0]);
                            buf.advance(V1Packet::STX_SIZE);
                            continue;
                        }
                    }
                }
                // V1 Codec
                CodecState::WaitingV1PacketHeader if ACCEPT_V1 => {
                    if buf.len() < V1Packet::HEADER_SIZE {
                        trace!(
                            "Not enough data, buf.len: {:?}, buf.capacity: {:?}",
                            buf.len(),
                            buf.capacity()
                        );
                        return Ok(None);
                    }

                    let packet_size = v1::packet_size(buf);
                    self.state = CodecState::ValidatingV1Packet { packet_size };
                }
                CodecState::ValidatingV1Packet { packet_size } if ACCEPT_V1 => {
                    if buf.len() < packet_size {
                        trace!(
                            "Not enough data, buf.len: {:?}, buf.capacity: {:?}",
                            buf.len(),
                            buf.capacity()
                        );
                        return Ok(None);
                    }

                    // System ID validation
                    if DROP_INVALID_SYSID {
                        let sysid = *v1::sysid(buf);
                        if sysid == 0 {
                            trace!("Invalid SystemID: {sysid:?}. Data: {:?}", &buf[..]);

                            buf.advance(V1Packet::STX_SIZE); // Discard this STX
                            self.state = CodecState::WaitingForStx;

                            return Ok(Some(Err(DecoderError::InvalidSystemID { sysid })));
                        }
                    }

                    // Component ID validation
                    if DROP_INVALID_COMPID {
                        let compid = *v1::compid(buf);
                        if compid == 0 {
                            trace!("Invalid SystemID: {compid:?}. Data: {:?}", &buf[..]);

                            buf.advance(V1Packet::STX_SIZE); // Discard this STX
                            self.state = CodecState::WaitingForStx;

                            return Ok(Some(Err(DecoderError::InvalidComponentID { compid })));
                        }
                    }

                    // CRC Validation
                    if !SKIP_CRC_VALIDATION {
                        let msgid = *v1::msgid(buf) as u32;
                        let Some(extra_crc) = get_extra_crc(msgid) else {
                            trace!("Unknown message ID {msgid:?}. Data: {:?}", &buf[..]);

                            buf.advance(V1Packet::STX_SIZE); // Discard this STX
                            self.state = CodecState::WaitingForStx;

                            return Ok(Some(Err(DecoderError::UnknownMessageID { msgid })));
                        };
                        let checksum_data = v1::checksum_data(buf);
                        let calculated_crc = calculate_crc(checksum_data, extra_crc);

                        let expected_crc = v1::checksum(buf);
                        if calculated_crc.ne(&expected_crc) {
                            trace!(
                                "Invalid CRC: expected: {expected_crc:?}, calculated: {calculated_crc:?}. checksum_data: {checksum_data:?}"
                            );

                            buf.advance(V1Packet::STX_SIZE); // Discard this STX
                            self.state = CodecState::WaitingForStx;

                            return Ok(Some(Err(DecoderError::InvalidCRC {
                                expected_crc,
                                calculated_crc,
                            })));
                        }
                    } else {
                        trace!("CRC Validation skipped.");
                    }

                    // Signature Verification
                    if VERIFY_SIGNATURE {
                        buf.advance(V1Packet::STX_SIZE);
                        self.state = CodecState::WaitingForStx;

                        return Ok(Some(Err(DecoderError::InvalidSignature)));
                    } else {
                        trace!("Signature Verification skipped.");
                    }

                    self.state = CodecState::CopyV1Packet { packet_size };
                }
                CodecState::CopyV1Packet { packet_size } if ACCEPT_V1 => {
                    let buf_packet = buf.split_to(packet_size);
                    let packet = V1Packet {
                        buffer: buf_packet.freeze(),
                    };

                    self.state = CodecState::WaitingForStx;
                    return Ok(Some(Ok(Packet::V1(packet))));
                }
                // V2 Codec
                CodecState::WaitingV2PacketHeader if ACCEPT_V2 => {
                    if buf.len() < V2Packet::HEADER_SIZE {
                        trace!(
                            "Not enough data, buf.len: {:?}, buf.capacity: {:?}",
                            buf.len(),
                            buf.capacity()
                        );
                        return Ok(None);
                    }

                    if DROP_INCOMPATIBLE {
                        let incompat_flags = *v2::incompat_flags(buf);
                        if incompat_flags & !MAVLINK_SUPPORTED_IFLAGS > 0 {
                            buf.advance(V2Packet::STX_SIZE); // Discard this STX
                            self.state = CodecState::WaitingForStx;

                            return Ok(Some(Err(DecoderError::Incompatible { incompat_flags })));
                        }
                    }

                    let packet_size = v2::packet_size(buf);
                    self.state = CodecState::ValidatingV2Packet { packet_size };
                }
                CodecState::ValidatingV2Packet { packet_size } if ACCEPT_V2 => {
                    if buf.len() < packet_size {
                        trace!(
                            "Not enough data, buf.len: {:?}, buf.capacity: {:?}",
                            buf.len(),
                            buf.capacity()
                        );
                        return Ok(None);
                    }

                    // System ID validation
                    if DROP_INVALID_SYSID {
                        let sysid = *v2::sysid(buf);
                        if sysid == 0 {
                            trace!("Invalid SystemID: {sysid:?}. Data: {:?}", &buf[..]);

                            buf.advance(V2Packet::STX_SIZE); // Discard this STX
                            self.state = CodecState::WaitingForStx;

                            return Ok(Some(Err(DecoderError::InvalidSystemID { sysid })));
                        }
                    }

                    // Component ID validation
                    if DROP_INVALID_COMPID {
                        let compid = *v2::compid(buf);
                        if compid == 0 {
                            trace!("Invalid SystemID: {compid:?}. Data: {:?}", &buf[..]);

                            buf.advance(V2Packet::STX_SIZE); // Discard this STX
                            self.state = CodecState::WaitingForStx;

                            return Ok(Some(Err(DecoderError::InvalidComponentID { compid })));
                        }
                    }

                    // CRC Validation
                    if !SKIP_CRC_VALIDATION {
                        let msgid = v2::msgid(buf);
                        let Some(extra_crc) = get_extra_crc(msgid) else {
                            trace!("Unknown message ID {msgid:?}. Data: {:?}", &buf[..]);

                            buf.advance(V2Packet::STX_SIZE); // Discard this STX
                            self.state = CodecState::WaitingForStx;

                            return Ok(Some(Err(DecoderError::UnknownMessageID { msgid })));
                        };
                        let checksum_data = v2::checksum_data(buf);
                        let calculated_crc = calculate_crc(checksum_data, extra_crc);

                        let expected_crc = v2::checksum(buf);
                        if calculated_crc.ne(&expected_crc) {
                            trace!(
                                "Invalid CRC: expected: {expected_crc:?}, calculated: {calculated_crc:?}. checksum_data: {checksum_data:?}"
                            );

                            buf.advance(V2Packet::STX_SIZE); // Discard this STX
                            self.state = CodecState::WaitingForStx;

                            return Ok(Some(Err(DecoderError::InvalidCRC {
                                expected_crc,
                                calculated_crc,
                            })));
                        }
                    } else {
                        trace!("CRC Validation skipped.");
                    }

                    // Signature Verification
                    if VERIFY_SIGNATURE {
                        let v2_packet = V2Packet {
                            buffer: Bytes::copy_from_slice(&buf[..packet_size]),
                        };
                        let signature_ok = self.signing.as_ref().is_some_and(|signing| {
                            mavlink::MAVLinkV2MessageRaw::try_from(v2_packet)
                                .map(|raw| signing.verify_signature(&raw))
                                .unwrap_or(false)
                        });
                        if !signature_ok {
                            buf.advance(V2Packet::STX_SIZE);
                            self.state = CodecState::WaitingForStx;

                            return Ok(Some(Err(DecoderError::InvalidSignature)));
                        }
                    } else {
                        trace!("Signature Verification skipped.");
                    }

                    self.state = CodecState::CopyV2Packet { packet_size };
                }
                CodecState::CopyV2Packet { packet_size } if ACCEPT_V2 => {
                    let buf_packet = buf.split_to(packet_size);

                    let packet = V2Packet {
                        buffer: buf_packet.freeze(),
                    };

                    self.state = CodecState::WaitingForStx;
                    return Ok(Some(Ok(Packet::V2(packet))));
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
}

impl<
        const ACCEPT_V1: bool,
        const ACCEPT_V2: bool,
        const DROP_INVALID_SYSID: bool,
        const DROP_INVALID_COMPID: bool,
        const SKIP_CRC_VALIDATION: bool,
        const DROP_INCOMPATIBLE: bool,
        const VERIFY_SIGNATURE: bool,
    > Encoder<Packet>
    for MavlinkCodec<
        ACCEPT_V1,
        ACCEPT_V2,
        DROP_INVALID_SYSID,
        DROP_INVALID_COMPID,
        SKIP_CRC_VALIDATION,
        DROP_INCOMPATIBLE,
        VERIFY_SIGNATURE,
    >
{
    type Error = std::io::Error;

    fn encode(&mut self, packet: Packet, buf: &mut BytesMut) -> Result<(), Self::Error> {
        trace!("encoding...");
        match packet {
            Packet::V1(v1_packet) if ACCEPT_V1 => {
                trace!("v1 package written");
                buf.put(v1_packet.as_slice());
            }
            Packet::V2(v2_packet) if ACCEPT_V2 => {
                trace!("v2 package written");
                buf.put(v2_packet.as_slice());
            }
            _ => {
                trace!("unsupported package version");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unsupported packet version",
                ));
            }
        }

        Ok(())
    }
}

#[inline(always)]
pub fn get_extra_crc(msgid: u32) -> Option<u8> {
    use mavlink::Message;

    Some(mavlink::ardupilotmega::MavMessage::extra_crc(msgid))
}

#[cfg(test)]
mod test_encode {
    use super::*;
    use mavlink::{
        ardupilotmega::MavMessage, MAVLinkV1MessageRaw, MAVLinkV2MessageRaw, MavHeader, Message,
    };

    #[test]
    fn test_encode_v1() {
        let mut codec = MavlinkCodec::<true, true, false, false, false, false, false>::default();

        let v1_packet = {
            let header = MavHeader {
                system_id: 1,
                component_id: 1,
                sequence: 0,
            };

            let message_data = MavMessage::default_message_from_id(0).unwrap(); // Heartbeat message
            let mut raw_v1_message = MAVLinkV1MessageRaw::new();
            raw_v1_message.serialize_message(header, &message_data);
            V1Packet::from(raw_v1_message)
        };
        let mut buf = BytesMut::with_capacity(V1Packet::MAX_PACKET_SIZE);

        codec
            .encode(Packet::V1(v1_packet.clone()), &mut buf)
            .unwrap();

        assert_eq!(&buf[..v1_packet.packet_size()], v1_packet.as_slice())
    }

    #[test]
    fn test_encode_v2() {
        let mut codec = MavlinkCodec::<true, true, false, false, false, false, false>::default();

        let v2_packet = {
            let header = MavHeader {
                system_id: 1,
                component_id: 1,
                sequence: 0,
            };

            let message_data = MavMessage::default_message_from_id(0).unwrap(); // Heartbeat message
            let mut raw_v2_message = MAVLinkV2MessageRaw::new();
            raw_v2_message.serialize_message(header, &message_data);
            V2Packet::from(raw_v2_message)
        };

        let mut buf = BytesMut::with_capacity(V2Packet::MAX_PACKET_SIZE);

        codec
            .encode(Packet::V2(v2_packet.clone()), &mut buf)
            .unwrap();

        assert_eq!(&buf[..v2_packet.packet_size()], v2_packet.as_slice())
    }
}

#[cfg(test)]
mod test_decode {
    use super::*;
    use mavlink::{
        ardupilotmega::MavMessage, MAVLinkV1MessageRaw, MAVLinkV2MessageRaw, MavHeader, Message,
    };

    #[test]
    fn test_decode_v1() {
        let mut codec = MavlinkCodec::<true, false, false, false, false, false, false>::default();

        let mut buf = BytesMut::with_capacity(V1Packet::MAX_PACKET_SIZE);

        let expected_packet = {
            let header = MavHeader {
                system_id: 1,
                component_id: 1,
                sequence: 0,
            };

            let message_data = MavMessage::default_message_from_id(0).unwrap(); // Heartbeat message
            let mut raw_v1_message = MAVLinkV1MessageRaw::new();
            raw_v1_message.serialize_message(header, &message_data);

            buf.put(raw_v1_message.raw_bytes());

            Packet::V1(V1Packet::from(raw_v1_message))
        };
        assert!(!buf.is_empty());

        let packet = codec.decode(&mut buf).unwrap().unwrap().unwrap();

        assert_eq!(packet, expected_packet);
    }

    #[test]
    fn test_decode_v2() {
        let mut codec = MavlinkCodec::<false, true, false, false, false, false, false>::default();

        let mut buf = BytesMut::with_capacity(V1Packet::MAX_PACKET_SIZE);

        let expected_packet = {
            let header = MavHeader {
                system_id: 1,
                component_id: 1,
                sequence: 0,
            };

            let message_data = MavMessage::default_message_from_id(0).unwrap(); // Heartbeat message
            let mut raw_v2_message = MAVLinkV2MessageRaw::new();
            raw_v2_message.serialize_message(header, &message_data);

            buf.put(raw_v2_message.raw_bytes());

            Packet::V2(V2Packet::from(raw_v2_message))
        };
        assert!(!buf.is_empty());

        let packet = codec.decode(&mut buf).unwrap().unwrap().unwrap();

        assert_eq!(packet, expected_packet);
    }
}
