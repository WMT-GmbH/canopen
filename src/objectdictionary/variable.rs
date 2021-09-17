use crate::objectdictionary::CANOpenDataType;

#[derive(Debug)]
pub struct Variable {
    pub index: u16,
    pub subindex: u8,
    pub unique_id: u32,
    pub default_value: Option<CANOpenDataType>,
}

impl Variable {
    pub const fn new(index: u16, subindex: u8, default_value: Option<CANOpenDataType>) -> Self {
        Variable {
            index,
            subindex,
            unique_id: ((index as u32) << 8) + subindex as u32,
            default_value,
        }
    }
}
