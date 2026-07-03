use std::{io::Write, sync::OnceLock};

use mavlink::{MAVLinkV1MessageRaw, MAVLinkV2MessageRaw};
use rand::{prelude::StdRng, Rng};

pub fn all_message_ids() -> &'static [(&'static str, u32)] {
    static IDS: OnceLock<Vec<(&'static str, u32)>> = OnceLock::new();

    IDS.get_or_init(|| {
        use mavlink::{dialects::ardupilotmega::MavMessage, Message};

        MavMessage::all_ids()
            .iter()
            .map(|&id| {
                let msg = MavMessage::default_message_from_id(id)
                    .unwrap_or_else(|| panic!("dialect lists id {id} but has no default message"));
                (msg.message_name(), id)
            })
            .collect()
    })
    .as_slice()
}

pub fn create_random_v1_message_from_id(rng: &mut StdRng, id: u32) -> Option<MAVLinkV1MessageRaw> {
    use mavlink::{dialects::ardupilotmega::MavMessage, Message};

    let message_data = MavMessage::random_message_from_id(id, rng)?;
    let header = random_header(rng);
    let mut raw = MAVLinkV1MessageRaw::new();
    raw.serialize_message(header, &message_data);
    Some(raw)
}

pub fn create_random_v2_message_from_id(rng: &mut StdRng, id: u32) -> Option<MAVLinkV2MessageRaw> {
    use mavlink::{dialects::ardupilotmega::MavMessage, Message};

    let message_data = MavMessage::random_message_from_id(id, rng)?;
    let header = random_header(rng);
    let mut raw = MAVLinkV2MessageRaw::new();
    raw.serialize_message(header, &message_data);
    Some(raw)
}

fn random_header(rng: &mut StdRng) -> mavlink::MavHeader {
    mavlink::MavHeader {
        system_id: rng.random_range(1..255),
        component_id: rng.random_range(1..255),
        sequence: rng.random_range(0..255),
    }
}

pub fn add_random_v1_message(buf: &mut Vec<u8>, rng: &mut StdRng) {
    let raw_v1_message = create_random_v1_raw_message(rng);

    buf.write_all(raw_v1_message.raw_bytes()).unwrap();
}

pub fn create_random_v1_raw_message(rng: &mut StdRng) -> MAVLinkV1MessageRaw {
    use mavlink::{dialects::ardupilotmega::*, Message};

    let header = mavlink::MavHeader {
        system_id: rng.random_range(1..255),
        component_id: rng.random_range(1..255),
        sequence: rng.random_range(0..255),
    };

    loop {
        let message_id = rng.random_range(0..2 ^ 24);
        if let Some(message_data) = MavMessage::default_message_from_id(message_id) {
            let mut raw_v1_message = MAVLinkV1MessageRaw::new();

            raw_v1_message.serialize_message(header, &message_data);

            return raw_v1_message;
        };
    }
}

pub fn add_random_v2_message(buf: &mut Vec<u8>, rng: &mut StdRng) {
    let raw_v2_message = create_random_v2_raw_message(rng);

    buf.write_all(raw_v2_message.raw_bytes()).unwrap();
}

pub fn create_random_v2_raw_message(rng: &mut StdRng) -> MAVLinkV2MessageRaw {
    use mavlink::{dialects::ardupilotmega::*, Message};

    let header = mavlink::MavHeader {
        system_id: rng.random_range(1..255),
        component_id: rng.random_range(1..255),
        sequence: rng.random_range(0..255),
    };

    loop {
        let message_id = rng.random_range(0..2 ^ 24);
        if let Some(message_data) = MavMessage::default_message_from_id(message_id) {
            let mut raw_v2_message = MAVLinkV2MessageRaw::new();

            raw_v2_message.serialize_message(header, &message_data);

            return raw_v2_message;
        };
    }
}

pub fn chunk_buffer_randomly(buf: &[u8], rng: &mut StdRng, min: usize, max: usize) -> Vec<Vec<u8>> {
    let mut chunks = Vec::new();
    let mut remaining = buf.len();
    let mut start = 0;

    while remaining > 0 {
        let chunk_size = rng.random_range(min..=max).min(remaining);
        let end = start + chunk_size;
        chunks.push(buf[start..end].to_vec());
        start = end;
        remaining -= chunk_size;
    }

    chunks
}
