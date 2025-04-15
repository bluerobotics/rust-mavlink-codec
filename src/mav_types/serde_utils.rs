use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BitsField<T> {
    pub bits: T,
}

#[macro_export]
macro_rules! define_mav_message_fields {
    (
        $type:ty, $map_name:ident, {
            $(
                $key:literal => $method:ident() => $wrap:expr
            ),* $(,)?
        }
    ) => {
        pub static $map_name: ::phf::Map<
            &'static str,
            fn(&dyn $crate::mav_types::mav_message::MavMessageFields) -> serde_json::Value
        > = ::phf::phf_map! {
            $(
                $key => {
                    fn get(msg: &dyn $crate::mav_types::mav_message::MavMessageFields) -> serde_json::Value {
                        let msg = msg.as_any().downcast_ref::<$type>()
                            .expect(concat!("downcast failed for: ", stringify!($type)));

                        let value = msg.$method();
                        $wrap(value)
                    }
                    get as fn(&dyn $crate::mav_types::mav_message::MavMessageFields) -> serde_json::Value
                }
            ),*
        };
    };
}
