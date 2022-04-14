use core::cmp::Ordering;
use core::ops::Deref;
use core::sync::atomic::Ordering::Relaxed;

use crate::objectdictionary::ODError;

pub trait DataLink {
    fn read(&self, index: u16, subindex: u8) -> Result<ReadData<'_>, ODError>;
    fn write(&mut self, write_stream: WriteStream<'_>) -> Result<(), ODError>;
}

pub trait AtomicDataLink {
    fn read(&self, index: u16, subindex: u8) -> Result<ReadData<'_>, ODError>;
    fn write(&self, write_stream: WriteStream<'_>) -> Result<(), ODError>;
}

pub enum ReadData<'a> {
    B1([u8; 1]),
    B2([u8; 2]),
    B3([u8; 3]),
    B4([u8; 4]),
    B5([u8; 5]),
    B6([u8; 6]),
    B7([u8; 7]),
    Bytes(&'a [u8]),
}

impl ReadData<'_> {
    pub fn get(&self) -> &[u8] {
        match self {
            ReadData::B1(val) => val,
            ReadData::B2(val) => val,
            ReadData::B3(val) => val,
            ReadData::B4(val) => val,
            ReadData::B5(val) => val,
            ReadData::B6(val) => val,
            ReadData::B7(val) => val,
            ReadData::Bytes(val) => val,
        }
    }
}

pub struct WriteData<'a> {
    pub index: u16,
    pub subindex: u8,
    pub new_data: &'a [u8],
    pub offset: usize,
    pub promised_size: Option<usize>,
    pub is_last_segment: bool,
}

pub struct WriteStream<'a>(pub(crate) &'a WriteData<'a>);

impl<'a> Deref for WriteStream<'a> {
    type Target = WriteData<'a>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl WriteData<'_> {
    #[inline(always)]
    pub fn is_first_segment(&self) -> bool {
        self.offset == 0
    }
}

impl WriteStream<'_> {
    pub fn write_into(self, buf: &mut [u8]) -> Result<WriteStatus, ODError> {
        if let Some(promised_size) = self.promised_size {
            check_size(promised_size, buf.len())?;
        }

        let bytes_written = self.offset + self.new_data.len();
        if bytes_written > buf.len() {
            return Err(ODError::TooLong);
        }

        buf[self.offset..bytes_written].copy_from_slice(self.new_data);

        if self.is_last_segment {
            Ok(WriteStatus::Done { bytes_written })
        } else {
            Ok(WriteStatus::InProgress { bytes_written })
        }
    }
}

pub enum WriteStatus {
    InProgress { bytes_written: usize },
    Done { bytes_written: usize },
}

impl<'a> From<&'a [u8]> for ReadData<'a> {
    #[inline]
    fn from(val: &'a [u8]) -> Self {
        ReadData::Bytes(val)
    }
}

impl<'a> From<&'a str> for ReadData<'a> {
    #[inline]
    fn from(val: &'a str) -> Self {
        ReadData::Bytes(val.as_bytes())
    }
}

macro_rules! from_impl {
    ($typ:ty, $variant:path) => {
        impl From<$typ> for ReadData<'_> {
            #[inline]
            fn from(val: $typ) -> Self {
                $variant(val.to_le_bytes())
            }
        }
    };
}

from_impl!(u8, ReadData::B1);
from_impl!(i8, ReadData::B1);
from_impl!(u16, ReadData::B2);
from_impl!(i16, ReadData::B2);
from_impl!(u32, ReadData::B4);
from_impl!(i32, ReadData::B4);

macro_rules! try_from_impl {
    ($typ:ty) => {
        impl<'a> TryFrom<WriteStream<'a>> for $typ {
            type Error = ODError;

            fn try_from(write_stream: WriteStream<'a>) -> Result<Self, Self::Error> {
                debug_assert!(write_stream.offset == 0);
                debug_assert!(write_stream.is_last_segment);

                check_size(write_stream.new_data.len(), core::mem::size_of::<$typ>())?;
                if let Some(size) = write_stream.promised_size {
                    check_size(size, core::mem::size_of::<$typ>())?;
                }
                let data = write_stream.new_data.try_into().ok().unwrap();
                Ok(<$typ>::from_le_bytes(data))
            }
        }
    };
}

try_from_impl!(u8);
try_from_impl!(i8);
try_from_impl!(u16);
try_from_impl!(i16);
try_from_impl!(u32);
try_from_impl!(i32);

macro_rules! try_from_impl_array {
    ($typ:ty) => {
        impl<'a> TryFrom<WriteStream<'a>> for $typ {
            type Error = ODError;

            fn try_from(write_stream: WriteStream<'a>) -> Result<Self, Self::Error> {
                debug_assert!(write_stream.offset == 0);
                debug_assert!(write_stream.is_last_segment);

                check_size(write_stream.new_data.len(), core::mem::size_of::<$typ>())?;
                if let Some(size) = write_stream.promised_size {
                    check_size(size, core::mem::size_of::<$typ>())?;
                }
                let data = write_stream.new_data.try_into().ok().unwrap();
                Ok(data)
            }
        }
    };
}

try_from_impl_array!([u8; 1]);
try_from_impl_array!([u8; 2]);
try_from_impl_array!([u8; 3]);
try_from_impl_array!([u8; 4]);
try_from_impl_array!([u8; 5]);
try_from_impl_array!([u8; 6]);
try_from_impl_array!([u8; 7]);

fn check_size(given: usize, expected: usize) -> Result<(), ODError> {
    match given.cmp(&expected) {
        Ordering::Less => Err(ODError::TooShort),
        Ordering::Greater => Err(ODError::TooLong),
        Ordering::Equal => Ok(()),
    }
}

macro_rules! atomic_data_link {
    ($typ:ty) => {
        impl AtomicDataLink for $typ {
            fn read(&self, _: u16, _: u8) -> Result<ReadData<'_>, ODError> {
                Ok(self.load(Relaxed).into())
            }

            fn write(&self, write_stream: WriteStream<'_>) -> Result<(), ODError> {
                self.store(write_stream.try_into()?, Relaxed);
                Ok(())
            }
        }
    };
}

atomic_data_link!(core::sync::atomic::AtomicU8);
atomic_data_link!(core::sync::atomic::AtomicI8);
atomic_data_link!(core::sync::atomic::AtomicU16);
atomic_data_link!(core::sync::atomic::AtomicI16);
atomic_data_link!(core::sync::atomic::AtomicU32);
atomic_data_link!(core::sync::atomic::AtomicI32);

macro_rules! atomic_data_link_cell {
    ($typ:ty) => {
        impl AtomicDataLink for $typ {
            fn read(&self, _: u16, _: u8) -> Result<ReadData<'_>, ODError> {
                Ok(self.get().into())
            }

            fn write(&self, write_stream: WriteStream<'_>) -> Result<(), ODError> {
                self.set(write_stream.try_into()?);
                Ok(())
            }
        }
    };
}

atomic_data_link_cell!(core::cell::Cell<u8>);
atomic_data_link_cell!(core::cell::Cell<i8>);
atomic_data_link_cell!(core::cell::Cell<u16>);
atomic_data_link_cell!(core::cell::Cell<i16>);
atomic_data_link_cell!(core::cell::Cell<u32>);
atomic_data_link_cell!(core::cell::Cell<i32>);
