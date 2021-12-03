use embedded_can::Id;

use crate::lss::LSS;
use crate::node::NodeId;
use crate::objectdictionary::ObjectDictionary;
use crate::pdo::{
    CobId, DefaultTPDO, InvalidCobId, MappedVariables, PDOCommunicationParameter, TPDO,
};
use crate::sdo::SdoServer;

pub struct CanOpenNode<'a> {
    pub sdo_server: SdoServer<'a>,
    pub lss_slave: LSS<'a>,
    node_id: NodeId,
}

impl<'a> CanOpenNode<'a> {
    pub fn new(node_id: NodeId, od: ObjectDictionary<'a>, lss_slave: LSS<'a>) -> Self {
        CanOpenNode {
            sdo_server: SdoServer::new(node_id, od),
            lss_slave,
            node_id,
        }
    }

    pub fn on_message<F: embedded_can::Frame>(&mut self, frame: &F) -> Option<F> {
        match frame.id() {
            Id::Standard(id) => {
                // ignore messages with wrong length
                if let Ok(data) = frame.data().try_into() {
                    if id == self.sdo_server.rx_cobid {
                        return self.sdo_server.on_request(data);
                    }
                    if id == LSS::LSS_REQUEST_ID {
                        return self.lss_slave.on_request(data);
                    }
                }
            }
            Id::Extended(_) => {}
        }
        None
    }

    pub fn tpdo1(
        &self,
        cob_id_update_func: fn(CobId, CobId) -> Result<CobId, InvalidCobId>,
    ) -> TPDO<'a> {
        let com =
            PDOCommunicationParameter::new(DefaultTPDO::TPDO1.cob_id(self.node_id, false, false));
        TPDO::new(
            self.sdo_server.od,
            com,
            MappedVariables::default(),
            cob_id_update_func,
        )
    }

    pub fn tpdo2(
        &self,
        cob_id_update_func: fn(CobId, CobId) -> Result<CobId, InvalidCobId>,
    ) -> TPDO<'a> {
        let com =
            PDOCommunicationParameter::new(DefaultTPDO::TPDO2.cob_id(self.node_id, false, false));
        TPDO::new(
            self.sdo_server.od,
            com,
            MappedVariables::default(),
            cob_id_update_func,
        )
    }

    pub fn tpdo3(
        &self,
        cob_id_update_func: fn(CobId, CobId) -> Result<CobId, InvalidCobId>,
    ) -> TPDO<'a> {
        let com =
            PDOCommunicationParameter::new(DefaultTPDO::TPDO3.cob_id(self.node_id, false, false));
        TPDO::new(
            self.sdo_server.od,
            com,
            MappedVariables::default(),
            cob_id_update_func,
        )
    }

    pub fn tpdo4(
        &self,
        cob_id_update_func: fn(CobId, CobId) -> Result<CobId, InvalidCobId>,
    ) -> TPDO<'a> {
        let com =
            PDOCommunicationParameter::new(DefaultTPDO::TPDO4.cob_id(self.node_id, false, false));
        TPDO::new(
            self.sdo_server.od,
            com,
            MappedVariables::default(),
            cob_id_update_func,
        )
    }
}
