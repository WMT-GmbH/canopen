use crate::network::Network;
use crate::objectdictionary::{DataStore, ObjectDictionary};
use crate::sdo::SdoServer;

pub struct LocalNode<'n, 'o> {
    pub sdo_server: SdoServer<'n, 'o>,
    pub data_store: DataStore,
}

impl<'n, 'o, 'd> LocalNode<'n, 'o> {
    pub fn new(
        node_id: u8,
        network: &'n dyn Network,
        od: &'o ObjectDictionary,
    ) -> LocalNode<'n, 'o> {
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
