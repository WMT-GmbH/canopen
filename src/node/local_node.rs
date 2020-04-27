use crate::network::Network;
use crate::objectdictionary::{DataStore, ObjectDictionary};
use crate::sdo::SdoServer;

pub struct LocalNode<'a> {
    pub sdo_server: SdoServer<'a>,
    pub data_store: DataStore,
}

impl<'a> LocalNode<'a> {
    pub fn new(node_id: u8, network: &'a dyn Network, od: &'a ObjectDictionary) -> Self {
        let data_store = DataStore::default();

        LocalNode {
            sdo_server: SdoServer::new(0x600 + node_id as u32, 0x580 + node_id as u32, network, od),
            data_store,
        }
    }

    pub fn on_message(&mut self, data: &[u8]) {
        // TODO message dispatch
        self.sdo_server.on_request(&mut self.data_store, data);
    }
}
