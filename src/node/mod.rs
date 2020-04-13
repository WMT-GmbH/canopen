use crate::network::Network;
use crate::sdo::SdoAbortedError;

pub struct Node<'a> {
    pub network: &'a dyn Network,
}

impl<'a> Node<'a> {
    pub fn get_data(&self, index: u16, subindex: u8) -> Result<Vec<u8>, SdoAbortedError> {
        dbg!(index, subindex);
        Ok(vec![1, 2, 3, 4])
    }
}
