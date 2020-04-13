use crate::hal;
use core::cell::RefCell;

pub trait Network {
    // TODO provide Network trait or Hal trait
    fn send_message(&self, can_id: u32, data: [u8; 8]);
}

pub struct HalNetwork<'a> {
    interface: &'a RefCell<dyn hal::MyTransmitter>,
}

impl<'a> HalNetwork<'a> {
    pub fn new(interface: &'a RefCell<dyn hal::MyTransmitter>) -> HalNetwork {
        HalNetwork { interface }
    }

    pub fn send_message(&self, can_id: u32, data: [u8; 8]) {
        // TODO check if borrow will succeed
        self.interface.borrow_mut().transmit(can_id, &data).ok();
    }
}
