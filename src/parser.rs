use crate::{
    error::DecoderError,
    mav_types::mav_message::{heartbeat::HeartbeatMessage, MavMessage, MavMessageDef},
    v2::V2Packet,
    Packet,
};

pub fn parse(packet: &Packet) -> Result<MavMessage, DecoderError> {
    let payload_start = V2Packet::STX_SIZE + V2Packet::HEADER_SIZE;
    let payload_end = payload_start + *packet.payload_length() as usize;

    // This increases the reference counter from the original packet.buffer, so it is guarantee to exist beyond packet's life
    let payload = packet.bytes().slice(payload_start..payload_end);

    let message_id = packet.message_id();

    match message_id {
        HeartbeatMessage::ID => Ok(MavMessage::Heartbeat(HeartbeatMessage { buffer: payload })),
        _ => Err(DecoderError::UnknownMessageID { msgid: message_id }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mavlink::Message as _;

    #[test]
    fn test_parse() {
        // 1. Creates a default Heartbeat message using rust-mavlink
        let original_message =
            mavlink::ardupilotmega::MavMessage::default_message_from_id(0).unwrap();
        dbg!(&original_message);

        // 2. Serialize it into a raw message
        let mut raw_v2_message = mavlink::MAVLinkV2MessageRaw::new();
        raw_v2_message.serialize_message(mavlink::MavHeader::default(), &original_message);
        let mavlink::ardupilotmega::MavMessage::HEARTBEAT(original_heartbeat_message_data) =
            original_message
        else {
            unreachable!("Wrong message parsed!");
        };

        // 2. Convert it to V2Packet, this is our decoded mavlink packet:
        let packet = Packet::from(raw_v2_message);
        dbg!(&packet);

        // 3. From the decoded packet, parse the mavlink message
        let parsed_message = parse(&packet).unwrap();

        drop(packet); // Just to be sure the parsed message outlives the packet it came from

        dbg!(&parsed_message);
        let MavMessage::Heartbeat(parsed_heartbeat_message_data) = parsed_message else {
            panic!("Wrong message parsed!");
        };
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
