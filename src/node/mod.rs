use crate::sdo::SdoAbortedError;

#[derive(Debug)]
pub struct Node{}

impl Node{
    pub fn get_data(&self, index: u16, subindex: u8) -> Result<Vec<u8>, SdoAbortedError>{
        println!("get => index: {} subindex: {}", index, subindex);
        Ok(vec!(1, 2, 3, 4))
    }
}