use core::cell::Cell;
use core::num::NonZeroUsize;

use crate::objectdictionary::datalink::{DataLink, ReadStream, WriteStream};
use crate::sdo::SDOAbortCode;

pub struct Variable<'a> {
    pub index: u16,
    pub subindex: u8,
    pub datalink: Cell<&'a dyn DataLink>,
}

impl<'a> Variable<'a> {
    pub const fn new(index: u16, subindex: u8, datalink: &'a dyn DataLink) -> Self {
        Variable {
            index,
            subindex,
            datalink: Cell::new(datalink),
        }
    }

    #[inline]
    pub fn size(&self) -> Option<NonZeroUsize> {
        self.datalink.get().size(self.index, self.subindex)
    }

    #[inline]
    pub fn read(&self, read_stream: &mut ReadStream<'_>) -> Result<(), SDOAbortCode> {
        self.datalink.get().read(read_stream)
    }

    #[inline]
    pub fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        self.datalink.get().write(write_stream)
    }
}
