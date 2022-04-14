pub use variable::{CANOpenData, Variable};

use crate::sdo::SDOAbortCode;

pub mod datalink;
pub mod odcell;
pub mod variable;

pub type ObjectDictionary<'a> = &'a [Variable<'a>];

pub trait ObjectDictionaryExt<'a> {
    fn find(&self, index: u16, subindex: u8) -> Result<&'a Variable<'a>, ODError>;
}

impl<'a> ObjectDictionaryExt<'a> for ObjectDictionary<'a> {
    fn find(&self, index: u16, subindex: u8) -> Result<&'a Variable<'a>, ODError> {
        match self.binary_search_by(|obj| (obj.index, obj.subindex).cmp(&(index, subindex))) {
            Ok(position) => Ok(&self[position]),
            Err(position) => {
                // If there is an object with the same index but different subindex
                // we need to return ODError::SubindexDoesNotExist.

                // Binary search will return the position at which one could insert
                // the searched for variable.
                // If an object with the same index exists, the returned position will point into
                // or just past such an object.

                // So if the variables at position and position - 1 do not match,
                // such an object cannot exist.
                if position < self.len() && self[position].index == index {
                    return Err(ODError::SubindexDoesNotExist);
                }
                if position != 0 && self[position - 1].index == index {
                    return Err(ODError::SubindexDoesNotExist);
                }
                Err(ODError::ObjectDoesNotExist)
            }
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum ODError {
    ObjectDoesNotExist = SDOAbortCode::ObjectDoesNotExist as u32,
    OutOfMemory = SDOAbortCode::OutOfMemory as u32,
    UnsupportedAccess = SDOAbortCode::UnsupportedAccess as u32,
    WriteOnlyError = SDOAbortCode::WriteOnlyError as u32,
    ReadOnlyError = SDOAbortCode::ReadOnlyError as u32,
    ObjectCannotBeMapped = SDOAbortCode::ObjectCannotBeMapped as u32,
    PDOOverflow = SDOAbortCode::PDOOverflow as u32,
    ParameterIncompatibility = SDOAbortCode::ParameterIncompatibility as u32,
    InternalIncompatibility = SDOAbortCode::InternalIncompatibility as u32,
    HardwareError = SDOAbortCode::HardwareError as u32,
    WrongLength = SDOAbortCode::WrongLength as u32,
    TooLong = SDOAbortCode::TooLong as u32,
    TooShort = SDOAbortCode::TooShort as u32,
    SubindexDoesNotExist = SDOAbortCode::SubindexDoesNotExist as u32,
    InvalidValue = SDOAbortCode::InvalidValue as u32,
    ValueTooHigh = SDOAbortCode::ValueTooHigh as u32,
    ValueTooLow = SDOAbortCode::ValueTooLow as u32,
    MaxLessThanMin = SDOAbortCode::MaxLessThanMin as u32,
    ResourceNotAvailable = SDOAbortCode::ResourceNotAvailable as u32,
    GeneralError = SDOAbortCode::GeneralError as u32,
    TransferOrStorageError = SDOAbortCode::TransferOrStorageError as u32,
    LocalControlError = SDOAbortCode::LocalControlError as u32,
    DeviceStateError = SDOAbortCode::DeviceStateError as u32,
    DictionaryError = SDOAbortCode::DictionaryError as u32,
    NoDataAvailable = SDOAbortCode::NoDataAvailable as u32,
}
