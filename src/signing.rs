use std::{collections::HashMap, time::SystemTime};

use sha2::{Digest, Sha256};

use crate::v2::{self, MAVLINK_IFLAG_SIGNED};

/// Size in bytes of the MAVLink 2 signing shared secret key.
pub const SECRET_KEY_SIZE: usize = 32;

/// Signing data owned by a codec instance: the configuration plus the mutable replay-protection
/// state.
///
/// Unlike rust-mavlink's `SigningData`, this holds no `Mutex`: the codec verifies through
/// `Decoder::decode(&mut self)`, so the state is mutated through exclusive `&mut` access.
pub struct SigningData {
    config: SigningConfig,
    state: SigningState,
}

/// MAVLink 2 message-signing configuration as defined in
/// <https://mavlink.io/en/guide/message_signing.html>.
#[derive(Debug, Clone)]
pub struct SigningConfig {
    /// Shared secret key.
    pub secret_key: [u8; SECRET_KEY_SIZE],
    /// Accept unsigned frames instead of rejecting them.
    pub allow_unsigned: bool,
}

struct SigningState {
    timestamp: u64,
    stream_timestamps: HashMap<(u8, u8, u8), u64>,
}

impl SigningData {
    pub fn new(config: SigningConfig) -> Self {
        Self {
            config,
            state: SigningState {
                timestamp: 0,
                stream_timestamps: HashMap::new(),
            },
        }
    }

    /// Verifies the signature of a complete MAVLink 2 frame in place.
    ///
    /// Returns whether the frame is accepted. Unsigned frames are accepted only when
    /// `allow_unsigned` is set. Implements the same SHA-256 signature and per-stream timestamp
    /// replay window as the MAVLink 2 signing specification.
    pub fn verify_signature(&mut self, frame: &[u8]) -> bool {
        if *v2::incompat_flags(&frame) & MAVLINK_IFLAG_SIGNED == 0 {
            return self.config.allow_unsigned;
        }

        let (Some(link_id), Some(timestamp), Some(signature_value)) = (
            v2::signature_link_id(&frame),
            v2::signature_timestamp_u64(&frame),
            v2::signature_value(&frame),
        ) else {
            return self.config.allow_unsigned;
        };

        let stream_key = (link_id, *v2::sysid(&frame), *v2::compid(&frame));

        self.state.timestamp = self.state.timestamp.max(Self::current_timestamp());
        match self.state.stream_timestamps.get(&stream_key) {
            // Reject a timestamp that is not strictly newer than the last accepted one.
            Some(stream_timestamp) if timestamp <= *stream_timestamp => return false,
            // Reject a brand-new stream whose timestamp is more than a minute stale.
            None if timestamp + 60 * 1000 * 100 < self.state.timestamp => return false,
            _ => {}
        }

        // The hashed region is contiguous (STX, header, payload, CRC, link id, timestamp) and
        // already lives in `frame`, so verification is zero-copy. The trailing bytes (everything
        // past the signed region) are the signature value being checked.
        let signed_len = v2::packet_size(&frame) - signature_value.len();
        let mut hasher = Sha256::new();
        hasher.update(self.config.secret_key);
        hasher.update(&frame[..signed_len]);
        let accepted = hasher.finalize()[..signature_value.len()] == *signature_value;

        if accepted {
            self.state.stream_timestamps.insert(stream_key, timestamp);
            self.state.timestamp = self.state.timestamp.max(timestamp);
        }
        accepted
    }

    fn current_timestamp() -> u64 {
        // Fallback to 0 if the system time appears to be before epoch.
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|n| n.as_micros())
            .unwrap_or(0);
        // Offset from 1st January 2015 GMT, in units of 10 microseconds.
        (now.saturating_sub(1420070400u128 * 1000000u128) / 10u128) as u64
    }
}
