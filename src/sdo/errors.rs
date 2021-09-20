use core::fmt;

use crate::objectdictionary::ODError;

#[repr(u32)]
#[derive(Debug)]
pub enum SDOAbortCode {
    UnknownAbortCode,
    ToggleBitNotAlternated = 0x0503_0000,
    SDOProtocolTimedOut = 0x0504_0000,
    CommandSpecifierError = 0x0504_0001,
    InvalidBlockSize = 0x0504_0002,
    InvalidSequenceNumber = 0x0504_0003,
    CRCError = 0x0504_0004,
    OutOfMemory = 0x0504_0005,
    UnsupportedAccess = 0x0601_0000,
    WriteOnlyError = 0x0601_0001,
    ReadOnlyError = 0x0601_0002,
    ObjectDoesNotExist = 0x0602_0000,
    ObjectCannotBeMapped = 0x0604_0041,
    PDOOverflow = 0x0604_0042,
    ParameterIncompatibility = 0x0604_0043,
    InternalIncompatibility = 0x0604_0047,
    HardwareError = 0x0606_0000,
    WrongLength = 0x0607_0010,
    TooLong = 0x0607_0012,
    TooShort = 0x0607_0013,
    SubindexDoesNotExist = 0x0609_0011,
    InvalidValue = 0x0609_0030,
    ValueTooHigh = 0x0609_0031,
    ValueTooLow = 0x0609_0032,
    MaxLessThanMin = 0x0609_0036,
    ResourceNotAvailable = 0x060A_0023,
    GeneralError = 0x0800_0000,
    TransferOrStorageError = 0x0800_0020,
    LocalControlError = 0x0800_0021,
    DeviceStateError = 0x0800_0022,
    DictionaryError = 0x0800_0023,
    NoDataAvailable = 0x0800_0024,
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

impl fmt::Display for SDOAbortCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            SDOAbortCode::UnknownAbortCode => "Unknown abort code",
        };

        //write!(f, "Code 0x{:08X}, {}", self as u32, text)
        write!(f, "{}", text)
    }
}

impl From<ODError> for SDOAbortCode {
    fn from(e: ODError) -> Self {
        match e {
            ODError::IndexDoesNotExist => SDOAbortCode::ObjectDoesNotExist,
            ODError::SubindexDoesNotExist => SDOAbortCode::SubindexDoesNotExist,
        }
    }
}
