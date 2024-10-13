use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration, Throughput,
};
use mavlink::Message;
use mavlink_codec::{codec::MavlinkCodec, v2::V2Packet};
use rand::{prelude::StdRng, SeedableRng};
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, FramedRead};

fn add_random_v2_message(buf: &mut Vec<u8>, rng: &mut StdRng) {
    use rand::Rng;

    use mavlink::ardupilotmega::*;

    let header = mavlink::MavHeader {
        system_id: rng.gen_range(1..255),
        component_id: rng.gen_range(1..255),
        sequence: rng.gen_range(0..255),
    };

    loop {
        let message_id = rng.gen_range(0..2 ^ 24);
        if let Ok(data) = MavMessage::default_message_from_id(message_id) {
            if mavlink::write_v2_msg(buf, header, &data).is_ok() {
                break;
            }
        };
    }
}

fn benchmark_decode(c: &mut Criterion) {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("decode");
    group.confidence_level(0.95).sample_size(100);

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    group.plot_config(plot_config);

    let messages_counts = vec![1, 5, 10, 50, 100, 500, 1000, 5000, 10000, 50000, 100000];

    let rt = tokio::runtime::Runtime::new().unwrap();

    for messages_count in &messages_counts {
        group.throughput(Throughput::Elements(*messages_count));

        let mut buf: Vec<u8> =
            Vec::with_capacity(V2Packet::MAX_PACKET_SIZE * *messages_count as usize);
        for _ in 0..*messages_count {
            add_random_v2_message(&mut buf, &mut rng);
        }

        group.bench_with_input(
            BenchmarkId::new("rust-mavlink", messages_count),
            messages_count,
            |b, &messages_count| {
                let buf = buf.clone();

                b.to_async(&rt).iter(|| async {
                    let mut reader = mavlink::peek_reader::PeekReader::new(&buf[..]);

                    for _ in 0..messages_count {
                        let _msg = black_box(
                            mavlink::read_v2_raw_message::<mavlink::ardupilotmega::MavMessage, _>(
                                &mut reader,
                            )
                            .unwrap(),
                        );
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rust-mavlink-async", messages_count),
            messages_count,
            |b, &messages_count| {
                let buf = buf.clone();

                b.to_async(&rt).iter(|| async {
                    let mut reader = mavlink::async_peek_reader::AsyncPeekReader::new(&buf[..]);

                    for _ in 0..messages_count {
                        let _msg = black_box(
                            mavlink::read_v2_raw_message_async::<
                                mavlink::ardupilotmega::MavMessage,
                                _,
                            >(&mut reader)
                            .await
                            .unwrap(),
                        );
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("decoder-decode", messages_count),
            messages_count,
            |b, &messages_count| {
                let buf = buf.clone(); // Reset buffer each time

                b.to_async(&rt).iter(|| async {
                    let mut buf = bytes::BytesMut::from(buf.as_slice());
                    let mut codec =
                        MavlinkCodec::<true, true, false, false, false, false>::default();

                    for _ in 0..messages_count {
                        let _msg = black_box(codec.decode(&mut buf).unwrap().unwrap());
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("decoder-framed.next", messages_count),
            messages_count,
            |b, &messages_count| {
                let buf = buf.clone();

                b.to_async(&rt).iter(|| async {
                    let codec = MavlinkCodec::<true, true, false, false, false, false>::default();
                    let mut framed = FramedRead::new(buf.as_slice(), codec);

                    for _ in 0..messages_count {
                        let _msg = black_box(framed.next().await.unwrap().unwrap());
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_decode);
criterion_main!(benches);
