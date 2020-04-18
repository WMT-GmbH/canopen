use crate::network::Network;
use crate::objectdictionary::ObjectDictionary;
use crate::sdo::SdoServer;

pub struct LocalNode<'a, 'b> {
    pub sdo_server: SdoServer<'a, 'b>,
}

impl<'a, 'b> LocalNode<'a, 'b> {
    pub fn new(
        node_id: u8,
        network: &'a dyn Network,
        od: &'b ObjectDictionary,
    ) -> LocalNode<'a, 'b> {
        LocalNode {
            sdo_server: SdoServer::new(0x600 + node_id as u32, 0x580 + node_id as u32, network, od),
        }
    }
}
