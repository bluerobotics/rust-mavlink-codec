use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BatchSize, BenchmarkId, Criterion,
    PlotConfiguration, Throughput,
};

use mavlink::{Message, MessageData};
use mavlink_codec::{codec::MavlinkCodec, v2::V2Packet, Packet};
use rand::{rngs::StdRng, seq::SliceRandom as _, SeedableRng};
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
        let message_id = *[
            mavlink::ardupilotmega::RC_CHANNELS_DATA::ID,
            mavlink::ardupilotmega::HEARTBEAT_DATA::ID,
            // Add more message types
        ]
        .choose(rng)
        .unwrap();

        if let Ok(data) = mavlink::ardupilotmega::MavMessage::default_message_from_id(message_id) {
            if mavlink::write_v2_msg(buf, header, &data).is_ok() {
                break;
            }
        };
    }
}

fn benchmark_packet_to_mavframe(c: &mut Criterion) {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("packet_to_mavframe");
    group
        .confidence_level(0.95)
        .sample_size(1000)
        .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for messages_count in &vec![1, 100, 10000] {
        group.throughput(Throughput::Elements(*messages_count));

        let mut buf: Vec<u8> =
            Vec::with_capacity(V2Packet::MAX_PACKET_SIZE * *messages_count as usize);
        for _ in 0..*messages_count {
            add_random_v2_message(&mut buf, &mut rng);
        }

        let mut decoded_packets = Vec::with_capacity(*messages_count as usize);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut buf = bytes::BytesMut::from(buf.as_slice());
            let mut codec = MavlinkCodec::<true, true, false, false, false, false>::default();

            for _ in 0..*messages_count {
                let decodec_packet = codec.decode(&mut buf).unwrap().unwrap().unwrap();
                decoded_packets.push(decodec_packet)
            }
        });

        group.bench_function(BenchmarkId::new("new_nodrop", messages_count), |b| {
            b.iter_batched(
                || decoded_packets.clone(),
                |decoded_packets| {
                    decoded_packets
                        .iter()
                        .map(|packet| black_box(mavframe_from_packet_new(packet)))
                        .collect::<Vec<_>>()
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("old_nodrop", messages_count), |b| {
            b.iter_batched(
                || decoded_packets.clone(),
                |decoded_packets| {
                    decoded_packets
                        .iter()
                        .map(|packet| black_box(mavframe_from_packet_old(packet)))
                        .collect::<Vec<_>>()
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("new_drop", messages_count), |b| {
            b.iter_batched(
                || decoded_packets.clone(),
                |decoded_packets| {
                    decoded_packets.iter().for_each(|packet| {
                        let _packet = black_box(mavframe_from_packet_new(packet));
                    })
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("old_drop", messages_count), |b| {
            b.iter_batched(
                || decoded_packets.clone(),
                |decoded_packets| {
                    decoded_packets.iter().for_each(|packet| {
                        let _packet = black_box(mavframe_from_packet_old(packet));
                    })
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn benchmark_packet_to_json_value(c: &mut Criterion) {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("packet_to_json_value");
    group
        .confidence_level(0.95)
        .sample_size(100)
        .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for messages_count in &vec![1, 100, 10000] {
        group.throughput(Throughput::Elements(*messages_count));

        let mut buf: Vec<u8> =
            Vec::with_capacity(V2Packet::MAX_PACKET_SIZE * *messages_count as usize);
        for _ in 0..*messages_count {
            add_random_v2_message(&mut buf, &mut rng);
        }

        let mut decoded_packets = Vec::with_capacity(*messages_count as usize);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut buf = bytes::BytesMut::from(buf.as_slice());
            let mut codec = MavlinkCodec::<true, true, false, false, false, false>::default();

            for _ in 0..*messages_count {
                let decodec_packet = codec.decode(&mut buf).unwrap().unwrap().unwrap();
                decoded_packets.push(decodec_packet)
            }
        });

        group.bench_function(BenchmarkId::new("new_nodrop", messages_count), |b| {
            b.iter_batched(
                || decoded_packets.clone(),
                |decoded_packets| {
                    decoded_packets
                        .iter()
                        .map(|packet| {
                            let frame = mavframe_from_packet_new(packet);

                            black_box(serde_json::to_value(&frame).unwrap())
                        })
                        .collect::<Vec<_>>()
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("old_nodrop", messages_count), |b| {
            b.iter_batched(
                || decoded_packets.clone(),
                |decoded_packets| {
                    decoded_packets
                        .iter()
                        .map(|packet| {
                            let frame = mavframe_from_packet_old(packet);

                            black_box(serde_json::to_value(&frame).unwrap())
                        })
                        .collect::<Vec<_>>()
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("new_drop", messages_count), |b| {
            b.iter_batched(
                || decoded_packets.clone(),
                |decoded_packets| {
                    decoded_packets.iter().for_each(|packet| {
                        let frame = mavframe_from_packet_new(packet);

                        let _ret = black_box(serde_json::to_value(&frame).unwrap());
                    })
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("old_drop", messages_count), |b| {
            b.iter_batched(
                || decoded_packets.clone(),
                |decoded_packets| {
                    decoded_packets.iter().for_each(|packet| {
                        let frame = mavframe_from_packet_old(packet);

                        let _ret = black_box(serde_json::to_value(&frame).unwrap());
                    })
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn benchmark_packet_to_json_string(c: &mut Criterion) {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("packet_to_json_string");
    group
        .confidence_level(0.95)
        .sample_size(100)
        .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for messages_count in &vec![1, 100, 10000] {
        group.throughput(Throughput::Elements(*messages_count));

        let mut buf: Vec<u8> =
            Vec::with_capacity(V2Packet::MAX_PACKET_SIZE * *messages_count as usize);
        for _ in 0..*messages_count {
            add_random_v2_message(&mut buf, &mut rng);
        }

        let mut decoded_packets = Vec::with_capacity(*messages_count as usize);

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let mut buf = bytes::BytesMut::from(buf.as_slice());
            let mut codec = MavlinkCodec::<true, true, false, false, false, false>::default();

            for _ in 0..*messages_count {
                let decodec_packet = codec.decode(&mut buf).unwrap().unwrap().unwrap();
                decoded_packets.push(decodec_packet)
            }
        });

        group.bench_function(BenchmarkId::new("new_nodrop", messages_count), |b| {
            b.iter_batched(
                || decoded_packets.clone(),
                |decoded_packets| {
                    decoded_packets
                        .iter()
                        .map(|packet| {
                            let frame = mavframe_from_packet_new(packet);

                            black_box(serde_json::to_string_pretty(&frame).unwrap())
                        })
                        .collect::<Vec<_>>()
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("old_nodrop", messages_count), |b| {
            b.iter_batched(
                || decoded_packets.clone(),
                |decoded_packets| {
                    decoded_packets
                        .iter()
                        .map(|packet| {
                            let frame = mavframe_from_packet_old(packet);

                            black_box(serde_json::to_string_pretty(&frame).unwrap())
                        })
                        .collect::<Vec<_>>()
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("new_drop", messages_count), |b| {
            b.iter_batched(
                || decoded_packets.clone(),
                |decoded_packets| {
                    decoded_packets.iter().for_each(|packet| {
                        let frame = mavframe_from_packet_new(packet);

                        let _ret = black_box(serde_json::to_string_pretty(&frame).unwrap());
                    })
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("old_drop", messages_count), |b| {
            b.iter_batched(
                || decoded_packets.clone(),
                |decoded_packets| {
                    decoded_packets.iter().for_each(|packet| {
                        let frame = mavframe_from_packet_old(packet);

                        let _ret = black_box(serde_json::to_string_pretty(&frame).unwrap());
                    })
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn benchmark_from_json_string_to_frame(c: &mut Criterion) {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("from_json_string_to_frame");
    group
        .confidence_level(0.95)
        .sample_size(100)
        .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for messages_count in &vec![1, 100, 10000] {
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

        group.bench_function(BenchmarkId::new("new_nodrop", messages_count), |b| {
            b.iter_batched(
                || json_strings.clone(),
                |json_strings| {
                    json_strings
                        .iter()
                        .map(|json_string| black_box(mavframe_from_string_new(json_string)))
                        .collect::<Vec<_>>()
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("old_nodrop", messages_count), |b| {
            b.iter_batched(
                || json_strings.clone(),
                |json_strings| {
                    json_strings
                        .iter()
                        .map(|json_string| black_box(mavframe_from_string_old(json_string)))
                        .collect::<Vec<_>>()
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("new_drop", messages_count), |b| {
            b.iter_batched(
                || json_strings.clone(),
                |json_strings| {
                    json_strings.iter().for_each(|json_string| {
                        let _ret = black_box(mavframe_from_string_new(json_string));
                    })
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("old_drop", messages_count), |b| {
            b.iter_batched(
                || json_strings.clone(),
                |json_strings| {
                    json_strings.iter().for_each(|json_string| {
                        let _ret = black_box(mavframe_from_string_old(json_string));
                    })
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn benchmark_from_json_string_to_packet(c: &mut Criterion) {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("from_json_string_to_packet");
    group
        .confidence_level(0.95)
        .sample_size(100)
        .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for messages_count in &vec![1, 100, 10000] {
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

        group.bench_function(BenchmarkId::new("new_nodrop", messages_count), |b| {
            b.iter_batched(
                || json_strings.clone(),
                |json_strings| {
                    json_strings
                        .iter()
                        .map(|json_string| black_box(packet_from_string_new(json_string)))
                        .collect::<Vec<_>>()
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("old_nodrop", messages_count), |b| {
            b.iter_batched(
                || json_strings.clone(),
                |json_strings| {
                    json_strings
                        .iter()
                        .map(|json_string| black_box(packet_from_string_old(json_string)))
                        .collect::<Vec<_>>()
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("new_drop", messages_count), |b| {
            b.iter_batched(
                || json_strings.clone(),
                |json_strings| {
                    json_strings.iter().for_each(|json_string| {
                        let _ret = black_box(packet_from_string_new(json_string));
                    })
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("old_drop", messages_count), |b| {
            b.iter_batched(
                || json_strings.clone(),
                |json_strings| {
                    json_strings.iter().for_each(|json_string| {
                        let _ret = black_box(packet_from_string_old(json_string));
                    })
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

#[inline(always)]
fn mavframe_from_string_new(
    json_string: &str,
) -> mavlink_codec::mav_types::mav_frame::serde_impl::MavFrameSemanticModel {
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

#[inline(always)]
fn mavframe_from_packet_new(packet: &Packet) -> mavlink_codec::mav_types::mav_frame::MavFrame {
    mavlink_codec::mav_types::mav_frame::MavFrame::from(packet)
}

#[inline(always)]
fn mavframe_from_packet_old(packet: &Packet) -> MAVLinkMessage<mavlink::ardupilotmega::MavMessage> {
    let header = mavlink::MavHeader {
        sequence: packet.sequence(),
        system_id: packet.system_id(),
        component_id: packet.component_id(),
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

    MAVLinkMessage { header, message }
}

criterion_group!(
    benches,
    benchmark_packet_to_mavframe,
    benchmark_packet_to_json_value,
    benchmark_packet_to_json_string,
    benchmark_from_json_string_to_frame,
    benchmark_from_json_string_to_packet,
);
criterion_main!(benches);
