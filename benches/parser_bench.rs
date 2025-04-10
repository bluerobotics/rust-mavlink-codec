use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration, Throughput,
};

use mavlink::Message;
use mavlink_codec::{codec::MavlinkCodec, v2::V2Packet, Packet};
use rand::{rngs::StdRng, SeedableRng};
use serde_derive::{Deserialize, Serialize};
use tokio_util::codec::Decoder;

/// Adapted from mavlink2rest and mavlink-server
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MAVLinkMessage<T> {
    pub header: mavlink::MavHeader,
    pub message: T,
}

fn add_random_v2_message(buf: &mut Vec<u8>, rng: &mut StdRng) {
    use rand::Rng;

    let header = mavlink::MavHeader {
        system_id: rng.gen_range(1..255),
        component_id: rng.gen_range(1..255),
        sequence: rng.gen_range(0..255),
    };

    loop {
        // let message_id = rng.gen_range(0..2 ^ 24);
        let message_id = 0;
        if let Ok(data) = mavlink::ardupilotmega::MavMessage::default_message_from_id(message_id) {
            if mavlink::write_v2_msg(buf, header, &data).is_ok() {
                break;
            }
        };
    }
}

fn benchmark_packet_to_json_value(c: &mut Criterion) {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("packet_to_json_value");
    group.confidence_level(0.95).sample_size(100);

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    group.plot_config(plot_config);

    let messages_counts = vec![
        1, 5, 10, 50, 100, /*500, 1000, 5000, 10000, 50000, 100000*/
    ];

    let rt = tokio::runtime::Runtime::new().unwrap();

    for messages_count in &messages_counts {
        group.throughput(Throughput::Elements(*messages_count));

        let mut buf: Vec<u8> =
            Vec::with_capacity(V2Packet::MAX_PACKET_SIZE * *messages_count as usize);
        for _ in 0..*messages_count {
            add_random_v2_message(&mut buf, &mut rng);
        }

        let mut decoded_packets = Vec::with_capacity(*messages_count as usize);

        rt.block_on(async {
            let mut buf = bytes::BytesMut::from(buf.as_slice());
            let mut codec = MavlinkCodec::<true, true, false, false, false, false>::default();

            for _ in 0..*messages_count {
                let decodec_packet = codec.decode(&mut buf).unwrap().unwrap().unwrap();
                decoded_packets.push(decodec_packet)
            }
        });

        group.bench_with_input(
            BenchmarkId::new("new", messages_count),
            messages_count,
            |b, _messages_count| {
                let decoded_packets = decoded_packets.clone();

                b.to_async(&rt).iter(|| async {
                    let decoded_packets = decoded_packets.clone();

                    for packet in decoded_packets {
                        let frame = mavframe_from_packet_new(packet);

                        let _json = black_box(serde_json::to_value(&frame).unwrap());
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("old", messages_count),
            messages_count,
            |b, _messages_count| {
                let decoded_packets = decoded_packets.clone();

                b.to_async(&rt).iter(|| async {
                    let decoded_packets = decoded_packets.clone();

                    for packet in decoded_packets {
                        let frame = mavframe_from_packet_old(packet);

                        let _json = black_box(serde_json::to_value(&frame).unwrap());
                    }
                })
            },
        );
    }
}

fn benchmark_packet_to_json_string(c: &mut Criterion) {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("packet_to_json_string");
    group.confidence_level(0.95).sample_size(100);

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    group.plot_config(plot_config);

    let messages_counts = vec![
        1, 5, 10, 50, 100, /*500, 1000, 5000, 10000, 50000, 100000*/
    ];

    let rt = tokio::runtime::Runtime::new().unwrap();

    for messages_count in &messages_counts {
        group.throughput(Throughput::Elements(*messages_count));

        let mut buf: Vec<u8> =
            Vec::with_capacity(V2Packet::MAX_PACKET_SIZE * *messages_count as usize);
        for _ in 0..*messages_count {
            add_random_v2_message(&mut buf, &mut rng);
        }

        let mut decoded_packets = Vec::with_capacity(*messages_count as usize);

        rt.block_on(async {
            let mut buf = bytes::BytesMut::from(buf.as_slice());
            let mut codec = MavlinkCodec::<true, true, false, false, false, false>::default();

            for _ in 0..*messages_count {
                let decodec_packet = codec.decode(&mut buf).unwrap().unwrap().unwrap();
                decoded_packets.push(decodec_packet)
            }
        });

        group.bench_with_input(
            BenchmarkId::new("new", messages_count),
            messages_count,
            |b, _messages_count| {
                let decoded_packets = decoded_packets.clone();

                b.to_async(&rt).iter(|| async {
                    let decoded_packets = decoded_packets.clone();

                    for packet in decoded_packets {
                        let frame = mavframe_from_packet_new(packet);

                        let _json = black_box(serde_json::to_string_pretty(&frame).unwrap());
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("old", messages_count),
            messages_count,
            |b, _messages_count| {
                let decoded_packets = decoded_packets.clone();

                b.to_async(&rt).iter(|| async {
                    let decoded_packets = decoded_packets.clone();

                    for packet in decoded_packets {
                        let frame = mavframe_from_packet_old(packet);

                        let _json = black_box(serde_json::to_string_pretty(&frame).unwrap());
                    }
                })
            },
        );
    }
}

fn benchmark_from_json_string_to_frame(c: &mut Criterion) {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("from_json_string_to_frame");
    group.confidence_level(0.95).sample_size(100);

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    group.plot_config(plot_config);

    let messages_counts = vec![
        1, 5, 10, 50, 100, /*500, 1000, 5000, 10000, 50000, 100000*/
    ];

    let rt = tokio::runtime::Runtime::new().unwrap();

    for messages_count in &messages_counts {
        group.throughput(Throughput::Elements(*messages_count));

        let mut json_strings: Vec<String> = Vec::with_capacity(*messages_count as usize);

        for _ in 0..*messages_count {
            use rand::Rng;

            let header = mavlink::MavHeader {
                system_id: rng.gen_range(1..255),
                component_id: rng.gen_range(1..255),
                sequence: rng.gen_range(0..255),
            };

            loop {
                // let message_id = rng.gen_range(0..2 ^ 24);
                let message_id = 0;
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

        group.bench_with_input(
            BenchmarkId::new("new", messages_count),
            messages_count,
            |b, _messages_count| {
                b.to_async(&rt).iter(|| async {
                    let json_strings = json_strings.clone();

                    for json_string in &json_strings {
                        let _frame = black_box(mavframe_from_string_new(json_string));
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("old", messages_count),
            messages_count,
            |b, _messages_count| {
                b.to_async(&rt).iter(|| async {
                    let json_strings = json_strings.clone();

                    for json_string in &json_strings {
                        let _frame = black_box(mavframe_from_string_old(json_string));
                    }
                })
            },
        );
    }
}

#[inline(always)]
fn mavframe_from_string_new(json_string: &str) -> mavlink_codec::mav_types::mav_frame::MavFrame {
    serde_json::from_str(json_string).unwrap()
}

#[inline(always)]
fn mavframe_from_string_old(
    json_string: &str,
) -> MAVLinkMessage<mavlink::ardupilotmega::MavMessage> {
    serde_json::from_str(json_string).unwrap()
}

#[inline(always)]
fn packet_from_string_new(json_string: &str) -> Packet {
    // let frame: mavlink_codec::mav_types::mavframe::MavFrame = serde_json::from_str(json_string).unwrap();
    todo!()
}

#[inline(always)]
fn packet_from_string_old(json_string: &str) -> Packet {
    todo!()
}

#[inline(always)]
fn mavframe_from_packet_new(packet: Packet) -> mavlink_codec::mav_types::mav_frame::MavFrame {
    todo!()
    // mavlink_codec::mav_types::mavframe::MavFrame::try_from(&packet).unwrap()
}

#[inline(always)]
fn mavframe_from_packet_old(
    packet: Packet,
) -> mavlink::MavFrame<mavlink::ardupilotmega::MavMessage> {
    let header = mavlink::MavHeader {
        sequence: *packet.sequence(),
        system_id: *packet.system_id(),
        component_id: *packet.component_id(),
    };

    let version = match &packet {
        Packet::V1(_) => mavlink::MavlinkVersion::V1,
        Packet::V2(_) => mavlink::MavlinkVersion::V2,
    };

    let message = mavlink::ardupilotmega::MavMessage::parse(
        version,
        u32::from(packet.message_id()),
        packet.payload(),
    )
    .unwrap();

    let frame = mavlink::MavFrame {
        header,
        msg: message,
        protocol_version: version,
    };
    frame
}

#[inline(always)]
fn packet_from_mavframe_new(frame: mavlink_codec::mav_types::mav_frame::MavFrame) -> Packet {
    // Packet::try_from(&frame).unwrap()
    todo!()
}

criterion_group!(
    benches,
    // benchmark_packet_to_json_value,
    // benchmark_packet_to_json_string,
    benchmark_from_json_string_to_frame
);
criterion_main!(benches);
