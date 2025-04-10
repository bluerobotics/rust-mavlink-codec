pub mod serde_impl;

use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use serde_derive::{Deserialize as DeriveDeserialize, Serialize as DeriveSerialize};

use crate::{error::DecoderError, Packet};

#[derive(Debug, Clone, PartialEq)]
pub struct MavHeader {
    buffer: bytes::Bytes,
}

impl MavHeader {
    pub fn sequence(&self) -> u8 {
        todo!()
    }
    pub fn system_id(&self) -> u8 {
        todo!()
    }
    pub fn component_id(&self) -> u8 {
        todo!()
    }
    pub fn message_id(&self) -> u32 {
        todo!()
    }
}
