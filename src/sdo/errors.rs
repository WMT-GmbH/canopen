use core::fmt;

pub struct SdoAbortedError(pub u32);

impl SdoAbortedError {
    pub fn to_le_bytes(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

const TOGGLE_BIT_NOT_ALTERNATED: u32 = 0x0503_0000;
const SDO_PROTOCOL_TIMED_OUT: u32 = 0x0504_0000;
const COMMAND_SPECIFIER_ERROR: u32 = 0x0504_0001;
const INVALID_BLOCK_SIZE: u32 = 0x0504_0002;
const INVALID_SEQUENCE_NUMBER: u32 = 0x0504_0003;
const CRC_ERROR: u32 = 0x0504_0004;
const OUT_OF_MEMORY: u32 = 0x0504_0005;
const UNSUPPORTED_ACCESS: u32 = 0x0601_0000;
const WRITE_ONLY_ERROR: u32 = 0x0601_0001;
const READ_ONLY_ERROR: u32 = 0x0601_0002;
const OBJECT_DOES_NOT_EXIST: u32 = 0x0602_0000;
const OBJECT_CANNOT_BE_MAPPED: u32 = 0x0604_0041;
const PDO_OVERFLOW: u32 = 0x0604_0042;
const PARAMETER_INCOMPATIBILITY: u32 = 0x0604_0043;
const INTERNAL_INCOMPATIBILITY: u32 = 0x0604_0047;
const HARDWARE_ERROR: u32 = 0x0606_0000;
const WRONG_LENGTH: u32 = 0x0607_0010;
const TOO_LONG: u32 = 0x0607_0012;
const TOO_SHORT: u32 = 0x0607_0013;
const SUBINDEX_DOES_NOT_EXIST: u32 = 0x0609_0011;
const INVALID_VALUE: u32 = 0x0609_0030;
const VALUE_TOO_HIGH: u32 = 0x0609_0031;
const VALUE_TOO_LOW: u32 = 0x0609_0032;
const MAX_LESS_THAN_MIN: u32 = 0x0609_0036;
const RESOURCE_NOT_AVAILABLE: u32 = 0x060A_0023;
const GENERAL_ERROR: u32 = 0x0800_0000;
const TRANSFER_OR_STORAGE_ERROR: u32 = 0x0800_0020;
const LOCAL_CONTROL_ERROR: u32 = 0x0800_0021;
const DEVICE_STATE_ERROR: u32 = 0x0800_0022;
const DICTIONARY_ERROR: u32 = 0x0800_0023;
const NO_DATA_AVAILABLE: u32 = 0x0800_0024;

impl fmt::Display for SdoAbortedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self.0 {
            TOGGLE_BIT_NOT_ALTERNATED => "Toggle bit not alternated",
            SDO_PROTOCOL_TIMED_OUT => "SDO protocol timed out",
            COMMAND_SPECIFIER_ERROR => "Client/server command specifier not valid or unknown",
            INVALID_BLOCK_SIZE => "Invalid block size",
            INVALID_SEQUENCE_NUMBER => "Invalid sequence number",
            CRC_ERROR => "CRC error",
            OUT_OF_MEMORY => "Out of memory",
            UNSUPPORTED_ACCESS => "Unsupported access to an object",
            WRITE_ONLY_ERROR => "Attempt to read a write only object",
            READ_ONLY_ERROR => "Attempt to write a read only object",
            OBJECT_DOES_NOT_EXIST => "Object does not exist in the object dictionary",
            OBJECT_CANNOT_BE_MAPPED => "Object cannot be mapped to the PDO",
            PDO_OVERFLOW => "The number and length of the objects to be mapped would exceed PDO length",
            PARAMETER_INCOMPATIBILITY => "General parameter incompatibility reason",
            INTERNAL_INCOMPATIBILITY => "General internal incompatibility in the device",
            HARDWARE_ERROR => "Access failed due to a hardware error",
            WRONG_LENGTH => "Data type does not match, length of service parameter does not match",
            TOO_LONG => "Data type does not match, length of service parameter too high",
            TOO_SHORT => "Data type does not match, length of service parameter too low",
            SUBINDEX_DOES_NOT_EXIST => "Subindex does not exist",
            INVALID_VALUE => "Invalid value for parameter",
            VALUE_TOO_HIGH => "Value of parameter written too high",
            VALUE_TOO_LOW => "Value of parameter written too low",
            MAX_LESS_THAN_MIN => "Maximum value is less than minimum value",
            RESOURCE_NOT_AVAILABLE => "Resource not available: SDO connection",
            GENERAL_ERROR => "General error",
            TRANSFER_OR_STORAGE_ERROR => "Data cannot be transferred or stored to the application",
            LOCAL_CONTROL_ERROR => "Data can not be transferred or stored to the application because of local control",
            DEVICE_STATE_ERROR => "Data can not be transferred or stored to the application because of the present device state",
            DICTIONARY_ERROR => "Object dictionary dynamic generation fails or no object dictionary is present",
            NO_DATA_AVAILABLE => "No data available",
            _ => "",
        };

        write!(f, "Code 0x{:08X}, {}", self.0, text)
    }
}
