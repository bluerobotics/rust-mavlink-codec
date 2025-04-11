pub mod serde_impl;

#[derive(Debug, Clone, PartialEq)]
pub struct MavHeader {
    pub buffer: bytes::Bytes,
}

impl MavHeader {
    pub fn new(bytes: bytes::Bytes) -> Self {
        Self { buffer: bytes }
    }

    pub fn sequence(&self) -> u8 {
        self.buffer[0]
    }

    pub fn system_id(&self) -> u8 {
        self.buffer[1]
    }

    pub fn component_id(&self) -> u8 {
        self.buffer[2]
    }

    pub fn message_id(&self) -> Option<u32> {
        match self.buffer.len() {
            4 => Some(u32::from_le_bytes([self.buffer[3], 0, 0, 0])),
            6 => Some(u32::from_le_bytes([
                self.buffer[3],
                self.buffer[4],
                self.buffer[5],
                0,
            ])),
            _ => None,
        }
    }
}
