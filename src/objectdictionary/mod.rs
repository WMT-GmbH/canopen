use crate::meta::{from_raw_parts_mut, DynMetadata};
use crate::objectdictionary::datalink::{BasicData, DataLink, ReadData, WriteStream};
use core::num::NonZeroU8;
pub use variable::{CANOpenData, Variable};

use crate::sdo::SDOAbortCode;

pub mod datalink;
pub mod odcell;
pub mod variable;

pub use canopen_derive::OdData;

pub trait OdData {
    type OdType;

    fn into_od(self) -> Self::OdType;
}

pub struct ObjectDictionary<T, const N: usize> {
    indices: [u16; N],
    subindices: [u8; N],
    pdo_sizes: [Option<NonZeroU8>; N],
    offsets: [usize; N],
    vtables: [DynMetadata<dyn DataLink>; N],
    pub data: T,
}

impl<T, const N: usize> ObjectDictionary<T, N> {
    #[doc(hidden)]
    pub unsafe fn new(
        indices: [u16; N],
        subindices: [u8; N],
        pdo_sizes: [Option<NonZeroU8>; N],
        offsets: [usize; N],
        vtables: [DynMetadata<dyn DataLink>; N],
        data: T,
    ) -> Self {
        ObjectDictionary {
            indices,
            subindices,
            pdo_sizes,
            offsets,
            vtables,
            data,
        }
    }

    pub fn find(&mut self, index: u16, subindex: u8) -> Result<&mut dyn DataLink, ODError> {
        let position = self.search(index, subindex)?;
        Ok(self.get(position))
    }

    pub fn read(&mut self, index: u16, subindex: u8) -> Result<ReadData<'_>, ODError> {
        self.find(index, subindex)?.read(index, subindex)
    }

    pub(crate) fn get(&mut self, position: usize) -> &mut dyn DataLink {
        let mut data_ptr = &mut self.data as *mut T as *mut ();
        unsafe {
            data_ptr = data_ptr.byte_add(self.offsets[position]);
        }
        let metadata = self.vtables[position];
        let fat_ptr = from_raw_parts_mut(data_ptr, metadata);
        unsafe { &mut *fat_ptr }
    }

    pub(crate) fn get_plus(&mut self, position: usize) -> (&mut dyn DataLink, OdInfo) {
        let mut data_ptr = &mut self.data as *mut T as *mut ();
        unsafe {
            data_ptr = data_ptr.byte_add(self.offsets[position]);
        }
        let metadata = self.vtables[position];
        let fat_ptr = from_raw_parts_mut(data_ptr, metadata);
        (
            unsafe { &mut *fat_ptr },
            OdInfo {
                indices: &self.indices,
                subindices: &self.subindices,
                pdo_sizes: &self.pdo_sizes,
            },
        )
    }

    pub(crate) fn search(&self, index: u16, subindex: u8) -> Result<usize, ODError> {
        od_search(&self.indices, &self.subindices, index, subindex)
    }
}

pub struct OdInfo<'a> {
    indices: &'a [u16],
    subindices: &'a [u8],
    pdo_sizes: &'a [Option<NonZeroU8>],
}

pub(crate) fn od_search(
    indices: &[u16],
    subindices: &[u8],
    index: u16,
    subindex: u8,
) -> Result<usize, ODError> {
    let mut partion_point = indices.partition_point(|&i| i < index);
    if indices.get(partion_point).copied() != Some(index) {
        return Err(ODError::ObjectDoesNotExist);
    }
    loop {
        if indices.get(partion_point).copied() != Some(index) {
            break Err(ODError::SubindexDoesNotExist);
        }
        if subindices[partion_point] == subindex {
            break Ok(partion_point);
        }
        partion_point += 1;
    }
}

pub struct OdArray<T, const N: usize> {
    pub array: [T; N],
}

impl<T, const N: usize> OdArray<T, N> {
    pub fn new(array: [T; N]) -> Self {
        OdArray { array }
    }
}

impl<T: BasicData, const N: usize> DataLink for OdArray<T, N> {
    fn read(&self, _: u16, subindex: u8) -> Result<ReadData<'_>, ODError> {
        if subindex == 0 {
            const { assert!(N <= u8::MAX as usize) };
            Ok((N as u8).into())
        } else if let Some(variable) = self.array.get(subindex as usize - 1) {
            Ok(variable.read())
        } else {
            Err(ODError::SubindexDoesNotExist)
        }
    }

    fn write(&mut self, write_stream: WriteStream<'_>) -> Result<(), ODError> {
        if write_stream.subindex == 0 {
            Err(ODError::ReadOnlyError)
        } else if let Some(variable) = self.array.get_mut(write_stream.subindex as usize - 1) {
            variable.write(write_stream)
        } else {
            Err(ODError::SubindexDoesNotExist)
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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
