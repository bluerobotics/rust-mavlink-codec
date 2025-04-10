pub mod serde_impl;

use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use serde_derive::{Deserialize as DeriveDeserialize, Serialize as DeriveSerialize};

use crate::{error::DecoderError, Packet};

use super::mav_header::MavHeader;
use super::mav_message::MavMessage;

#[derive(Debug, Clone, PartialEq)]
pub struct MavFrame {
    packet: Packet,
}

impl MavFrame {
    pub fn header() -> MavHeader {
        todo!()
    }

    pub fn message() -> MavMessage {
        todo!()
    }
}
