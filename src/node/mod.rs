use crate::hal;
use crate::sdo::SdoAbortedError;
use core::cell::RefCell;

pub struct Node<'a> {
    pub network: &'a RefCell<dyn hal::MyTransmitter>,
}

impl<'a> Node<'a> {
    pub fn get_data(&self, index: u16, subindex: u8) -> Result<Vec<u8>, SdoAbortedError> {
        dbg!(index, subindex);
        Ok(vec![1, 2, 3, 4])
    }
}
