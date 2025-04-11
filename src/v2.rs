use bytes::Bytes;

pub const V2_STX: u8 = 0xFD;
pub const MAVLINK_IFLAG_SIGNED: u8 = 0x01;
pub const MAVLINK_SUPPORTED_IFLAGS: u8 = MAVLINK_IFLAG_SIGNED;

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct V2Packet {
    pub(crate) buffer: Bytes,
}

impl std::fmt::Debug for V2Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("V2Packet")
            .field("buffer", &&self.buffer[..])
            .finish()
    }
}

impl V2Packet {
    pub const STX_SIZE: usize = 1;
    pub const HEADER_SIZE: usize = 9;
    pub const MAX_PAYLOAD_SIZE: usize = 255;
    pub const CHECKSUM_SIZE: usize = std::mem::size_of::<u16>();
    pub const SIGNATURE_SIZE: usize = 13;
    pub const MAX_PACKET_SIZE: usize = V2Packet::STX_SIZE
        + V2Packet::HEADER_SIZE
        + V2Packet::MAX_PAYLOAD_SIZE
        + V2Packet::CHECKSUM_SIZE
        + V2Packet::SIGNATURE_SIZE;

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
    pub fn signature(&self) -> Option<&[u8]> {
        signature(&self.buffer)
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
    pub fn has_signature(&self) -> bool {
        has_signature(&self.buffer)
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
    pub fn incompatibility_flags(&self) -> &u8 {
        incompat_flags(&self.buffer)
    }

    #[inline(always)]
    pub fn compatibility_flags(&self) -> &u8 {
        compat_flags(&self.buffer)
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
    pub fn message_id(&self) -> u32 {
        msgid(&self.buffer)
    }

    #[inline(always)]
    pub(crate) fn frame_header_bytes(&self) -> Bytes {
        const LEN_SIZE: usize = 1;
        let header_start = V2Packet::STX_SIZE + LEN_SIZE;
        let header_end = header_start + V2Packet::HEADER_SIZE;

        self.buffer.slice(header_start..header_end)
    }

    #[inline(always)]
    pub(crate) fn payload_bytes(&self) -> Bytes {
        let payload_start = V2Packet::STX_SIZE + V2Packet::HEADER_SIZE;
        let payload_size = *len(&self.buffer) as usize;
        let payload_end = payload_start + payload_size;

        self.buffer.slice(payload_start..payload_end)
    }
}

#[inline(always)]
pub(crate) fn header<T: AsRef<[u8]>>(buf: &T) -> &[u8] {
    let header_start = V2Packet::STX_SIZE;
    let header_end = header_start + V2Packet::HEADER_SIZE;

    &buf.as_ref()[header_start..header_end]
}

#[inline(always)]
pub(crate) fn payload<T: AsRef<[u8]>>(buf: &T) -> &[u8] {
    let payload_start = V2Packet::STX_SIZE + V2Packet::HEADER_SIZE;
    let payload_size = *len(buf) as usize;
    let payload_end = payload_start + payload_size;

    &buf.as_ref()[payload_start..payload_end]
}

#[inline(always)]
pub(crate) fn checksum<T: AsRef<[u8]>>(buf: &T) -> u16 {
    let checksum_end = packet_size(buf);
    let checksum_start = checksum_end - V2Packet::CHECKSUM_SIZE;

    let buf = buf.as_ref();
    u16::from_le_bytes([buf[checksum_start], buf[checksum_end - 1]])
}

#[inline(always)]
pub(crate) fn checksum_data<T: AsRef<[u8]>>(buf: &T) -> &[u8] {
    let checksum_data_start = V2Packet::STX_SIZE;
    let payload_size = *len(buf) as usize;
    let checksum_data_end = V2Packet::STX_SIZE + V2Packet::HEADER_SIZE + payload_size;

    &buf.as_ref()[checksum_data_start..checksum_data_end]
}

#[inline(always)]
pub(crate) fn signature<T: AsRef<[u8]>>(buf: &T) -> Option<&[u8]> {
    if !has_signature(buf) {
        return None;
    }

    let payload_size = *len(buf) as usize;
    let signature_start =
        V2Packet::STX_SIZE + V2Packet::HEADER_SIZE + payload_size + V2Packet::CHECKSUM_SIZE;
    let signature_end = signature_start + V2Packet::SIGNATURE_SIZE;

    Some(&buf.as_ref()[signature_start..signature_end])
}

#[inline(always)]
pub(crate) fn packet_size<T: AsRef<[u8]>>(buf: &T) -> usize {
    let stx = V2Packet::STX_SIZE;
    let header = V2Packet::HEADER_SIZE;
    let payload = *len(buf) as usize;
    let checksum = V2Packet::CHECKSUM_SIZE;
    let signature = has_signature(buf)
        .then_some(V2Packet::SIGNATURE_SIZE)
        .unwrap_or_default();

    stx + header + payload + checksum + signature
}

#[inline(always)]
pub(crate) fn has_signature<T: AsRef<[u8]>>(buf: &T) -> bool {
    incompat_flags(buf) & (IncompatibilityFlags::Signed as u8) == 1
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
pub(crate) fn incompat_flags<T: AsRef<[u8]>>(buf: &T) -> &u8 {
    &buf.as_ref()[2]
}

#[inline(always)]
pub(crate) fn compat_flags<T: AsRef<[u8]>>(buf: &T) -> &u8 {
    &buf.as_ref()[3]
}

#[inline(always)]
pub(crate) fn seq<T: AsRef<[u8]>>(buf: &T) -> &u8 {
    &buf.as_ref()[4]
}

#[inline(always)]
pub(crate) fn sysid<T: AsRef<[u8]>>(buf: &T) -> &u8 {
    &buf.as_ref()[5]
}

#[inline(always)]
pub(crate) fn compid<T: AsRef<[u8]>>(buf: &T) -> &u8 {
    &buf.as_ref()[6]
}

#[inline(always)]
pub(crate) fn msgid<T: AsRef<[u8]>>(buf: &T) -> u32 {
    let buf = buf.as_ref();
    u32::from_le_bytes([buf[7], buf[8], buf[9], 0])
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
enum IncompatibilityFlags {
    Signed = 0x01,
}

#[cfg(test)]
mod test {
    use super::*;

    const COMMAND_LONG: &[u8] = &[
        253, // stx
        // start of header
        30, // payload len
        0,  // incompat flags
        0,  // compat flags
        0,  // seq
        0,  // sys ID
        50, // comp ID
        76, 0, 0, // msg ID
        // end of header
        // start of payload
        0, 0, 230, 66, 0, 64, 156, 69, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        255, 1, //
        // end of payload
        188, 195, // crc
    ];

    #[test]
    fn test_stx() {
        assert_eq!(*stx(&COMMAND_LONG), V2_STX);
    }

    #[test]
    fn test_len() {
        assert_eq!(*len(&COMMAND_LONG), 30);
    }

    #[test]
    fn test_incompat_flags() {
        assert_eq!(*incompat_flags(&COMMAND_LONG), 0);
    }

    #[test]
    fn test_compat_flags() {
        assert_eq!(*compat_flags(&COMMAND_LONG), 0);
    }

    #[test]
    fn test_seq() {
        assert_eq!(*seq(&COMMAND_LONG), 0);
    }

    #[test]
    fn test_sysid() {
        assert_eq!(*sysid(&COMMAND_LONG), 0);
    }

    #[test]
    fn test_compid() {
        assert_eq!(*compid(&COMMAND_LONG), 50);
    }

    #[test]
    fn test_msgid() {
        assert_eq!(msgid(&COMMAND_LONG), 76);
    }

    #[test]
    fn test_header() {
        assert_eq!(header(&COMMAND_LONG), &COMMAND_LONG[1..(1 + 9)]);
    }

    #[test]
    fn test_payload() {
        assert_eq!(
            payload(&COMMAND_LONG),
            &COMMAND_LONG[(1 + 9)..((1 + 9) + 30)]
        );
    }

    #[test]
    fn test_checksum() {
        assert_eq!(checksum(&COMMAND_LONG), u16::from_le_bytes([188, 195]));
    }

    #[test]
    fn test_signature_none() {
        assert!(signature(&COMMAND_LONG).is_none());
    }

    #[test]
    fn test_has_signature_false() {
        assert!(!has_signature(&COMMAND_LONG));
    }

    #[test]
    fn test_checksum_data() {
        assert_eq!(
            checksum_data(&COMMAND_LONG),
            &COMMAND_LONG[1..((1 + 9) + 30)]
        );
    }

    #[test]
    fn test_packet_size_no_signature() {
        assert_eq!(packet_size(&COMMAND_LONG), (1 + 9) + 30 + 2);
    }

    #[test]
    fn test_v2packet_from_raw_v2_message() {
        use mavlink::{ardupilotmega::MavMessage, MAVLinkV2MessageRaw, MavHeader, Message};

        let raw_v2_message = {
            let header = MavHeader {
                system_id: 1,
                component_id: 1,
                sequence: 0,
            };

            let message_data = MavMessage::default_message_from_id(0).unwrap(); // Heartbeat message
            let mut raw_v2_message = MAVLinkV2MessageRaw::new();
            raw_v2_message.serialize_message(header, &message_data);
            raw_v2_message
        };

        let v2_packet = V2Packet::from(raw_v2_message);

        assert_eq!(v2_packet.header(), raw_v2_message.header());
        assert_eq!(*v2_packet.stx(), raw_v2_message.raw_bytes()[0]);
        assert_eq!(*v2_packet.payload_length(), raw_v2_message.payload_length());
        assert_eq!(
            *v2_packet.incompatibility_flags(),
            raw_v2_message.incompatibility_flags()
        );
        assert_eq!(
            *v2_packet.compatibility_flags(),
            raw_v2_message.compatibility_flags()
        );
        assert_eq!(*v2_packet.sequence(), raw_v2_message.sequence());
        assert_eq!(*v2_packet.system_id(), raw_v2_message.system_id());
        assert_eq!(*v2_packet.component_id(), raw_v2_message.component_id());
        assert_eq!(v2_packet.message_id(), raw_v2_message.message_id());
        assert_eq!(v2_packet.payload(), raw_v2_message.payload());
        assert_eq!(v2_packet.checksum(), raw_v2_message.checksum());
    }

    #[test]
    fn test_raw_v2_message_from_v2packet() {
        use mavlink::{ardupilotmega::MavMessage, MAVLinkV2MessageRaw, MavHeader, Message};

        let raw_v2_message_original = {
            let header = MavHeader {
                system_id: 1,
                component_id: 1,
                sequence: 0,
            };

            let message_data = MavMessage::default_message_from_id(0).unwrap(); // Heartbeat message
            let mut raw_v2_message = MAVLinkV2MessageRaw::new();
            raw_v2_message.serialize_message(header, &message_data);
            raw_v2_message
        };

        let v2_packet = V2Packet::from(raw_v2_message_original);

        assert_eq!(v2_packet.as_slice(), raw_v2_message_original.raw_bytes());

        let raw_v2_message = MAVLinkV2MessageRaw::try_from(v2_packet).unwrap();

        assert_eq!(raw_v2_message_original, raw_v2_message);
    }
}
