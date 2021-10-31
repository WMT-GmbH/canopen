use embedded_can::Id;

use crate::node::NodeId;
use crate::objectdictionary::ObjectDictionary;
use crate::sdo::SdoServer;

pub struct CanOpenNode<'a> {
    pub sdo_server: SdoServer<'a>,
}

impl<'a> CanOpenNode<'a> {
    pub fn new(node_id: NodeId, od: ObjectDictionary<'a>) -> Self {
        CanOpenNode {
            sdo_server: SdoServer::new(node_id, od),
        }
    }

    pub fn on_message<F: embedded_can::Frame>(&mut self, frame: &F) -> Option<F> {
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
