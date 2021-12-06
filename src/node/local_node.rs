use embedded_can::Id;

use crate::lss::Lss;
use crate::nmt::Nmt;
use crate::node::NodeId;
use crate::objectdictionary::ObjectDictionary;
use crate::pdo::{
    CobId, DefaultTPDO, InvalidCobId, MappedVariables, PDOCommunicationParameter, TPDO,
};
use crate::sdo::SdoServer;

pub struct CanOpenNode<'a> {
    pub sdo_server: SdoServer<'a>,
    pub lss_slave: Lss<'a>,
    pub nmt_slave: Nmt<'a>,
}

impl<'a> CanOpenNode<'a> {
    pub fn new(node_id: NodeId, od: ObjectDictionary<'a>, lss_slave: Lss<'a>) -> Self {
        CanOpenNode {
            sdo_server: SdoServer::new(node_id, od),
            lss_slave,
            nmt_slave: Nmt::default(),
        }
    }

    pub fn on_message<F: embedded_can::Frame>(&mut self, frame: &F) -> Option<F> {
        match frame.id() {
            Id::Standard(id) => {
                // ignore messages with wrong length

                if id == Nmt::NMT_REQUEST_ID {
                    match frame.data() {
                        &[command_code, node_id]
                            if node_id == 0 || node_id == self.lss_slave.node_id.raw() =>
                        {
                            return self.nmt_slave.on_request(command_code);
                        }
                        _ => {}
                    }
                } else if id == self.sdo_server.rx_cobid {
                    if let Ok(data) = frame.data().try_into() {
                        return self.sdo_server.on_request(data);
                    }
                } else if id == Lss::LSS_REQUEST_ID {
                    if let Ok(data) = frame.data().try_into() {
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
        let com = PDOCommunicationParameter::new(DefaultTPDO::TPDO1.cob_id(
            self.lss_slave.node_id,
            false,
            false,
        ));
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
        let com = PDOCommunicationParameter::new(DefaultTPDO::TPDO2.cob_id(
            self.lss_slave.node_id,
            false,
            false,
        ));
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
        let com = PDOCommunicationParameter::new(DefaultTPDO::TPDO3.cob_id(
            self.lss_slave.node_id,
            false,
            false,
        ));
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
        let com = PDOCommunicationParameter::new(DefaultTPDO::TPDO4.cob_id(
            self.lss_slave.node_id,
            false,
            false,
        ));
        TPDO::new(
            self.sdo_server.od,
            com,
            MappedVariables::default(),
            cob_id_update_func,
        )
    }
}
