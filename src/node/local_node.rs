use crate::network::Network;
use crate::objectdictionary::ObjectDictionary;
use crate::sdo::SdoServer;

pub struct LocalNode<'n, 'o> {
    pub sdo_server: SdoServer<'n, 'o>,
}

impl<'n, 'o> LocalNode<'n, 'o> {
    pub fn new(
        node_id: u8,
        network: &'n dyn Network,
        od: &'o ObjectDictionary,
    ) -> LocalNode<'n, 'o> {
        LocalNode {
            sdo_server: SdoServer::new(0x600 + node_id as u32, 0x580 + node_id as u32, network, od),
        }
    }
}
