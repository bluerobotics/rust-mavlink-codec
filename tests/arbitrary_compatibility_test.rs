//! Cross-checks our codec against rust-mavlink over the entire ardupilotmega dialect.
//!
//! For every message id in the compiled dialect we generate several random-payload frames
//! (via rust-mavlink's `arbitrary` support), then feed the exact same on-wire bytes into both
//! our `MavlinkCodec` and rust-mavlink's raw-message reader, asserting the two implementations
//! agree on acceptance and on every decoded field.

use bytes::BytesMut;
use rand::{prelude::StdRng, SeedableRng};
use tokio_util::codec::Decoder;

use dev_utils::{
    all_message_ids, create_random_v1_message_from_id, create_random_v2_message_from_id,
};
use mavlink::dialects::ardupilotmega::MavMessage;
use mavlink_codec::codec::MavlinkCodec;

const SEED: u64 = 42;
const ITERATIONS_PER_MESSAGE: usize = 32;

#[test]
fn v2_all_messages_match_rust_mavlink() {
    let mut rng: StdRng = SeedableRng::seed_from_u64(SEED);
    let mut compared = 0usize;

    for &(name, id) in all_message_ids() {
        for _ in 0..ITERATIONS_PER_MESSAGE {
            let raw = create_random_v2_message_from_id(&mut rng, id)
                .unwrap_or_else(|| panic!("dialect is missing v2 message {name} (id {id})"));
            let wire = raw.raw_bytes();

            // Our codec decodes the frame we just serialized.
            let mut codec =
                MavlinkCodec::<false, true, false, false, false, false, false>::default();
            let mut buf = BytesMut::with_capacity(wire.len());
            buf.extend_from_slice(wire);
            let our_packet = match codec.decode(&mut buf) {
                Ok(Some(Ok(packet))) => packet,
                other => panic!("our codec rejected v2 {name} (id {id}): {other:?}"),
            };
            assert!(
                buf.is_empty(),
                "our codec left {} trailing bytes for v2 {name} (id {id})",
                buf.len()
            );

            // rust-mavlink parses the same bytes.
            let mut reader = mavlink::peek_reader::PeekReader::new(wire);
            let their = mavlink::read_v2_raw_message::<MavMessage, _>(&mut reader)
                .unwrap_or_else(|e| panic!("rust-mavlink rejected v2 {name} (id {id}): {e:?}"));

            assert_eq!(
                our_packet.as_slice(),
                wire,
                "v2 {name} (id {id}): our frame bytes diverged from the serialized frame"
            );
            assert_eq!(
                our_packet.as_slice(),
                their.raw_bytes(),
                "v2 {name} (id {id}): our frame bytes diverged from rust-mavlink"
            );
            assert_eq!(
                our_packet.message_id(),
                id,
                "v2 {name} (id {id}): message id"
            );
            assert_eq!(
                our_packet.message_id(),
                their.message_id(),
                "v2 {name} (id {id}): message id vs rust-mavlink"
            );
            assert_eq!(
                *our_packet.system_id(),
                their.system_id(),
                "v2 {name} (id {id}): system id"
            );
            assert_eq!(
                *our_packet.component_id(),
                their.component_id(),
                "v2 {name} (id {id}): component id"
            );
            assert_eq!(
                *our_packet.sequence(),
                their.sequence(),
                "v2 {name} (id {id}): sequence"
            );
            assert_eq!(
                our_packet.payload(),
                their.payload(),
                "v2 {name} (id {id}): payload"
            );
            assert_eq!(
                our_packet.checksum(),
                their.checksum(),
                "v2 {name} (id {id}): checksum"
            );

            compared += 1;
        }
    }

    println!(
        "Compared {compared} v2 frames across {} messages",
        all_message_ids().len()
    );
}

#[test]
fn v1_all_messages_match_rust_mavlink() {
    let mut rng: StdRng = SeedableRng::seed_from_u64(SEED);
    let mut compared = 0usize;

    for &(name, id) in all_message_ids() {
        // MAVLink 1 carries the message id in a single byte, so ids above 255 are unrepresentable.
        if id > u8::MAX as u32 {
            continue;
        }

        for _ in 0..ITERATIONS_PER_MESSAGE {
            let raw = create_random_v1_message_from_id(&mut rng, id)
                .unwrap_or_else(|| panic!("dialect is missing v1 message {name} (id {id})"));
            let wire = raw.raw_bytes();

            let mut codec =
                MavlinkCodec::<true, false, false, false, false, false, false>::default();
            let mut buf = BytesMut::with_capacity(wire.len());
            buf.extend_from_slice(wire);
            let our_packet = match codec.decode(&mut buf) {
                Ok(Some(Ok(packet))) => packet,
                other => panic!("our codec rejected v1 {name} (id {id}): {other:?}"),
            };
            assert!(
                buf.is_empty(),
                "our codec left {} trailing bytes for v1 {name} (id {id})",
                buf.len()
            );

            let mut reader = mavlink::peek_reader::PeekReader::new(wire);
            let their = mavlink::read_v1_raw_message::<MavMessage, _>(&mut reader)
                .unwrap_or_else(|e| panic!("rust-mavlink rejected v1 {name} (id {id}): {e:?}"));

            assert_eq!(
                our_packet.as_slice(),
                wire,
                "v1 {name} (id {id}): our frame bytes diverged from the serialized frame"
            );
            assert_eq!(
                our_packet.as_slice(),
                their.raw_bytes(),
                "v1 {name} (id {id}): our frame bytes diverged from rust-mavlink"
            );
            assert_eq!(
                our_packet.message_id(),
                id,
                "v1 {name} (id {id}): message id"
            );
            assert_eq!(
                our_packet.message_id(),
                their.message_id() as u32,
                "v1 {name} (id {id}): message id vs rust-mavlink"
            );
            assert_eq!(
                *our_packet.system_id(),
                their.system_id(),
                "v1 {name} (id {id}): system id"
            );
            assert_eq!(
                *our_packet.component_id(),
                their.component_id(),
                "v1 {name} (id {id}): component id"
            );
            assert_eq!(
                *our_packet.sequence(),
                their.sequence(),
                "v1 {name} (id {id}): sequence"
            );
            assert_eq!(
                our_packet.payload(),
                their.payload(),
                "v1 {name} (id {id}): payload"
            );
            assert_eq!(
                our_packet.checksum(),
                their.checksum(),
                "v1 {name} (id {id}): checksum"
            );

            compared += 1;
        }
    }

    println!("Compared {compared} v1 frames");
}
