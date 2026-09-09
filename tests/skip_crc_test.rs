use bytes::BytesMut;
use dev_utils::{create_random_v1_raw_message, create_random_v2_raw_message};
use mavlink_codec::{codec::MavlinkCodec, error::DecoderError, Packet};
use rand::{rngs::StdRng, SeedableRng};
use tokio_util::codec::Decoder;

const SEED: u64 = 42;

type SkipV1Codec = MavlinkCodec<true, false, false, false, true, false>;
type SkipV2Codec = MavlinkCodec<false, true, false, false, true, false>;
type StrictV1Codec = MavlinkCodec<true, false, false, false, false, false>;
type StrictV2Codec = MavlinkCodec<false, true, false, false, false, false>;

fn corrupt_crc(buf: &mut [u8]) {
    let len = buf.len();
    assert!(len >= 2, "packet must have at least two bytes to corrupt");
    buf[len - 2] = buf[len - 2].wrapping_add(1);
    buf[len - 1] = buf[len - 1].wrapping_add(1);
}

#[test]
fn skip_crc_v1_decodes_valid_packet() {
    let mut rng: StdRng = SeedableRng::seed_from_u64(SEED);
    let raw = create_random_v1_raw_message(&mut rng);
    let total = raw.raw_bytes().len();
    let mut buf = BytesMut::from(raw.raw_bytes());

    let mut codec = SkipV1Codec::default();
    let decoded = codec.decode(&mut buf).unwrap();

    assert!(
        matches!(decoded, Some(Ok(Packet::V1(_)))),
        "expected Packet::V1, got {decoded:?}"
    );
    assert!(
        buf.is_empty(),
        "F3: {} of {total} bytes remained after a single successful decode",
        buf.len()
    );
    assert!(
        codec.decode(&mut buf).unwrap().is_none(),
        "F3: decoder emitted a ghost packet from bytes that should already have been consumed"
    );
}

#[test]
fn skip_crc_v2_decodes_valid_packet() {
    let mut rng: StdRng = SeedableRng::seed_from_u64(SEED);
    let raw = create_random_v2_raw_message(&mut rng);
    let total = raw.raw_bytes().len();
    let mut buf = BytesMut::from(raw.raw_bytes());

    let mut codec = SkipV2Codec::default();
    let decoded = codec.decode(&mut buf).unwrap();

    assert!(
        matches!(decoded, Some(Ok(Packet::V2(_)))),
        "expected Packet::V2, got {decoded:?}"
    );
    assert!(
        buf.is_empty(),
        "F3: {} of {total} bytes remained after a single successful decode",
        buf.len()
    );
    assert!(
        codec.decode(&mut buf).unwrap().is_none(),
        "F3: decoder emitted a ghost packet from bytes that should already have been consumed"
    );
}

#[test]
fn skip_crc_v1_accepts_corrupted_crc() {
    let mut rng: StdRng = SeedableRng::seed_from_u64(SEED);
    let raw = create_random_v1_raw_message(&mut rng);

    let mut corrupted: Vec<u8> = raw.raw_bytes().to_vec();
    corrupt_crc(&mut corrupted);

    // Skip codec must accept despite the broken CRC.
    {
        let mut buf = BytesMut::from(corrupted.as_slice());
        let mut codec = SkipV1Codec::default();
        let decoded = codec.decode(&mut buf).unwrap();
        assert!(
            matches!(decoded, Some(Ok(Packet::V1(_)))),
            "skip-CRC codec must accept a packet with corrupted CRC, got {decoded:?}"
        );
        assert!(
            buf.is_empty(),
            "F3: {} bytes remained after decoding a corrupted-CRC packet under SKIP_CRC_VALIDATION",
            buf.len()
        );
    }

    // Non-skip codec must reject with InvalidCRC, proving the toggle's scope.
    {
        let mut buf = BytesMut::from(corrupted.as_slice());
        let mut codec = StrictV1Codec::default();
        let decoded = codec.decode(&mut buf).unwrap();
        assert!(
            matches!(decoded, Some(Err(DecoderError::InvalidCRC { .. }))),
            "strict codec must reject corrupted CRC, got {decoded:?}"
        );
    }
}

#[test]
fn skip_crc_v2_accepts_corrupted_crc() {
    let mut rng: StdRng = SeedableRng::seed_from_u64(SEED);
    let raw = create_random_v2_raw_message(&mut rng);

    let mut corrupted: Vec<u8> = raw.raw_bytes().to_vec();
    corrupt_crc(&mut corrupted);

    {
        let mut buf = BytesMut::from(corrupted.as_slice());
        let mut codec = SkipV2Codec::default();
        let decoded = codec.decode(&mut buf).unwrap();
        assert!(
            matches!(decoded, Some(Ok(Packet::V2(_)))),
            "skip-CRC codec must accept a packet with corrupted CRC, got {decoded:?}"
        );
        assert!(
            buf.is_empty(),
            "F3: {} bytes remained after decoding a corrupted-CRC packet under SKIP_CRC_VALIDATION",
            buf.len()
        );
    }

    {
        let mut buf = BytesMut::from(corrupted.as_slice());
        let mut codec = StrictV2Codec::default();
        let decoded = codec.decode(&mut buf).unwrap();
        assert!(
            matches!(decoded, Some(Err(DecoderError::InvalidCRC { .. }))),
            "strict codec must reject corrupted CRC, got {decoded:?}"
        );
    }
}
