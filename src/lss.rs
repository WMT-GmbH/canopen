use embedded_can::{Id, StandardId};

use crate::CanOpenService;
use crate::NodeId;

type RequestResult = Option<[u8; 8]>;

const SWITCH_GLOBAL: u8 = 0x04;
const CONFIGURE_NODE_ID: u8 = 0x11;
const CONFIGURE_BIT_TIMING: u8 = 0x13;
const ACTIVATE_BIT_TIMING: u8 = 0x15;
const STORE_CONFIGURATION: u8 = 0x17;

const SWITCH_SELECTIVE_VENDOR_ID: u8 = 0x40;
const SWITCH_SELECTIVE_PRODUCT_CODE: u8 = 0x41;
const SWITCH_SELECTIVE_REVISION_NUMBER: u8 = 0x42;
const SWITCH_SELECTIVE_SERIAL_NUMBER: u8 = 0x43;
const SWITCH_SELECTIVE_SERIAL_RESPONSE: u8 = 0x44;

const IDENTIFY_VENDOR_ID: u8 = 0x46;
const IDENTIFY_PRODUCT_CODE: u8 = 0x47;
const IDENTIFY_REVISION_NUMBER_LOW: u8 = 0x48;
const IDENTIFY_REVISION_NUMBER_HIGH: u8 = 0x49;
const IDENTIFY_SERIAL_NUMBER_LOW: u8 = 0x4A;
const IDENTIFY_SERIAL_NUMBER_HIGH: u8 = 0x4B;

const IDENTIFY_RESPONSE: u8 = 0x4F;

const FAST_SCAN: u8 = 0x51;

const INQUIRE_VENDOR_ID: u8 = 0x5A;
const INQUIRE_PRODUCT_CODE: u8 = 0x5B;
const INQUIRE_REVISION_NUMBER: u8 = 0x5C;
const INQUIRE_SERIAL_NUMBER: u8 = 0x5D;

const INQUIRE_NODE_ID: u8 = 0x5E;

const LSS_OK: u8 = 0x00;
const LSS_GENERIC_ERROR: u8 = 0x01;
const LSS_STORE_FAILED: u8 = 0x02;

pub static STANDARD_BAUDRATE_TABLE: &[u16] = &[1000, 800, 500, 250, 125, 100, 50, 20, 10];

pub struct Lss<'a> {
    pub node_id: NodeId,
    lss_address: [u32; 4],
    mode: LssMode,
    partial_command_state: PartialCommandState,
    expected_lss_sub: u8, // used in fast_scan
    node_id_changed: bool,
    callback: Option<&'a mut dyn LssCallback>,
}

impl<'a> Lss<'a> {
    pub const LSS_REQUEST_ID: StandardId = unsafe { StandardId::new_unchecked(0x7E5) };
    pub const LSS_RESPONSE_ID: StandardId = unsafe { StandardId::new_unchecked(0x7E4) };

    /// According to CiA 301 `0` shall indicate an invalid vendor-ID.
    /// Other values must be registered with CiA.
    pub const INVALID_VENDOR_ID: u8 = 0;

    pub fn new(
        node_id: NodeId,
        vendor_id: u32,
        product_code: u32,
        revision_number: u32,
        serial_number: u32,
    ) -> Self {
        Lss {
            node_id,
            lss_address: [vendor_id, product_code, revision_number, serial_number],
            mode: LssMode::Wait,
            partial_command_state: PartialCommandState::Init,
            expected_lss_sub: 0,
            node_id_changed: false,
            callback: None,
        }
    }

    pub fn add_callback(&mut self, callback: &'a mut dyn LssCallback) {
        self.callback = Some(callback);
    }

    pub fn on_request<F: embedded_can::Frame>(&mut self, request: &[u8; 8]) -> Option<F> {
        let command_specifier = request[0];

        // check services that don't care about mode
        match command_specifier {
            SWITCH_GLOBAL => {
                self.partial_command_state = PartialCommandState::Init;
                // Switch state global service
                match request[1] {
                    0x00 => {
                        self.mode = LssMode::Wait;
                        if let Some(callback) = self.callback.as_mut() {
                            if self.node_id_changed {
                                callback.on_new_node_id(self.node_id);
                            }
                        }
                        self.node_id_changed = false;
                        // TODO maybe restart?
                        //  https://github.com/CANopenNode/CANopenNode/blob/master/305/CO_LSSslave.c#L67-L77
                    }
                    0x01 => {
                        self.mode = LssMode::Configuration;
                    }
                    _ => {}
                }
                return None;
            }

            SWITCH_SELECTIVE_VENDOR_ID
            | SWITCH_SELECTIVE_PRODUCT_CODE
            | SWITCH_SELECTIVE_REVISION_NUMBER
            | SWITCH_SELECTIVE_SERIAL_NUMBER => {
                // Switch state selective service
                return self
                    .switch_selective(request)
                    .map(|response| F::new(Lss::LSS_RESPONSE_ID, &response).unwrap());
            }
            IDENTIFY_VENDOR_ID
            | IDENTIFY_PRODUCT_CODE
            | IDENTIFY_REVISION_NUMBER_LOW
            | IDENTIFY_REVISION_NUMBER_HIGH
            | IDENTIFY_SERIAL_NUMBER_LOW
            | IDENTIFY_SERIAL_NUMBER_HIGH => {
                // LSS identify remote slave service
                return self
                    .identify(request)
                    .map(|response| F::new(Lss::LSS_RESPONSE_ID, &response).unwrap());
            }
            FAST_SCAN => {
                return self
                    .fast_scan(request)
                    .map(|response| F::new(Lss::LSS_RESPONSE_ID, &response).unwrap());
            }
            _ => {
                self.partial_command_state = PartialCommandState::Init;
            }
        }

        // Other services require configuration mode
        if self.mode == LssMode::Wait {
            return None;
        }

        let result = match command_specifier {
            CONFIGURE_NODE_ID => {
                // Configure node-ID service
                self.set_node_id(request[1])
            }
            CONFIGURE_BIT_TIMING => {
                // Configure bit timing parameters service
                self.set_bit_timing(request)
            }
            ACTIVATE_BIT_TIMING => {
                // Activate bit timing parameters service
                todo!()
            }
            STORE_CONFIGURATION => {
                // Store configuration service
                self.store_configuration()
            }
            INQUIRE_VENDOR_ID
            | INQUIRE_PRODUCT_CODE
            | INQUIRE_REVISION_NUMBER
            | INQUIRE_SERIAL_NUMBER => {
                // Inquire LSS address service
                self.inquire(command_specifier)
            }
            INQUIRE_NODE_ID => {
                // Inquire node id service
                Some([INQUIRE_NODE_ID, self.node_id.raw(), 0, 0, 0, 0, 0, 0])
            }
            _ => None,
        };

        result.map(|response| F::new(Lss::LSS_RESPONSE_ID, &response).unwrap())
    }

    fn set_node_id(&mut self, node_id: u8) -> RequestResult {
        if let Some(node_id) = NodeId::new(node_id) {
            self.node_id_changed = self.node_id != node_id;
            self.node_id = node_id;
            Some([CONFIGURE_NODE_ID, LSS_OK, 0, 0, 0, 0, 0, 0])
        } else {
            Some([CONFIGURE_NODE_ID, LSS_GENERIC_ERROR, 0, 0, 0, 0, 0, 0])
        }
    }

    fn store_configuration(&mut self) -> RequestResult {
        let status = if let Some(callback) = self.callback.as_mut() {
            match callback.store_configuration(self.node_id) {
                Ok(()) => LSS_OK,
                Err(StoreConfigurationError::NotSupported) => LSS_GENERIC_ERROR,
                Err(StoreConfigurationError::Failed) => LSS_STORE_FAILED,
            }
        } else {
            LSS_GENERIC_ERROR
        };

        Some([STORE_CONFIGURATION, status, 0, 0, 0, 0, 0, 0])
    }

    fn set_bit_timing(&mut self, request: &[u8; 8]) -> RequestResult {
        // TODO:
        //  After execution of the Configure Bit
        //  Timing Parameters service the node may not execute any remote LSS services except the services Configure Bit
        //  Timing Parameters, Activate Bit Timing Parameters and Switch Mode.
        let _command_specifier = request[0];
        let _table_selector = request[1];
        let _table_index = request[2];

        todo!() // Some([command_specifier, LSS_OK, 0, 0, 0, 0, 0, 0])
    }

    fn inquire(&self, command_specifier: u8) -> RequestResult {
        let address_data =
            self.lss_address[command_specifier as usize - INQUIRE_VENDOR_ID as usize];
        let mut response = [command_specifier, 0, 0, 0, 0, 0, 0, 0];
        response[1..5].copy_from_slice(&address_data.to_le_bytes());
        Some(response)
    }

    fn switch_selective(&mut self, request: &[u8; 8]) -> RequestResult {
        let command_specifier = request[0];
        let address_data = request[1..5].try_into().unwrap(); // Infallible
        let address_data = u32::from_le_bytes(address_data);

        match (command_specifier, self.partial_command_state) {
            (SWITCH_SELECTIVE_VENDOR_ID, _) => {
                if address_data == self.lss_address[0] {
                    self.partial_command_state = PartialCommandState::SwitchVendorIdMatched;
                    return None;
                }
            }
            (SWITCH_SELECTIVE_PRODUCT_CODE, PartialCommandState::SwitchVendorIdMatched) => {
                if address_data == self.lss_address[1] {
                    self.partial_command_state = PartialCommandState::SwitchProductCodeMatched;
                    return None;
                }
            }
            (SWITCH_SELECTIVE_REVISION_NUMBER, PartialCommandState::SwitchProductCodeMatched) => {
                if address_data == self.lss_address[2] {
                    self.partial_command_state =
                        PartialCommandState::SwitchRevisionNumberCodeMatched;
                    return None;
                }
            }
            (
                SWITCH_SELECTIVE_SERIAL_NUMBER,
                PartialCommandState::SwitchRevisionNumberCodeMatched,
            ) => {
                if address_data == self.lss_address[3] {
                    self.partial_command_state = PartialCommandState::Init;
                    self.mode = LssMode::Configuration;
                    return Some([SWITCH_SELECTIVE_SERIAL_RESPONSE, 0, 0, 0, 0, 0, 0, 0]);
                }
            }
            _ => {}
        }

        self.partial_command_state = PartialCommandState::Init;
        None
    }

    fn identify(&mut self, request: &[u8; 8]) -> RequestResult {
        let command_specifier = request[0];
        let address_data = request[1..5].try_into().unwrap(); // Infallible
        let address_data = u32::from_le_bytes(address_data);

        match (command_specifier, self.partial_command_state) {
            (IDENTIFY_VENDOR_ID, _) => {
                if address_data == self.lss_address[0] {
                    self.partial_command_state = PartialCommandState::IdentifyVendorIdMatched;
                    return None;
                }
            }
            (IDENTIFY_PRODUCT_CODE, PartialCommandState::IdentifyVendorIdMatched) => {
                if address_data == self.lss_address[1] {
                    self.partial_command_state = PartialCommandState::IdentifyProductCodeMatched;
                    return None;
                }
            }
            (IDENTIFY_REVISION_NUMBER_LOW, PartialCommandState::IdentifyProductCodeMatched) => {
                if address_data >= self.lss_address[2] {
                    self.partial_command_state =
                        PartialCommandState::IdentifyRevisionNumberLowCodeMatched;
                    return None;
                }
            }
            (
                IDENTIFY_REVISION_NUMBER_HIGH,
                PartialCommandState::IdentifyRevisionNumberLowCodeMatched,
            ) => {
                if address_data <= self.lss_address[2] {
                    self.partial_command_state =
                        PartialCommandState::IdentifyRevisionNumberHighCodeMatched;
                    return None;
                }
            }
            (
                IDENTIFY_SERIAL_NUMBER_LOW,
                PartialCommandState::IdentifyRevisionNumberHighCodeMatched,
            ) => {
                if address_data >= self.lss_address[3] {
                    self.partial_command_state =
                        PartialCommandState::IdentifySerialNumberLowCodeMatched;
                    return None;
                }
            }
            (
                IDENTIFY_SERIAL_NUMBER_HIGH,
                PartialCommandState::IdentifySerialNumberLowCodeMatched,
            ) => {
                if address_data >= self.lss_address[3] {
                    self.partial_command_state = PartialCommandState::Init;
                    return Some([IDENTIFY_RESPONSE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
                }
            }
            _ => {}
        }
        self.partial_command_state = PartialCommandState::Init;
        None
    }

    fn fast_scan(&mut self, request: &[u8; 8]) -> RequestResult {
        if self.mode == LssMode::Configuration {
            return None;
        }

        let id_number = request[1..5].try_into().unwrap(); // Infallible
        let id_number = u32::from_le_bytes(id_number);
        let bit_checked = request[5]; // Number of unchecked bits
        let lss_sub = request[6]; // index into lss_address
        let lss_next = request[7];

        if bit_checked == 128 {
            // TODO check if we want to participate
            self.expected_lss_sub = 0;
            return Some([IDENTIFY_RESPONSE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        } else if lss_sub == self.expected_lss_sub && lss_sub < 4 && bit_checked < 32 {
            let bit_mask = u32::MAX << bit_checked;

            if (self.lss_address[lss_sub as usize] ^ id_number) & bit_mask == 0 {
                // Checked bits match
                self.expected_lss_sub = lss_next; // only update lss_next if we're still matching
                if lss_sub == 3 && bit_checked == 0 {
                    // Complete match, scan completed
                    self.mode = LssMode::Configuration;
                }
                return Some([IDENTIFY_RESPONSE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
            }
        }
        None
    }
}

pub trait LssCallback {
    fn store_configuration(&mut self, node_id: NodeId) -> Result<(), StoreConfigurationError>;
    fn on_new_node_id(&mut self, node_id: NodeId);
}

pub enum StoreConfigurationError {
    NotSupported,
    Failed,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum LssMode {
    Configuration,
    Wait,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum PartialCommandState {
    Init,
    IdentifyVendorIdMatched,
    IdentifyProductCodeMatched,
    IdentifyRevisionNumberLowCodeMatched,
    IdentifyRevisionNumberHighCodeMatched,
    IdentifySerialNumberLowCodeMatched,
    SwitchVendorIdMatched,
    SwitchProductCodeMatched,
    SwitchRevisionNumberCodeMatched,
}

impl<F: embedded_can::Frame> CanOpenService<F> for Lss<'_> {
    fn on_message(&mut self, frame: &F) -> Option<F> {
        if frame.id() != Id::Standard(Lss::LSS_REQUEST_ID) {
            return None;
        }
        if let Ok(data) = frame.data().try_into() {
            self.on_request(data)
        } else {
            None
        }
    }
}