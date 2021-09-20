pub use CANOpenDataType::*;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum CANOpenDataType<'a> {
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
    VISIBLE_STRING(&'a str),
    OCTET_STRING(&'a [u8]),
    UNICODE_STRING(&'a str),
    TIME_OF_DAY,
    TIME_DIFFERENCE,
    DOMAIN, // TODO Strings and Domain
}
