use std::hint::black_box;

use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
    Throughput,
};
use mavlink::Message;
use mavlink_codec::{codec::MavlinkCodec, v2::V2Packet};
use rand::{prelude::StdRng, SeedableRng};
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, FramedRead};

#[cfg(feature = "bench-c-reference")]
mod c_reference {
    use std::{ffi::c_int, mem::MaybeUninit, ptr};

    pub struct BenchState(*mut std::ffi::c_void);

    extern "C" {
        fn mavlink_codec_bench_state_new() -> *mut std::ffi::c_void;
        fn mavlink_codec_bench_state_reset(state: *mut std::ffi::c_void);
        fn mavlink_codec_bench_state_free(state: *mut std::ffi::c_void);
        fn mavlink_codec_bench_state_decode(
            state: *mut std::ffi::c_void,
            data: *const u8,
            len: usize,
        ) -> c_int;
        fn mavlink_codec_bench_state_last_message(
            state: *const std::ffi::c_void,
        ) -> *const std::ffi::c_void;
    }

    impl BenchState {
        pub fn new() -> Self {
            let state = unsafe { mavlink_codec_bench_state_new() };
            assert!(!state.is_null());
            Self(state)
        }

        pub fn reset(&mut self) {
            unsafe { mavlink_codec_bench_state_reset(self.0) };
        }

        pub fn decode(&mut self, data: &[u8]) -> c_int {
            unsafe { mavlink_codec_bench_state_decode(self.0, data.as_ptr(), data.len()) }
        }

        pub fn last_message_word(&self) -> u64 {
            let msg = unsafe { mavlink_codec_bench_state_last_message(self.0) };
            let mut word = MaybeUninit::<u64>::uninit();
            unsafe {
                ptr::copy_nonoverlapping(msg.cast(), word.as_mut_ptr(), 1);
                word.assume_init()
            }
        }
    }

    impl Drop for BenchState {
        fn drop(&mut self) {
            unsafe { mavlink_codec_bench_state_free(self.0) };
        }
    }
}

const SIGNING_KEY: [u8; mavlink_codec::signing::SECRET_KEY_SIZE] = [
    0x00, 0x01, 0xf2, 0xe3, 0xd4, 0xc5, 0xb6, 0xa7, 0x98, 0x00, 0x70, 0x76, 0x34, 0x32, 0x00, 0x16,
    0x22, 0x42, 0x00, 0xcc, 0xff, 0x7a, 0x00, 0x52, 0x75, 0x73, 0x74, 0x00, 0x4d, 0x41, 0x56, 0xb3,
];

fn add_random_v2_message(buf: &mut Vec<u8>, rng: &mut StdRng) {
    use rand::Rng;

    use mavlink::dialects::ardupilotmega::*;

    let header = mavlink::MavHeader {
        system_id: rng.gen_range(1..255),
        component_id: rng.gen_range(1..255),
        sequence: rng.gen_range(0..255),
    };

    loop {
        let message_id = rng.gen_range(0..2 ^ 24);
        if let Some(data) = MavMessage::default_message_from_id(message_id) {
            if mavlink::write_v2_msg(buf, header, &data).is_ok() {
                break;
            }
        };
    }
}

fn add_random_signed_v2_message(
    buf: &mut Vec<u8>,
    rng: &mut StdRng,
    signing: &mavlink::SigningData,
) {
    use rand::Rng;

    use mavlink::dialects::ardupilotmega::*;

    let header = mavlink::MavHeader {
        system_id: rng.gen_range(1..255),
        component_id: rng.gen_range(1..255),
        sequence: rng.gen_range(0..255),
    };

    loop {
        let message_id = rng.gen_range(0..2 ^ 24);
        if let Some(data) = MavMessage::default_message_from_id(message_id) {
            let mut raw = mavlink::MAVLinkV2MessageRaw::new();
            raw.serialize_message_for_signing(header, &data);
            signing.sign_message(&mut raw);
            buf.extend_from_slice(raw.raw_bytes());
            break;
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

                b.to_async(&rt).iter_batched(
                    || {
                        let reader = mavlink::peek_reader::PeekReader::new(&buf[..]);

                        reader
                    },
                    |mut reader| async move {
                        for _ in 0..messages_count {
                            let _msg = black_box(
                                mavlink::read_v2_raw_message::<
                                    mavlink::dialects::ardupilotmega::MavMessage,
                                    _,
                                >(&mut reader)
                                .unwrap(),
                            );
                        }
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rust-mavlink-async", messages_count),
            messages_count,
            |b, &messages_count| {
                let buf = buf.clone();

                b.to_async(&rt).iter_batched(
                    || {
                        let reader = mavlink::async_peek_reader::AsyncPeekReader::new(&buf[..]);

                        reader
                    },
                    |mut reader| async move {
                        for _ in 0..messages_count {
                            let _msg = black_box(
                                mavlink::read_v2_raw_message_async::<
                                    mavlink::dialects::ardupilotmega::MavMessage,
                                    _,
                                >(&mut reader)
                                .await
                                .unwrap(),
                            );
                        }
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("decoder-decode", messages_count),
            messages_count,
            |b, &messages_count| {
                let buf = buf.clone(); // Reset buffer each time

                b.to_async(&rt).iter_batched(
                    || {
                        let buf = bytes::BytesMut::from(buf.as_slice());
                        let codec =
                            MavlinkCodec::<true, true, false, false, false, false, false>::default(
                            );

                        (buf, codec)
                    },
                    |(mut buf, mut codec)| async move {
                        for _ in 0..messages_count {
                            let _msg = black_box(codec.decode(&mut buf).unwrap().unwrap());
                        }
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("decoder-framed.next", messages_count),
            messages_count,
            |b, &messages_count| {
                b.to_async(&rt).iter_batched(
                    || {
                        let codec =
                            MavlinkCodec::<true, true, false, false, false, false, false>::default(
                            );
                        let framed = FramedRead::new(buf.as_slice(), codec);

                        framed
                    },
                    |mut framed| async move {
                        for _ in 0..messages_count {
                            let _msg = black_box(framed.next().await.unwrap().unwrap());
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        #[cfg(feature = "bench-c-reference")]
        group.bench_with_input(
            BenchmarkId::new("c_library_v2", messages_count),
            messages_count,
            |b, &messages_count| {
                let buf = buf.clone();

                b.iter_batched(
                    || {
                        let mut state = c_reference::BenchState::new();
                        state.reset();
                        state
                    },
                    |mut state| {
                        let count = state.decode(&buf);
                        black_box(count);
                        black_box(state.last_message_word());
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn benchmark_decode_signed(c: &mut Criterion) {
    let seed = 42;
    println!("Using seed {seed:?}");
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut group = c.benchmark_group("decode_signed");
    group.confidence_level(0.95).sample_size(100);

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    group.plot_config(plot_config);

    let messages_counts = vec![1, 5, 10, 50, 100, 500, 1000, 5000, 10000, 50000, 100000];

    let rt = tokio::runtime::Runtime::new().unwrap();

    for messages_count in &messages_counts {
        group.throughput(Throughput::Elements(*messages_count));

        let mut buf: Vec<u8> =
            Vec::with_capacity(V2Packet::MAX_PACKET_SIZE * *messages_count as usize);
        let signing = mavlink::SigningData::from_config(mavlink::SigningConfig::new(
            SIGNING_KEY,
            0,
            true,
            false,
        ));
        for _ in 0..*messages_count {
            add_random_signed_v2_message(&mut buf, &mut rng, &signing);
        }

        group.bench_with_input(
            BenchmarkId::new("rust-mavlink", messages_count),
            messages_count,
            |b, &messages_count| {
                let buf = buf.clone();

                b.to_async(&rt).iter_batched(
                    || {
                        let reader = mavlink::peek_reader::PeekReader::new(&buf[..]);
                        let signing = mavlink::SigningData::from_config(
                            mavlink::SigningConfig::new(SIGNING_KEY, 0, false, false),
                        );

                        (reader, signing)
                    },
                    |(mut reader, signing)| async move {
                        for _ in 0..messages_count {
                            let msg = mavlink::read_v2_raw_message::<
                                mavlink::dialects::ardupilotmega::MavMessage,
                                _,
                            >(&mut reader)
                            .unwrap();
                            black_box(signing.verify_signature(&msg));
                        }
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rust-mavlink-async", messages_count),
            messages_count,
            |b, &messages_count| {
                let buf = buf.clone();

                b.to_async(&rt).iter_batched(
                    || {
                        let reader = mavlink::async_peek_reader::AsyncPeekReader::new(&buf[..]);
                        let signing = mavlink::SigningData::from_config(
                            mavlink::SigningConfig::new(SIGNING_KEY, 0, false, false),
                        );

                        (reader, signing)
                    },
                    |(mut reader, signing)| async move {
                        for _ in 0..messages_count {
                            let msg = mavlink::read_v2_raw_message_async::<
                                mavlink::dialects::ardupilotmega::MavMessage,
                                _,
                            >(&mut reader)
                            .await
                            .unwrap();
                            black_box(signing.verify_signature(&msg));
                        }
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("decoder-decode", messages_count),
            messages_count,
            |b, &messages_count| {
                let buf = buf.clone();

                b.to_async(&rt).iter_batched(
                    || {
                        let buf = bytes::BytesMut::from(buf.as_slice());
                        let codec = MavlinkCodec::<false, true, false, false, false, false, true>::with_signing(
                            mavlink_codec::signing::SigningData::new(
                                mavlink_codec::signing::SigningConfig {
                                    secret_key: SIGNING_KEY,
                                    allow_unsigned: false,
                                },
                            ),
                        );

                        (buf, codec)
                    },
                    |(mut buf, mut codec)| async move {
                        for _ in 0..messages_count {
                            let _msg = black_box(codec.decode(&mut buf).unwrap().unwrap());
                        }
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("decoder-framed.next", messages_count),
            messages_count,
            |b, &messages_count| {
                b.to_async(&rt).iter_batched(
                    || {
                        let codec = MavlinkCodec::<false, true, false, false, false, false, true>::with_signing(
                            mavlink_codec::signing::SigningData::new(
                                mavlink_codec::signing::SigningConfig {
                                    secret_key: SIGNING_KEY,
                                    allow_unsigned: false,
                                },
                            ),
                        );
                        let framed = FramedRead::new(buf.as_slice(), codec);

                        framed
                    },
                    |mut framed| async move {
                        for _ in 0..messages_count {
                            let _msg = black_box(framed.next().await.unwrap().unwrap());
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_decode, benchmark_decode_signed);
criterion_main!(benches);
