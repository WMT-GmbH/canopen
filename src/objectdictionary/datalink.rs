use core::cmp::Ordering;
use core::ops::Deref;

use crate::objectdictionary::od_cell::OdCell;
use crate::objectdictionary::{ODError, OdInfo};

mod private {
    use super::{BasicData, OdCell};

    pub trait Sealed {}
    impl<T: BasicData> Sealed for T {}
    impl Sealed for &str {}
    impl Sealed for &[u8] {}
    impl<T> Sealed for OdCell<T> {}
}
/* TODO wait for stabilization
#[diagnostic::on_unimplemented(
    note = "`{Self}` must either implement `BasicData` or be wrapped in `OdCell`"
)]*/
pub trait DataLink: private::Sealed {
    fn read(&mut self, index: u16, subindex: u8) -> Result<ReadData, ODError>;
    fn write(&mut self, data: &WriteData, od_info: OdInfo) -> Result<(), ODError>;
}

pub trait BasicData {
    fn read(&mut self, index: u16, subindex: u8) -> Result<BasicReadData, ODError>;
    fn write(&mut self, data: BasicWriteData, od_info: OdInfo) -> Result<(), ODError>;
}

pub trait CustomData {
    fn read(&self, index: u16, subindex: u8) -> Result<ReadData, ODError>;
    fn write(&mut self, data: WriteStream, od_info: OdInfo) -> Result<(), ODError>;
}

impl<T: BasicData> DataLink for T {
    fn read(&mut self, index: u16, subindex: u8) -> Result<ReadData, ODError> {
        Ok(BasicData::read(self, index, subindex)?.into())
    }

    fn write(&mut self, data: &WriteData, od_info: OdInfo) -> Result<(), ODError> {
        self.write(BasicWriteData(data), od_info)
    }
}

macro_rules! basic_data {
    ($typ:ty) => {
        impl BasicData for $typ {
            fn read(&mut self, _: u16, _: u8) -> Result<BasicReadData, ODError> {
                Ok(BasicReadData::from(*self))
            }

            fn write(&mut self, data: BasicWriteData, _: OdInfo) -> Result<(), ODError> {
                *self = data.try_into()?;
                Ok(())
            }
        }
    };
}

basic_data!(bool);
basic_data!(i8);
basic_data!(i16);
basic_data!(i32);
basic_data!(u8);
basic_data!(u16);
basic_data!(u32);
basic_data!(f32);

impl DataLink for &str {
    fn read(&mut self, _: u16, _: u8) -> Result<ReadData, ODError> {
        Ok(ReadData::Bytes(self.as_bytes()))
    }

    fn write(&mut self, _: &WriteData, _: OdInfo) -> Result<(), ODError> {
        Err(ODError::ReadOnlyError)
    }
}

impl DataLink for &[u8] {
    fn read(&mut self, _: u16, _: u8) -> Result<ReadData, ODError> {
        Ok(ReadData::Bytes(self))
    }

    fn write(&mut self, _: &WriteData, _: OdInfo) -> Result<(), ODError> {
        Err(ODError::ReadOnlyError)
    }
}

impl<T: CustomData> DataLink for OdCell<T> {
    fn read(&mut self, index: u16, subindex: u8) -> Result<ReadData, ODError> {
        // TODO locking
        CustomData::read(self.get(), index, subindex)
    }

    fn write(&mut self, data: &WriteData, od_info: OdInfo) -> Result<(), ODError> {
        if data.is_first_segment() {
            self.lock();
        } else if !self.is_locked() {
            return Err(ODError::LocalControlError);
        }
        if data.is_last_segment {
            self.unlock();
        }
        self.get_mut_unchecked().write(WriteStream(data), od_info)
    }
}

impl<const N: usize> CustomData for [u8; N] {
    fn read(&self, _: u16, _: u8) -> Result<ReadData, ODError> {
        Ok(self[..].into())
    }

    fn write(&mut self, data: WriteStream, _: OdInfo) -> Result<(), ODError> {
        data.write_into(self)?;
        Ok(())
    }
}

pub enum ReadData<'a> {
    B1([u8; 1]),
    B2([u8; 2]),
    B4([u8; 4]),
    Bytes(&'a [u8]),
}

pub enum BasicReadData {
    B1([u8; 1]),
    B2([u8; 2]),
    B4([u8; 4]),
}

impl From<BasicReadData> for ReadData<'_> {
    fn from(value: BasicReadData) -> Self {
        match value {
            BasicReadData::B1(val) => ReadData::B1(val),
            BasicReadData::B2(val) => ReadData::B2(val),
            BasicReadData::B4(val) => ReadData::B4(val),
        }
    }
}

impl ReadData<'_> {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            ReadData::B1(val) => val,
            ReadData::B2(val) => val,
            ReadData::B4(val) => val,
            ReadData::Bytes(val) => val,
        }
    }
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

macro_rules! read_data_from_impl {
    ($typ:ty, $variant:path, $basic_variant:path) => {
        impl From<$typ> for ReadData<'_> {
            #[inline]
            fn from(val: $typ) -> Self {
                $variant(val.to_le_bytes())
            }
        }
        impl From<$typ> for BasicReadData {
            #[inline]
            fn from(val: $typ) -> Self {
                $basic_variant(val.to_le_bytes())
            }
        }
    };
}

read_data_from_impl!(u8, ReadData::B1, BasicReadData::B1);
read_data_from_impl!(i8, ReadData::B1, BasicReadData::B1);
read_data_from_impl!(u16, ReadData::B2, BasicReadData::B2);
read_data_from_impl!(i16, ReadData::B2, BasicReadData::B2);
read_data_from_impl!(u32, ReadData::B4, BasicReadData::B4);
read_data_from_impl!(i32, ReadData::B4, BasicReadData::B4);
read_data_from_impl!(f32, ReadData::B4, BasicReadData::B4);

impl From<bool> for BasicReadData {
    #[inline]
    fn from(val: bool) -> Self {
        BasicReadData::from(val as u8)
    }
}

/// A chunk of data to be written using SDO
///
/// ## Expedited transfer
/// * `new_data` is 1 to 4 bytes long TODO 0 possible?
/// * `offset` is 0
/// * `is_last_segment` is true
///
/// ## Segmented transfer
/// * `new_data` is 1 to 7 bytes long TODO 0 possible?
/// * `offset` is the number of bytes already written in a previous chunk
pub struct WriteData<'a> {
    pub index: u16,
    pub subindex: u8,
    pub new_data: &'a [u8],
    pub offset: usize,
    pub promised_size: Option<usize>,
    pub is_last_segment: bool,
}

pub struct BasicWriteData<'a>(&'a WriteData<'a>);

impl BasicWriteData<'_> {
    pub fn index(&self) -> u16 {
        self.0.index
    }
    pub fn subindex(&self) -> u8 {
        self.0.subindex
    }
}

/// A read-only view of [`WriteData`]
///
/// Can be directly consumed into basic objects.
/// ```
/// # use canopen::objectdictionary::datalink::WriteStream;
/// # fn demo(write_stream: WriteStream){
///  let data: u32 = write_stream.try_into().unwrap();
/// # }
///
/// ```
///
/// Can be written into a buffer.
/// ```
/// # use canopen::objectdictionary::datalink::WriteStream;
/// # fn demo(write_stream: WriteStream){
///  let mut buf = [0; 256];
///  write_stream.write_into(&mut buf).unwrap();
/// # }
/// ```
///
/// Derefs into [`WriteData`] for more advanced use cases.
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
            if promised_size > buf.len() {
                return Err(ODError::TooLong);
            }
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

macro_rules! try_from_impl {
    ($typ:ty) => {
        impl<'a> TryFrom<BasicWriteData<'a>> for $typ {
            type Error = ODError;

            fn try_from(data: BasicWriteData<'a>) -> Result<Self, Self::Error> {
                debug_assert!(data.0.offset == 0);
                debug_assert!(data.0.is_last_segment);

                check_size(data.0.new_data.len(), core::mem::size_of::<$typ>())?;
                if let Some(size) = data.0.promised_size {
                    check_size(size, core::mem::size_of::<$typ>())?;
                }
                let data = data.0.new_data.try_into().ok().unwrap();
                Ok(<$typ>::from_le_bytes(data))
            }
        }
        impl<'a> TryFrom<WriteStream<'a>> for $typ {
            type Error = ODError;

            fn try_from(data: WriteStream<'a>) -> Result<Self, Self::Error> {
                BasicWriteData(data.0).try_into()
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
try_from_impl!(f32);

impl<'a> TryFrom<BasicWriteData<'a>> for bool {
    type Error = ODError;

    fn try_from(data: BasicWriteData<'a>) -> Result<Self, Self::Error> {
        match u8::try_from(data)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ODError::InvalidValue),
        }
    }
}

fn check_size(given: usize, expected: usize) -> Result<(), ODError> {
    match given.cmp(&expected) {
        Ordering::Less => Err(ODError::TooShort),
        Ordering::Greater => Err(ODError::TooLong),
        Ordering::Equal => Ok(()),
    }
}
