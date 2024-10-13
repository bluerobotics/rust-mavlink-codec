use thiserror::Error;

use std::io;

#[derive(Error, Debug)]
pub enum DecoderError {
    #[error("invalid System ID: {sysid}")]
    InvalidSystemID { sysid: u8 },

    #[error("invalid Component ID: {compid}")]
    InvalidComponentID { compid: u8 },

    #[error("found incompatible flags in {incompat_flags}")]
    Incompatible { incompat_flags: u8 },

    #[error("unknown Message ID")]
    UnknownMessageID { msgid: u32 },

    #[error("invalid CRC: expected {expected_crc}, calculated {calculated_crc}")]
    InvalidCRC {
        expected_crc: u16,
        calculated_crc: u16,
    },

    #[error("io error")]
    Io(#[from] io::Error),

    #[error("unknown error")]
    Unknown,
}
