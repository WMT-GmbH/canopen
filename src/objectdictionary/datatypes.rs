use alloc::vec::Vec;

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
}

impl From<&CANOpenDataType> for Vec<u8> {
    fn from(_: &CANOpenDataType) -> Self {
        unimplemented!()
    }
}
