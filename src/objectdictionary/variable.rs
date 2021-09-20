use crate::objectdictionary::datalink::DataLink;

pub struct Variable<'a> {
    pub index: u16,
    pub subindex: u8,
    pub datalink: &'a dyn DataLink,
}

impl<'a> Variable<'a> {
    pub const fn new(index: u16, subindex: u8, datalink: &'a dyn DataLink) -> Self {
        Variable {
            index,
            subindex,
            datalink,
        }
    }
}
