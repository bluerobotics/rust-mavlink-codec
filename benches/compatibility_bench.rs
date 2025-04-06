use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration, Throughput,
};
use dev_utils::{create_random_v1_raw_message, create_random_v2_raw_message};
use mavlink_codec::{v1::V1Packet, v2::V2Packet};
use rand::{prelude::StdRng, SeedableRng};

fn benchmark_mavlink_compatibility_v1(c: &mut Criterion) {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("mavlink_compatibility_v1");
    group.confidence_level(0.95).sample_size(100);

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    group.plot_config(plot_config);

    let messages_counts: Vec<usize> =
        vec![1, 5, 10, 50, 100, 500, 1000, 5000, 10000, 50000, 100000];

    let rt = tokio::runtime::Runtime::new().unwrap();

    for messages_count in &messages_counts {
        group.throughput(Throughput::Elements(*messages_count as u64));

        let mut packets = Vec::with_capacity(*messages_count);
        for _ in 0..*messages_count {
            let mavlink_v2_message_raw = create_random_v1_raw_message(&mut rng);
            let packet = V1Packet::from(mavlink_v2_message_raw);
            packets.push(packet);
        }

        group.bench_with_input(
            BenchmarkId::new("old", messages_count),
            messages_count,
            |b, &_messages_count| {
                let packets = packets.clone();

                b.to_async(&rt).iter(|| async {
                    let mut packets = packets.clone();

                    while let Some(packet) = packets.pop() {
                        let _msg: mavlink::MAVLinkV1MessageRaw =
                            black_box(try_from_first_implementation_v1(packet)).unwrap();
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("new", messages_count),
            messages_count,
            |b, &_messages_count| {
                let packets = packets.clone();

                b.to_async(&rt).iter(|| async {
                    let mut packets = packets.clone();

                    while let Some(packet) = packets.pop() {
                        let _msg: mavlink::MAVLinkV1MessageRaw =
                            black_box(V1Packet::try_into(packet)).unwrap();
                    }
                })
            },
        );
    }

    group.finish();
}

fn benchmark_mavlink_compatibility_v2(c: &mut Criterion) {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("mavlink_compatibility_v2");
    group.confidence_level(0.95).sample_size(100);

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    group.plot_config(plot_config);

    let messages_counts: Vec<usize> =
        vec![1, 5, 10, 50, 100, 500, 1000, 5000, 10000, 50000, 100000];

    let rt = tokio::runtime::Runtime::new().unwrap();

    for messages_count in &messages_counts {
        group.throughput(Throughput::Elements(*messages_count as u64));

        let mut packets = Vec::with_capacity(*messages_count);
        for _ in 0..*messages_count {
            let mavlink_v2_message_raw = create_random_v2_raw_message(&mut rng);
            let packet = V2Packet::from(mavlink_v2_message_raw);
            packets.push(packet);
        }

        group.bench_with_input(
            BenchmarkId::new("old", messages_count),
            messages_count,
            |b, &_messages_count| {
                let packets = packets.clone();

                b.to_async(&rt).iter(|| async {
                    let mut packets = packets.clone();

                    while let Some(packet) = packets.pop() {
                        let _msg: mavlink::MAVLinkV2MessageRaw =
                            black_box(try_from_first_implementation_v2(packet)).unwrap();
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("new", messages_count),
            messages_count,
            |b, &_messages_count| {
                let packets = packets.clone();

                b.to_async(&rt).iter(|| async {
                    let mut packets = packets.clone();

                    while let Some(packet) = packets.pop() {
                        let _msg: mavlink::MAVLinkV2MessageRaw =
                            black_box(V2Packet::try_into(packet)).unwrap();
                    }
                })
            },
        );
    }

    group.finish();
}

fn try_from_first_implementation_v1(
    value: V1Packet,
) -> Result<mavlink::MAVLinkV1MessageRaw, mavlink::error::MessageReadError> {
    use mavlink::ardupilotmega::MavMessage;

    let mut reader = mavlink::peek_reader::PeekReader::new(value.as_slice());
    let message = mavlink::read_v1_raw_message::<MavMessage, _>(&mut reader);
    return message;
}

fn try_from_first_implementation_v2(
    value: V2Packet,
) -> Result<mavlink::MAVLinkV2MessageRaw, mavlink::error::MessageReadError> {
    use mavlink::ardupilotmega::MavMessage;

    let mut reader = mavlink::peek_reader::PeekReader::new(value.as_slice());
    let message = mavlink::read_v2_raw_message::<MavMessage, _>(&mut reader);
    return message;
}

criterion_group!(
    benches,
    benchmark_mavlink_compatibility_v1,
    benchmark_mavlink_compatibility_v2
);
criterion_main!(benches);
