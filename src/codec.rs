use bytes::{Buf, BufMut, BytesMut};
use log::trace;
use mavlink::calculate_crc;
use tokio_util::codec::{Decoder, Encoder};

use crate::{
    error::DecoderError,
    v1::{self, V1Packet, V1_STX},
    v2::{self, V2Packet, MAVLINK_SUPPORTED_IFLAGS, V2_STX},
    Packet,
};

#[derive(Debug, Default)]
pub struct MavlinkCodec<
    const ACCEPT_V1: bool,
    const ACCEPT_V2: bool,
    const DROP_INVALID_SYSID: bool,
    const DROP_INVALID_COMPID: bool,
    const SKIP_CRC_VALIDATION: bool,
    const DROP_INCOMPATIBLE: bool,
> {
    pub state: CodecState,
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
    > Decoder
    for MavlinkCodec<
        ACCEPT_V1,
        ACCEPT_V2,
        DROP_INVALID_SYSID,
        DROP_INVALID_COMPID,
        SKIP_CRC_VALIDATION,
        DROP_INCOMPATIBLE,
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
                        if ACCEPT_V2 {
                            // buf.reserve(V2Packet::MAX_PACKET_SIZE);
                        } else {
                            // buf.reserve(V1Packet::MAX_PACKET_SIZE);
                        }

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
                        // buf.reserve(V1Packet::HEADER_SIZE);

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
                        // buf.reserve(V1Packet::MAX_PACKET_SIZE);

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
                    if SKIP_CRC_VALIDATION {
                        trace!("CRC Validation skipped.");
                        self.state = CodecState::CopyV1Packet { packet_size };
                        continue;
                    }

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

                    self.state = CodecState::CopyV1Packet { packet_size };
                }
                CodecState::CopyV1Packet { packet_size } if ACCEPT_V1 => {
                    let buf_packet = if SKIP_CRC_VALIDATION {
                        // Copy the entire packet consuming the source buffer
                        let mut buf_packet = BytesMut::with_capacity(packet_size);
                        buf_packet[..packet_size].copy_from_slice(&buf[..packet_size]);

                        // Since it is a non validated packet, there might be other packets within this buffer, so we can only discard this STX
                        buf.advance(V1Packet::STX_SIZE);

                        buf_packet
                    } else {
                        let buf_packet = buf.split_to(packet_size);
                        // buf.reserve(V1Packet::MAX_PACKET_SIZE);

                        buf_packet
                    };

                    let packet = V1Packet {
                        buffer: buf_packet.freeze(),
                    };

                    self.state = CodecState::WaitingForStx;
                    return Ok(Some(Ok(Packet::V1(packet))));
                }
                // V2 Codec
                CodecState::WaitingV2PacketHeader if ACCEPT_V2 => {
                    if buf.len() < V2Packet::HEADER_SIZE {
                        // buf.reserve(V2Packet::HEADER_SIZE);

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
                            buf.advance(V1Packet::STX_SIZE); // Discard this STX
                            self.state = CodecState::WaitingForStx;

                            return Ok(Some(Err(DecoderError::Incompatible { incompat_flags })));
                        }
                    }

                    let packet_size = v2::packet_size(buf);
                    self.state = CodecState::ValidatingV2Packet { packet_size };
                }
                CodecState::ValidatingV2Packet { packet_size } if ACCEPT_V2 => {
                    if buf.len() < packet_size {
                        // buf.reserve(V2Packet::MAX_PACKET_SIZE);

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
                    if SKIP_CRC_VALIDATION {
                        trace!("CRC Validation skipped.");
                        self.state = CodecState::CopyV2Packet { packet_size };
                        continue;
                    }

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

                    self.state = CodecState::CopyV2Packet { packet_size };
                }
                CodecState::CopyV2Packet { packet_size } if ACCEPT_V2 => {
                    let buf_packet = if SKIP_CRC_VALIDATION {
                        // Copy the entire packet consuming the source buffer
                        let mut buf_packet = BytesMut::with_capacity(packet_size);
                        buf_packet[..packet_size].copy_from_slice(&buf[..packet_size]);

                        // Since it is a non validated packet, there might be other packets within this buffer, so we can only discard this STX
                        buf.advance(V2Packet::STX_SIZE);

                        buf_packet
                    } else {
                        let buf_packet = buf.split_to(packet_size);
                        // buf.reserve(V2Packet::MAX_PACKET_SIZE);

                        buf_packet
                    };

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
    > Encoder<Packet>
    for MavlinkCodec<
        ACCEPT_V1,
        ACCEPT_V2,
        DROP_INVALID_SYSID,
        DROP_INVALID_COMPID,
        SKIP_CRC_VALIDATION,
        DROP_INCOMPATIBLE,
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
        let mut codec = MavlinkCodec::<true, true, false, false, false, false>::default();

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
        let mut codec = MavlinkCodec::<true, true, false, false, false, false>::default();

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
        let mut codec = MavlinkCodec::<true, false, false, false, false, false>::default();

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
        let mut codec = MavlinkCodec::<false, true, false, false, false, false>::default();

        let mut buf = BytesMut::with_capacity(V2Packet::MAX_PACKET_SIZE);

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
