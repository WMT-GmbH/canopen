use embedded_can::{Id, StandardId};

use crate::{CanOpenService, NodeId, ObjectDictionary};

pub struct Nmt<'a> {
    node_id: NodeId,
    callback: Option<&'a mut dyn NmtCallback>,
}

impl<'a> Nmt<'a> {
    pub const NMT_REQUEST_ID: StandardId = StandardId::ZERO;

    pub fn new(node_id: NodeId) -> Nmt<'a> {
        Nmt {
            node_id,
            callback: None,
        }
    }

    pub fn add_callback(&mut self, callback: &'a mut dyn NmtCallback) {
        self.callback = Some(callback);
    }

    pub fn on_request<F: embedded_can::Frame>(&mut self, command_code: u8) -> Option<F> {
        if let Some(callback) = self.callback.as_mut() {
            if command_code == 129 {
                callback.on_reset_request()
            }
        }
        None
    }
}

impl<F: embedded_can::Frame, T, const N: usize> CanOpenService<F, T, N> for Nmt<'_> {
    fn on_message(&mut self, frame: &F, _: &mut ObjectDictionary<T, N>) -> Option<F> {
        if frame.id() != Id::Standard(Self::NMT_REQUEST_ID) {
            return None;
        }
        match frame.data() {
            &[command_code, node_id] if node_id == 0 || node_id == self.node_id.raw() => {
                self.on_request(command_code)
            }
            _ => None,
        }
    }
}

pub trait NmtCallback {
    fn on_reset_request(&mut self);
}
