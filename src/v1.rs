use bytes::Bytes;

pub const V1_STX: u8 = 0xFE;

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct V1Packet {
    pub(crate) buffer: Bytes,
}

impl std::fmt::Debug for V1Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("V1Packet")
            .field("buffer", &&self.buffer[..])
            .finish()
    }
}

impl V1Packet {
    pub const STX_SIZE: usize = 1;
    pub const HEADER_SIZE: usize = 5;
    pub const MAX_PAYLOAD_SIZE: usize = 255;
    pub const CHECKSUM_SIZE: usize = std::mem::size_of::<u16>();
    pub const MAX_PACKET_SIZE: usize = V1Packet::STX_SIZE
        + V1Packet::HEADER_SIZE
        + V1Packet::MAX_PAYLOAD_SIZE
        + V1Packet::CHECKSUM_SIZE;

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
    pub fn header(&self) -> &[u8] {
        header(&self.buffer)
    }

    #[inline(always)]
    pub fn payload(&self) -> &[u8] {
        payload(&self.buffer)
    }

    #[inline(always)]
    pub fn checksum(&self) -> u16 {
        checksum(&self.buffer)
    }

    #[inline(always)]
    pub fn checksum_data(&self) -> &[u8] {
        checksum_data(&self.buffer)
    }

    #[inline(always)]
    pub fn packet_size(&self) -> usize {
        packet_size(&self.buffer)
    }

    #[inline(always)]
    pub fn stx(&self) -> &u8 {
        stx(&self.buffer)
    }

    #[inline(always)]
    pub fn payload_length(&self) -> &u8 {
        len(&self.buffer)
    }

    #[inline(always)]
    pub fn sequence(&self) -> &u8 {
        seq(&self.buffer)
    }

    #[inline(always)]
    pub fn system_id(&self) -> &u8 {
        sysid(&self.buffer)
    }

    #[inline(always)]
    pub fn component_id(&self) -> &u8 {
        compid(&self.buffer)
    }

    #[inline(always)]
    pub fn message_id(&self) -> &u8 {
        msgid(&self.buffer)
    }
}

#[inline(always)]
pub(crate) fn header<T: AsRef<[u8]>>(buf: &T) -> &[u8] {
    let header_start = V1Packet::STX_SIZE;
    let header_end = header_start + V1Packet::HEADER_SIZE;

    &buf.as_ref()[header_start..header_end]
}

#[inline(always)]
pub(crate) fn payload<T: AsRef<[u8]>>(buf: &T) -> &[u8] {
    let payload_start = V1Packet::STX_SIZE + V1Packet::HEADER_SIZE;
    let payload_size = *len(buf) as usize;
    let payload_end = payload_start + payload_size;

    &buf.as_ref()[payload_start..payload_end]
}

#[inline(always)]
pub(crate) fn checksum<T: AsRef<[u8]>>(buf: &T) -> u16 {
    let checksum_end = packet_size(buf);
    let checksum_start = checksum_end - V1Packet::CHECKSUM_SIZE;

    let buf = buf.as_ref();
    u16::from_le_bytes([buf[checksum_start], buf[checksum_end - 1]])
}

#[inline(always)]
pub(crate) fn checksum_data<T: AsRef<[u8]>>(buf: &T) -> &[u8] {
    let checksum_data_start = V1Packet::STX_SIZE;
    let payload_size = *len(buf) as usize;
    let checksum_data_end = V1Packet::STX_SIZE + V1Packet::HEADER_SIZE + payload_size;

    &buf.as_ref()[checksum_data_start..checksum_data_end]
}

#[inline(always)]
pub(crate) fn packet_size<T: AsRef<[u8]>>(buf: &T) -> usize {
    let stx = V1Packet::STX_SIZE;
    let header = V1Packet::HEADER_SIZE;
    let payload = *len(buf) as usize;
    let checksum = V1Packet::CHECKSUM_SIZE;

    stx + header + payload + checksum
}

#[inline(always)]
pub(crate) fn stx<T: AsRef<[u8]>>(buf: &T) -> &u8 {
    &buf.as_ref()[0]
}

#[inline(always)]
pub(crate) fn len<T: AsRef<[u8]>>(buf: &T) -> &u8 {
    &buf.as_ref()[1]
}

#[inline(always)]
pub(crate) fn seq<T: AsRef<[u8]>>(buf: &T) -> &u8 {
    &buf.as_ref()[2]
}

#[inline(always)]
pub(crate) fn sysid<T: AsRef<[u8]>>(buf: &T) -> &u8 {
    &buf.as_ref()[3]
}

#[inline(always)]
pub(crate) fn compid<T: AsRef<[u8]>>(buf: &T) -> &u8 {
    &buf.as_ref()[4]
}

#[inline(always)]
pub(crate) fn msgid<T: AsRef<[u8]>>(buf: &T) -> &u8 {
    &buf.as_ref()[5]
}

#[cfg(test)]
mod test {
    use super::*;

    pub const HEARTBEAT: &[u8] = &[
        254, // stx
        // start of header
        9,   // payload len
        239, // seq
        1,   // sys ID
        2,   // comp ID
        0,   // msg ID
        // end of header
        // start of payload
        5, 0, 0, 0, 2, 3, 89, 3, 3, //
        // end of payload
        31, 80, // crc
    ];

    #[test]
    fn test_stx() {
        assert_eq!(*stx(&HEARTBEAT), V1_STX);
    }

    #[test]
    fn test_len() {
        assert_eq!(*len(&HEARTBEAT), 9);
    }

    #[test]
    fn test_seq() {
        assert_eq!(*seq(&HEARTBEAT), 239);
    }

    #[test]
    fn test_sysid() {
        assert_eq!(*sysid(&HEARTBEAT), 1);
    }

    #[test]
    fn test_compid() {
        assert_eq!(*compid(&HEARTBEAT), 2);
    }

    #[test]
    fn test_msgid() {
        assert_eq!(*msgid(&HEARTBEAT), 0);
    }

    #[test]
    fn test_header() {
        assert_eq!(header(&HEARTBEAT), &HEARTBEAT[1..(1 + 5)]);
    }

    #[test]
    fn test_payload() {
        assert_eq!(payload(&HEARTBEAT), &HEARTBEAT[(1 + 5)..((1 + 5) + 9)]);
    }

    #[test]
    fn test_checksum() {
        assert_eq!(checksum(&HEARTBEAT), u16::from_le_bytes([31, 80]));
    }

    #[test]
    fn test_checksum_data() {
        assert_eq!(checksum_data(&HEARTBEAT), &HEARTBEAT[1..((1 + 5) + 9)]);
    }

    #[test]
    fn test_packet_size() {
        assert_eq!(packet_size(&HEARTBEAT), (1 + 5) + 9 + 2);
    }

    #[test]
    fn test_v1packet_from_raw_v1_message() {
        use mavlink::{ardupilotmega::MavMessage, MAVLinkV1MessageRaw, MavHeader, Message};

        let raw_v1_message = {
            let header = MavHeader {
                system_id: 1,
                component_id: 1,
                sequence: 0,
            };

            let message_data = MavMessage::default_message_from_id(0).unwrap(); // Heartbeat message
            let mut raw_v1_message = MAVLinkV1MessageRaw::new();
            raw_v1_message.serialize_message(header, &message_data);
            raw_v1_message
        };

        let v1_packet = V1Packet::from(raw_v1_message);

        assert_eq!(v1_packet.header(), raw_v1_message.clone().header()); // Todo: remote this clone once [this PR](https://github.com/mavlink/rust-mavlink/pull/288) get merged upstream
        assert_eq!(*v1_packet.stx(), raw_v1_message.raw_bytes()[0]);
        assert_eq!(*v1_packet.payload_length(), raw_v1_message.payload_length());
        assert_eq!(*v1_packet.sequence(), raw_v1_message.sequence());
        assert_eq!(*v1_packet.system_id(), raw_v1_message.system_id());
        assert_eq!(*v1_packet.component_id(), raw_v1_message.component_id());
        assert_eq!(*v1_packet.message_id(), raw_v1_message.message_id());
        assert_eq!(v1_packet.payload(), raw_v1_message.payload());
        assert_eq!(v1_packet.checksum(), raw_v1_message.checksum());
    }
}
