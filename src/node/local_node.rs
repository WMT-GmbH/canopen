use crate::network::Network;
use crate::objectdictionary::ObjectDictionary;
use crate::sdo::SdoServer;

pub struct LocalNode<'a, N: Network> {
    pub sdo_server: SdoServer<'a, N>,
}

impl<'a, N: Network> LocalNode<'a, N> {
    pub fn new(node_id: u8, network: &'a N, object_dictionary: &'a ObjectDictionary<'a>) -> Self {
        LocalNode {
            sdo_server: SdoServer::new(
                0x600 + node_id as u32,
                0x580 + node_id as u32,
                network,
                object_dictionary,
            ),
        }
    }

    pub fn on_message(&mut self, data: &[u8]) {
        // TODO message dispatch
        if let Ok(data) = data.try_into() {
            // ignore messages with wrong length
            self.sdo_server.on_request(data)
        }
    }
}
