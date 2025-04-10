pub mod field_types;
pub mod mav_frame;
pub mod mav_header;
pub mod mav_message;
pub mod serde_utils;

// impl TryFrom<&Packet> for MavFrame {
//     type Error = DecoderError;

//     fn try_from(value: &Packet) -> Result<Self, Self::Error> {
//         Ok(Self {
//             header: mav_header::from(value),
//             message: mav_message::from(value)?,
//         })
//     }
// }

// impl From<&Packet> for MavHeader {
//     fn from(value: &Packet) -> Self {
//         Self {
//             sequence: *value.sequence(),
//             system_id: *value.system_id(),
//             component_id: *value.component_id(),
//             message_id: Some(value.message_id()),
//         }
//     }
// }
