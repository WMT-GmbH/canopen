use alloc::vec::Vec;

pub use CANOpenDataType::*;

pub enum CANOpenDataType {
    BOOLEAN(bool),
    INTEGER8(i8),
    INTEGER16(i16),
    INTEGER32(i32),
    INTEGER64(i64),
    UNSIGNED8(u8),
    UNSIGNED16(u16),
    UNSIGNED32(u32),
    UNSIGNED64(u64),
    REAL32(f32),
    REAL64(f64),
    VISIBLE_STRING(&'static str),
    OCTET_STRING(&'static str),
    UNICODE_STRING(&'static str),
    DOMAIN, // TODO Strings and Domain
}

impl From<&CANOpenDataType> for Vec<u8> {
    fn from(data: &CANOpenDataType) -> Self {
        match data {
            BOOLEAN(data) => vec![*data as u8],
            INTEGER8(data) => data.to_le_bytes().to_vec(),
            INTEGER16(data) => data.to_le_bytes().to_vec(),
            INTEGER32(data) => data.to_le_bytes().to_vec(),
            INTEGER64(data) => data.to_le_bytes().to_vec(),
            UNSIGNED8(data) => data.to_le_bytes().to_vec(),
            UNSIGNED16(data) => data.to_le_bytes().to_vec(),
            UNSIGNED32(data) => data.to_le_bytes().to_vec(),
            UNSIGNED64(data) => data.to_le_bytes().to_vec(),
            REAL32(data) => data.to_le_bytes().to_vec(),
            REAL64(data) => data.to_le_bytes().to_vec(),
            _ => !unimplemented!(),
        }
    }
}
