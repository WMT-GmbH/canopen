use core::fmt;

pub enum SDOAbortCode {
    ToggleBitNotAlternated,
    SDOProtocolTimedOut,
    CommandSpecifierError,
    InvalidBlockSize,
    InvalidSequenceNumber,
    CRCError,
    OutOfMemory,
    UnsupportedAccess,
    WriteOnlyError,
    ReadOnlyError,
    ObjectDoesNotExist,
    ObjectCannotBeMapped,
    PDOOverflow,
    ParameterIncompatibility,
    InternalIncompatibility,
    HardwareError,
    WrongLength,
    TooLong,
    TooShort,
    SubindexDoesNotExist,
    InvalidValue,
    ValueTooHigh,
    ValueTooLow,
    MaxLessThanMin,
    ResourceNotAvailable,
    GeneralError,
    TransferOrStorageError,
    LocalControlError,
    DeviceStateError,
    DictionaryError,
    NoDataAvailable,
    UnknownAbortCode,
}

impl From<u32> for SDOAbortCode {
    fn from(abort_code: u32) -> Self {
        match abort_code {
            0x0503_0000 => SDOAbortCode::ToggleBitNotAlternated,
            0x0504_0000 => SDOAbortCode::SDOProtocolTimedOut,
            0x0504_0001 => SDOAbortCode::CommandSpecifierError,
            0x0504_0002 => SDOAbortCode::InvalidBlockSize,
            0x0504_0003 => SDOAbortCode::InvalidSequenceNumber,
            0x0504_0004 => SDOAbortCode::CRCError,
            0x0504_0005 => SDOAbortCode::OutOfMemory,
            0x0601_0000 => SDOAbortCode::UnsupportedAccess,
            0x0601_0001 => SDOAbortCode::WriteOnlyError,
            0x0601_0002 => SDOAbortCode::ReadOnlyError,
            0x0602_0000 => SDOAbortCode::ObjectDoesNotExist,
            0x0604_0041 => SDOAbortCode::ObjectCannotBeMapped,
            0x0604_0042 => SDOAbortCode::PDOOverflow,
            0x0604_0043 => SDOAbortCode::ParameterIncompatibility,
            0x0604_0047 => SDOAbortCode::InternalIncompatibility,
            0x0606_0000 => SDOAbortCode::HardwareError,
            0x0607_0010 => SDOAbortCode::WrongLength,
            0x0607_0012 => SDOAbortCode::TooLong,
            0x0607_0013 => SDOAbortCode::TooShort,
            0x0609_0011 => SDOAbortCode::SubindexDoesNotExist,
            0x0609_0030 => SDOAbortCode::InvalidValue,
            0x0609_0031 => SDOAbortCode::ValueTooHigh,
            0x0609_0032 => SDOAbortCode::ValueTooLow,
            0x0609_0036 => SDOAbortCode::MaxLessThanMin,
            0x0800_0000 => SDOAbortCode::GeneralError,
            0x0800_0020 => SDOAbortCode::TransferOrStorageError,
            0x0800_0021 => SDOAbortCode::LocalControlError,
            0x0800_0022 => SDOAbortCode::DeviceStateError,
            0x0800_0023 => SDOAbortCode::DictionaryError,
            0x0800_0024 => SDOAbortCode::NoDataAvailable,
            _ => SDOAbortCode::UnknownAbortCode,
        }
    }
}

impl Into<u32> for SDOAbortCode {
    fn into(self) -> u32 {
        match self {
            SDOAbortCode::ToggleBitNotAlternated => 0x0503_0000u32,
            SDOAbortCode::SDOProtocolTimedOut => 0x0504_0000u32,
            SDOAbortCode::CommandSpecifierError => 0x0504_0001u32,
            SDOAbortCode::InvalidBlockSize => 0x0504_0002u32,
            SDOAbortCode::InvalidSequenceNumber => 0x0504_0003u32,
            SDOAbortCode::CRCError => 0x0504_0004u32,
            SDOAbortCode::OutOfMemory => 0x0504_0005u32,
            SDOAbortCode::UnsupportedAccess => 0x0601_0000u32,
            SDOAbortCode::WriteOnlyError => 0x0601_0001u32,
            SDOAbortCode::ReadOnlyError => 0x0601_0002u32,
            SDOAbortCode::ObjectDoesNotExist => 0x0602_0000u32,
            SDOAbortCode::ObjectCannotBeMapped => 0x0604_0041u32,
            SDOAbortCode::PDOOverflow => 0x0604_0042u32,
            SDOAbortCode::ParameterIncompatibility => 0x0604_0043u32,
            SDOAbortCode::InternalIncompatibility => 0x0604_0047u32,
            SDOAbortCode::HardwareError => 0x0606_0000u32,
            SDOAbortCode::WrongLength => 0x0607_0010u32,
            SDOAbortCode::TooLong => 0x0607_0012u32,
            SDOAbortCode::TooShort => 0x0607_0013u32,
            SDOAbortCode::SubindexDoesNotExist => 0x0609_0011u32,
            SDOAbortCode::InvalidValue => 0x0609_0030u32,
            SDOAbortCode::ValueTooHigh => 0x0609_0031u32,
            SDOAbortCode::ValueTooLow => 0x0609_0032u32,
            SDOAbortCode::MaxLessThanMin => 0x0609_0036u32,
            SDOAbortCode::ResourceNotAvailable => 0x060A_0023u32,
            SDOAbortCode::GeneralError => 0x0800_0000u32,
            SDOAbortCode::TransferOrStorageError => 0x0800_0020u32,
            SDOAbortCode::LocalControlError => 0x0800_0021u32,
            SDOAbortCode::DeviceStateError => 0x0800_0022u32,
            SDOAbortCode::DictionaryError => 0x0800_0023u32,
            SDOAbortCode::NoDataAvailable => 0x0800_0024u32,
            SDOAbortCode::UnknownAbortCode => 0,
        }
    }
}

impl Into<[u8; 4]> for SDOAbortCode {
    fn into(self) -> [u8; 4] {
        let code: u32 = self.into();
        code.to_le_bytes()
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

impl fmt::Display for SDOAbortCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            SDOAbortCode::ToggleBitNotAlternated => "Toggle bit not alternated",
            SDOAbortCode::SDOProtocolTimedOut => "SDO protocol timed out",
            SDOAbortCode::CommandSpecifierError => "Client/server command specifier not valid or unknown",
            SDOAbortCode::InvalidBlockSize => "Invalid block size",
            SDOAbortCode::InvalidSequenceNumber => "Invalid sequence number",
            SDOAbortCode::CRCError => "CRC error",
            SDOAbortCode::OutOfMemory => "Out of memory",
            SDOAbortCode::UnsupportedAccess => "Unsupported access to an object",
            SDOAbortCode::WriteOnlyError => "Attempt to read a write only object",
            SDOAbortCode::ReadOnlyError => "Attempt to write a read only object",
            SDOAbortCode::ObjectDoesNotExist => "Object does not exist in the object dictionary",
            SDOAbortCode::ObjectCannotBeMapped => "Object cannot be mapped to the PDO",
            SDOAbortCode::PDOOverflow => "The number and length of the objects to be mapped would exceed PDO length",
            SDOAbortCode::ParameterIncompatibility => "General parameter incompatibility reason",
            SDOAbortCode::InternalIncompatibility => "General internal incompatibility in the device",
            SDOAbortCode::HardwareError => "Access failed due to a hardware error",
            SDOAbortCode::WrongLength => "Data type does not match, length of service parameter does not match",
            SDOAbortCode::TooLong => "Data type does not match, length of service parameter too high",
            SDOAbortCode::TooShort => "Data type does not match, length of service parameter too low",
            SDOAbortCode::SubindexDoesNotExist => "Subindex does not exist",
            SDOAbortCode::InvalidValue => "Invalid value for parameter",
            SDOAbortCode::ValueTooHigh => "Value of parameter written too high",
            SDOAbortCode::ValueTooLow => "Value of parameter written too low",
            SDOAbortCode::MaxLessThanMin => "Maximum value is less than minimum value",
            SDOAbortCode::ResourceNotAvailable => "Resource not available: SDO connection",
            SDOAbortCode::GeneralError => "General error",
            SDOAbortCode::TransferOrStorageError => "Data cannot be transferred or stored to the application",
            SDOAbortCode::LocalControlError => "Data can not be transferred or stored to the application because of local control",
            SDOAbortCode::DeviceStateError => "Data can not be transferred or stored to the application because of the present device state",
            SDOAbortCode::DictionaryError => "Object dictionary dynamic generation fails or no object dictionary is present",
            SDOAbortCode::NoDataAvailable => "No data available",
            SDOAbortCode::UnknownAbortCode => "UnknownAbortCode",
        };

        write!(f, "Code 0x{:08X}, {}", self.into(), text)
    }
}
