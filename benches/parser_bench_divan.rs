use divan::{AllocProfiler, Bencher};
use serde::{Deserialize, Serialize};

const LENS: &[usize] = &[100];

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

/// Adapted from mavlink2rest and mavlink-server
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MAVLinkMessage<T> {
    pub header: mavlink::MavHeader,
    pub message: T,
}

#[divan::bench_group(sample_size = 100)]
mod from_json_string_to_packet {
    use mavlink::{Message as _, MessageData as _};
    use mavlink_codec::Packet;
    use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

    use super::*;

    fn prepare(messages_count: usize) -> Vec<String> {
        let seed = 42;
        let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

        let mut json_strings: Vec<String> = Vec::with_capacity(messages_count as usize);

        for _ in 0..messages_count {
            use rand::Rng;

            let header = mavlink::MavHeader {
                system_id: rng.gen_range(1..255),
                component_id: rng.gen_range(1..255),
                sequence: rng.gen_range(0..255),
            };

            loop {
                // let message_id = rng.gen_range(0..2 ^ 24);
                let message_id = *[
                    mavlink::ardupilotmega::RC_CHANNELS_DATA::ID,
                    mavlink::ardupilotmega::HEARTBEAT_DATA::ID,
                    // Add more message types
                ]
                .choose(&mut rng)
                .unwrap();

                if let Ok(message) =
                    mavlink::ardupilotmega::MavMessage::default_message_from_id(message_id)
                {
                    let frame = MAVLinkMessage { header, message };

                    let json_string = serde_json::to_string_pretty(&frame).unwrap();

                    json_strings.push(json_string);
                    break;
                };
            }
        }

        json_strings
    }

    #[inline(always)]
    fn packet_from_string_new(json_string: &str) -> Packet {
        let frame: mavlink_codec::mav_types::mav_frame::MavFrame =
            serde_json::from_str(json_string).unwrap();

        Packet::from(frame)
    }

    #[inline(always)]
    fn packet_from_string_old(json_string: &str) -> Packet {
        let frame: MAVLinkMessage<mavlink::ardupilotmega::MavMessage> =
            serde_json::from_str(json_string).unwrap();

        let mut message_raw = mavlink::MAVLinkV2MessageRaw::new();
        message_raw.serialize_message(frame.header, &frame.message);

        Packet::from(message_raw)
    }

    #[divan::bench(args = LENS)]
    fn new(bencher: Bencher, messages_count: usize) {
        let json_strings = prepare(messages_count);
        let packets: Vec<Packet> = Vec::with_capacity(messages_count);

        bencher
            .counter(divan::counter::ItemsCount::new(messages_count))
            .with_inputs(|| (json_strings.clone(), packets.clone()))
            .bench_refs(|(json_strings, packets)| {
                json_strings.iter().for_each(|json_string| {
                    let packet = packet_from_string_new(json_string);
                    packets.push(packet);
                });
            });
    }

    #[divan::bench(args = LENS)]
    fn old(bencher: Bencher, messages_count: usize) {
        let json_strings = prepare(messages_count);
        let packets: Vec<Packet> = Vec::with_capacity(messages_count);

        bencher
            .counter(divan::counter::ItemsCount::new(messages_count))
            .with_inputs(|| (json_strings.clone(), packets.clone()))
            .bench_refs(|(json_strings, packets)| {
                json_strings.iter().for_each(|json_string| {
                    let packet = packet_from_string_old(json_string);
                    packets.push(packet);
                });
            });
    }
}

fn main() {
    divan::main();
}
