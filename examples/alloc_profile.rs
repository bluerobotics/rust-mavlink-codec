//! Allocation profiling harness for the MAVLink decoder.
//!
//! Drives the codec through its two hot paths with a deterministic packet
//! stream and records `(total_blocks, total_bytes, max_blocks, max_bytes,
//! curr_blocks, curr_bytes)` before and after the measured region.
//!
//! The dhat global allocator is active throughout. The JSON it writes on
//! drop (`dhat-heap.json`) is only useful when exactly one scenario runs
//! per process, so this binary takes `--scenario` and `--n` on the CLI and
//! is expected to be invoked multiple times by the outer driver.
//!
//! Usage:
//!
//! ```text
//! cargo run --release --example alloc_profile -- \
//!     --scenario framed-drop --n 100
//! ```
//!
//! Scenarios:
//!   decode-drop                standalone Decoder::decode, each Packet dropped immediately
//!   decode-retain              standalone Decoder::decode, all Packets retained until end
//!   framed-drop                FramedRead over &[u8], each Packet dropped immediately
//!   framed-retain              FramedRead over &[u8], all Packets retained until end
//!   decode-skipcrc-drop        same as decode-drop but with SKIP_CRC_VALIDATION = true
//!   framed-skipcrc-drop        same as framed-drop but with SKIP_CRC_VALIDATION = true
//!   framed-retain-broadcast    mirrors mavlink-server's default_receive_task: FramedRead ->
//!                              Arc<Protocol { origin: String, .. }> -> broadcast::channel
//!                              with concurrent subscribers draining the hub
//!   framed-retain-broadcast-arcstr
//!                              same as framed-retain-broadcast, but the Protocol carries
//!                              an Arc<str> origin cloned from a driver-owned handle
//!                              instead of re-allocating a String per packet

use std::{env, hint::black_box, sync::Arc};

use bytes::BytesMut;
use dev_utils::add_random_v2_message;
use futures::StreamExt;
use mavlink_codec::{codec::MavlinkCodec, Packet};
use rand::{rngs::StdRng, SeedableRng};
use tokio::sync::broadcast;
use tokio_util::codec::{Decoder, FramedRead};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const SEED: u64 = 42;

type Codec = MavlinkCodec<true, true, false, false, false, false, false>;
type SkipCrcCodec = MavlinkCodec<true, true, false, false, true, false, false>;

/// Mirror of `mavlink-server`'s `Protocol` wrapper with the same allocation
/// shape: a `String` origin plus the decoded `Packet`, reached by consumers
/// through `Arc<Protocol>` over a broadcast channel.
#[derive(Debug)]
#[allow(dead_code)] // fields exist to reproduce the wrapper's allocation footprint
struct Protocol {
    origin: String,
    timestamp: u64,
    packet: Packet,
}

impl Protocol {
    fn new(origin: &str, packet: Packet) -> Self {
        Self {
            origin: origin.to_string(),
            timestamp: 0,
            packet,
        }
    }
}

/// `Protocol` variant whose origin is cloned (refcount bump) from a shared
/// `Arc<str>` the driver creates once, removing the per-packet `String`
/// allocation.
#[derive(Debug)]
#[allow(dead_code)] // fields exist to reproduce the wrapper's allocation footprint
struct ProtocolArcStr {
    origin: Arc<str>,
    timestamp: u64,
    packet: Packet,
}

impl ProtocolArcStr {
    fn new(origin: &Arc<str>, packet: Packet) -> Self {
        Self {
            origin: Arc::clone(origin),
            timestamp: 0,
            packet,
        }
    }
}

fn build_stream(n: usize) -> Vec<u8> {
    let mut rng: StdRng = SeedableRng::seed_from_u64(SEED);
    let mut buf: Vec<u8> = Vec::with_capacity(n * 280);
    for _ in 0..n {
        add_random_v2_message(&mut buf, &mut rng);
    }
    buf
}

fn format_stats(label: &str, stats: dhat::HeapStats) {
    println!(
        "{label}: total_blocks={} total_bytes={} max_blocks={} max_bytes={} curr_blocks={} curr_bytes={}",
        stats.total_blocks,
        stats.total_bytes,
        stats.max_blocks,
        stats.max_bytes,
        stats.curr_blocks,
        stats.curr_bytes,
    );
}

fn measure<F: FnOnce()>(label: &str, f: F) {
    let before = dhat::HeapStats::get();
    f();
    let after = dhat::HeapStats::get();
    println!(
        "{label}: blocks_delta={} bytes_delta={} max_blocks_in_region={} max_bytes_in_region={}",
        after.total_blocks.saturating_sub(before.total_blocks),
        after.total_bytes.saturating_sub(before.total_bytes),
        after.max_blocks,
        after.max_bytes,
    );
    format_stats(&format!("{label} .before"), before);
    format_stats(&format!("{label} .after "), after);
}

fn run_decode(n: usize, retain: bool) {
    let stream = build_stream(n);

    // The BytesMut and Codec construction is part of the measured region
    // so we see the initial Vec::with_capacity + any setup allocation.
    let label = if retain {
        "decode-retain"
    } else {
        "decode-drop"
    };
    measure(label, || {
        let mut buf = BytesMut::from(stream.as_slice());
        let mut codec = Codec::default();
        let mut keep: Vec<Packet> = if retain {
            Vec::with_capacity(n)
        } else {
            Vec::new()
        };
        for _ in 0..n {
            let packet = black_box(codec.decode(&mut buf).unwrap().unwrap().unwrap());
            if retain {
                keep.push(packet);
            } else {
                drop(packet);
            }
        }
        black_box(keep);
    });
}

fn run_decode_skipcrc(n: usize) {
    let stream = build_stream(n);

    measure("decode-skipcrc-drop", || {
        let mut buf = BytesMut::from(stream.as_slice());
        let mut codec = SkipCrcCodec::default();
        for _ in 0..n {
            let packet = black_box(codec.decode(&mut buf).unwrap().unwrap().unwrap());
            drop(packet);
        }
    });
}

fn run_framed_skipcrc(n: usize) {
    let stream = build_stream(n);

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    measure("framed-skipcrc-drop", || {
        rt.block_on(async {
            let codec = SkipCrcCodec::default();
            let mut framed = FramedRead::new(stream.as_slice(), codec);
            for _ in 0..n {
                let packet = black_box(framed.next().await.unwrap().unwrap().unwrap());
                drop(packet);
            }
        });
    });
}

fn run_framed(n: usize, retain: bool) {
    let stream = build_stream(n);

    let label = if retain {
        "framed-retain"
    } else {
        "framed-drop"
    };
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    measure(label, || {
        rt.block_on(async {
            let codec = Codec::default();
            let mut framed = FramedRead::new(stream.as_slice(), codec);
            let mut keep: Vec<Packet> = if retain {
                Vec::with_capacity(n)
            } else {
                Vec::new()
            };
            for _ in 0..n {
                let packet = black_box(framed.next().await.unwrap().unwrap().unwrap());
                if retain {
                    keep.push(packet);
                } else {
                    drop(packet);
                }
            }
            black_box(keep);
        });
    });
}

/// Mirrors `mavlink-server`'s `default_receive_task`: decode via `FramedRead`,
/// wrap each `Packet` in `Arc::new(Protocol::new(origin, packet))`, and fan it
/// out over a `broadcast::channel` to concurrent subscribers that drain the
/// hub. The subscribers keep each `Arc<Protocol>` alive just long enough to
/// reproduce the sliding-window retention the real server imposes on the
/// codec's read buffer.
fn run_framed_retain_broadcast(n: usize) {
    const SUBSCRIBERS: usize = 2;
    const CHANNEL_CAP: usize = 1024;
    const YIELD_EVERY: usize = 32;

    let stream = build_stream(n);

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    measure("framed-retain-broadcast", || {
        rt.block_on(async {
            let (hub_sender, _) = broadcast::channel::<Arc<Protocol>>(CHANNEL_CAP);
            let rx1 = hub_sender.subscribe();
            let rx2 = hub_sender.subscribe();

            let send_fut = async {
                let codec = Codec::default();
                let mut framed = FramedRead::new(stream.as_slice(), codec);
                let origin = "127.0.0.1:14550";
                for i in 0..n {
                    let packet = black_box(framed.next().await.unwrap().unwrap().unwrap());
                    let message = Arc::new(Protocol::new(origin, packet));
                    let _ = hub_sender.send(black_box(message));
                    // The current-thread runtime never preempts a busy loop; yielding
                    // periodically lets subscribers drain so retention stays in a realistic
                    // window rather than accumulating the full stream in the channel buffer.
                    if i % YIELD_EVERY == 0 {
                        tokio::task::yield_now().await;
                    }
                }
                drop(hub_sender);
            };

            let drain = |mut rx: broadcast::Receiver<Arc<Protocol>>| async move {
                let mut count = 0usize;
                loop {
                    match rx.recv().await {
                        Ok(msg) => {
                            black_box(&*msg);
                            drop(msg);
                            count += 1;
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    }
                }
                count
            };

            let (_, c1, c2) = tokio::join!(send_fut, drain(rx1), drain(rx2));
            black_box((c1, c2, SUBSCRIBERS));
        });
    });
}

/// Same pipeline as `run_framed_retain_broadcast`, but the origin identifier
/// is an `Arc<str>` created once per driver and cloned (refcount bump) into
/// every `Protocol`, eliminating the per-packet `String` allocation.
fn run_framed_retain_broadcast_arcstr(n: usize) {
    const SUBSCRIBERS: usize = 2;
    const CHANNEL_CAP: usize = 1024;
    const YIELD_EVERY: usize = 32;

    let stream = build_stream(n);

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    measure("framed-retain-broadcast-arcstr", || {
        rt.block_on(async {
            let (hub_sender, _) = broadcast::channel::<Arc<ProtocolArcStr>>(CHANNEL_CAP);
            let rx1 = hub_sender.subscribe();
            let rx2 = hub_sender.subscribe();

            let send_fut = async {
                let codec = Codec::default();
                let mut framed = FramedRead::new(stream.as_slice(), codec);
                let origin: Arc<str> = Arc::from("127.0.0.1:14550");
                for i in 0..n {
                    let packet = black_box(framed.next().await.unwrap().unwrap().unwrap());
                    let message = Arc::new(ProtocolArcStr::new(&origin, packet));
                    let _ = hub_sender.send(black_box(message));
                    if i % YIELD_EVERY == 0 {
                        tokio::task::yield_now().await;
                    }
                }
                drop(hub_sender);
            };

            let drain = |mut rx: broadcast::Receiver<Arc<ProtocolArcStr>>| async move {
                let mut count = 0usize;
                loop {
                    match rx.recv().await {
                        Ok(msg) => {
                            black_box(&*msg);
                            drop(msg);
                            count += 1;
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    }
                }
                count
            };

            let (_, c1, c2) = tokio::join!(send_fut, drain(rx1), drain(rx2));
            black_box((c1, c2, SUBSCRIBERS));
        });
    });
}

fn parse_args() -> (String, usize) {
    let mut scenario = String::from("framed-drop");
    let mut n: usize = 100;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scenario" => scenario = args.next().expect("--scenario requires value"),
            "--n" => {
                n = args
                    .next()
                    .expect("--n requires value")
                    .parse()
                    .expect("--n must be a number");
            }
            other => panic!("unknown flag: {other}"),
        }
    }
    (scenario, n)
}

fn main() {
    let (scenario, n) = parse_args();

    // The Profiler must remain alive for the whole program to capture the
    // dhat-heap.json on drop. All allocations before this point (including
    // env::args parsing) are excluded from the measured region via the
    // before/after HeapStats deltas.
    let _profiler = dhat::Profiler::builder().build();

    println!("=== scenario={scenario} n={n} seed={SEED} ===");

    match scenario.as_str() {
        "decode-drop" => run_decode(n, false),
        "decode-retain" => run_decode(n, true),
        "framed-drop" => run_framed(n, false),
        "framed-retain" => run_framed(n, true),
        "decode-skipcrc-drop" => run_decode_skipcrc(n),
        "framed-skipcrc-drop" => run_framed_skipcrc(n),
        "framed-retain-broadcast" => run_framed_retain_broadcast(n),
        "framed-retain-broadcast-arcstr" => run_framed_retain_broadcast_arcstr(n),
        other => panic!("unknown scenario: {other}"),
    }

    let final_stats = dhat::HeapStats::get();
    format_stats("final", final_stats);
}
