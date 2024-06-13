use embedded_can::{Id, StandardId};

use crate::NodeId;

pub struct Nmt {
    pub(crate) node_id: NodeId,
}

impl Nmt {
    pub const NMT_REQUEST_ID: StandardId = StandardId::ZERO;

    pub fn new(node_id: NodeId) -> Nmt {
        Nmt { node_id }
    }

    pub fn on_message<F: embedded_can::Frame>(
        &mut self,
        frame: &F,
        callback: &mut impl NmtCallback,
    ) -> Option<F> {
        if frame.id() != Id::Standard(Self::NMT_REQUEST_ID) {
            return None;
        }
        match frame.data() {
            &[command_code, node_id] if node_id == 0 || node_id == self.node_id.raw() => {
                self.on_request(command_code, callback)
            }
            _ => None,
        }
    }

    pub fn on_request<F: embedded_can::Frame>(
        &mut self,
        command_code: u8,
        callback: &mut impl NmtCallback,
    ) -> Option<F> {
        if command_code == 129 {
            callback.on_reset_request()
        }

        None
    }
}

pub trait NmtCallback {
    fn on_reset_request(&mut self);
}
