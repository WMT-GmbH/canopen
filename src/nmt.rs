use embedded_can::{Id, StandardId};

use crate::NodeId;

const START_REMOTE_NODE: u8 = 0x01;
const STOP_REMOTE_NODE: u8 = 0x02;
const ENTER_PRE_OPERATIONAL: u8 = 0x80;
const RESET_NODE: u8 = 0x81;
const RESET_COMMUNICATION: u8 = 0x82;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum NmtRequest {
    StartRemoteNode = START_REMOTE_NODE,
    StopRemoteNode = STOP_REMOTE_NODE,
    EnterPreOperational = ENTER_PRE_OPERATIONAL,
    ResetNode = RESET_NODE,
    ResetCommunication = RESET_COMMUNICATION,
}

impl NmtRequest {
    pub fn from_u8(value: u8) -> Option<NmtRequest> {
        match value {
            START_REMOTE_NODE => Some(NmtRequest::StartRemoteNode),
            STOP_REMOTE_NODE => Some(NmtRequest::StopRemoteNode),
            ENTER_PRE_OPERATIONAL => Some(NmtRequest::EnterPreOperational),
            RESET_NODE => Some(NmtRequest::ResetNode),
            RESET_COMMUNICATION => Some(NmtRequest::ResetCommunication),
            _ => None,
        }
    }

    pub fn next_state(self) -> NmtState {
        match self {
            NmtRequest::StartRemoteNode => NmtState::Operational,
            NmtRequest::StopRemoteNode => NmtState::Stopped,
            NmtRequest::EnterPreOperational => NmtState::PreOperational,
            NmtRequest::ResetNode => NmtState::Initialisation,
            NmtRequest::ResetCommunication => NmtState::Initialisation,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NmtState {
    Initialisation = 0,
    PreOperational = 127,
    Operational = 5,
    Stopped = 4,
}

pub struct Nmt {
    pub(crate) node_id: NodeId,
    pub state: NmtState,
}

impl Nmt {
    pub const NMT_REQUEST_ID: StandardId = StandardId::ZERO;

    pub fn new(node_id: NodeId) -> Nmt {
        Nmt {
            node_id,
            state: NmtState::Initialisation,
        }
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
        let nmt_request = NmtRequest::from_u8(command_code)?;
        self.state = callback.on_nmt_request(nmt_request);
        None
    }

    pub fn boot_up_message<F: embedded_can::Frame>(&mut self) -> F {
        let node_id = unsafe { StandardId::new_unchecked(0x700 + self.node_id.raw() as u16) };
        let data = [0x00];
        F::new(node_id, &data).expect("data should fit")
    }
}

pub trait NmtCallback {
    /// Handle the NMT request and return the new state.
    fn on_nmt_request(&mut self, nmt_request: NmtRequest) -> NmtState {
        nmt_request.next_state()
    }
}
