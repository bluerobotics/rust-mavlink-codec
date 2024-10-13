use std::io::Write;

use mavlink::{MAVLinkV1MessageRaw, MAVLinkV2MessageRaw};
use rand::{prelude::StdRng, Rng};

pub fn add_random_v1_message(buf: &mut Vec<u8>, rng: &mut StdRng) {
    let raw_v1_message = create_random_v1_raw_message(rng);

    buf.write(raw_v1_message.raw_bytes()).unwrap();
}

pub fn create_random_v1_raw_message(rng: &mut StdRng) -> MAVLinkV1MessageRaw {
    use mavlink::{ardupilotmega::*, Message};

    let header = mavlink::MavHeader {
        system_id: rng.gen_range(1..255),
        component_id: rng.gen_range(1..255),
        sequence: rng.gen_range(0..255),
    };

    loop {
        let message_id = rng.gen_range(0..2 ^ 24);
        if let Ok(message_data) = MavMessage::default_message_from_id(message_id) {
            let mut raw_v1_message = MAVLinkV1MessageRaw::new();

            raw_v1_message.serialize_message(header, &message_data);

            return raw_v1_message;
        };
    }
}

pub fn add_random_v2_message(buf: &mut Vec<u8>, rng: &mut StdRng) {
    let raw_v2_message = create_random_v2_raw_message(rng);

    buf.write(raw_v2_message.raw_bytes()).unwrap();
}

pub fn create_random_v2_raw_message(rng: &mut StdRng) -> MAVLinkV2MessageRaw {
    use mavlink::{ardupilotmega::*, Message};

    let header = mavlink::MavHeader {
        system_id: rng.gen_range(1..255),
        component_id: rng.gen_range(1..255),
        sequence: rng.gen_range(0..255),
    };

    loop {
        let message_id = rng.gen_range(0..2 ^ 24);
        if let Ok(message_data) = MavMessage::default_message_from_id(message_id) {
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
        let chunk_size = rng.gen_range(min..=max).min(remaining);
        let end = start + chunk_size;
        chunks.push(buf[start..end].to_vec());
        start = end;
        remaining -= chunk_size;
    }

    chunks
}
