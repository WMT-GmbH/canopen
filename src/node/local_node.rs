use crate::network::Network;
use crate::objectdictionary::{DataStore, ObjectDictionary};
use crate::sdo::SdoServer;

pub struct LocalNode<'a, N: Network> {
    pub sdo_server: SdoServer<'a, N>,
    pub data_store: DataStore,
}

impl<'a, N: Network> LocalNode<'a, N> {
    pub fn new(node_id: u8, network: &'a N, od: &'a ObjectDictionary<'a>) -> Self {
        let data_store = DataStore::default();

        LocalNode {
            sdo_server: SdoServer::new(0x600 + node_id as u32, 0x580 + node_id as u32, network, od),
            data_store,
        }
    }

    pub fn on_message(&mut self, data: &[u8]) {
        // TODO message dispatch
        if let Ok(data) = data.try_into() {
            // ignore messages with wrong length
            self.sdo_server.on_request(data, &mut self.data_store)
        }
    }
}
