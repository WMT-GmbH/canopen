use crate::objectdictionary::ObjectDictionary;
use crate::sdo::SdoServer;
use embedded_can::{Id, StandardId};

pub struct CanOpenNode<'a, F> {
    pub sdo_server: SdoServer<'a, F>,
}

impl<'a, F: embedded_can::Frame> CanOpenNode<'a, F> {
    pub fn new(node_id: u8, object_dictionary: &'a ObjectDictionary<'a>) -> Self {
        // SAFETY: Maximum StandardId is 0x7FF, maximum node_id is 0xFF TODO should be 127
        let tx_cobid = unsafe { StandardId::new_unchecked(0x580 + node_id as u16) };
        let rx_cobid = unsafe { StandardId::new_unchecked(0x600 + node_id as u16) };

        CanOpenNode {
            sdo_server: SdoServer::new(rx_cobid, tx_cobid, object_dictionary),
        }
    }

    pub fn on_message(&mut self, frame: &F) -> Option<F> {
        match frame.id() {
            Id::Standard(id) => {
                if id == self.sdo_server.rx_cobid {
                    if let Ok(data) = frame.data().try_into() {
                        // ignore messages with wrong length
                        return self.sdo_server.on_request(data);
                    }
                }
            }
            Id::Extended(_) => {}
        }
        None
    }
}
