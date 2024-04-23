use core::cell::Cell;
use core::cmp::Ordering;
use core::ops::Deref;

use crate::objectdictionary::{ODError, OdInfo};

pub trait DataLink {
    fn read(&self, index: u16, subindex: u8) -> Result<ReadData<'_>, ODError>;
    fn write(&mut self, write_stream: WriteStream<'_>, od_info: OdInfo<'_>) -> Result<(), ODError>;
}

pub trait BasicData {
    fn read(&self) -> impl Into<ReadData<'_>>;
    fn write(&mut self, write_stream: WriteStream<'_>) -> Result<(), ODError>;
}

impl<T: BasicData> DataLink for T {
    fn read(&self, _: u16, _: u8) -> Result<ReadData<'_>, ODError> {
        Ok(self.read().into())
    }

    fn write(&mut self, write_stream: WriteStream<'_>, _: OdInfo<'_>) -> Result<(), ODError> {
        self.write(write_stream)
    }
}

impl<const N: usize> DataLink for [u8; N] {
    fn read(&self, _: u16, _: u8) -> Result<ReadData<'_>, ODError> {
        Ok(self[..].into())
    }

    fn write(&mut self, write_stream: WriteStream<'_>, _: OdInfo<'_>) -> Result<(), ODError> {
        write_stream.write_into(self)?;
        Ok(())
    }
}

macro_rules! basic_data {
    ($typ:ty) => {
        impl BasicData for $typ {
            fn read(&self) -> impl Into<ReadData<'_>> {
                *self
            }

            fn write(&mut self, write_stream: WriteStream<'_>) -> Result<(), ODError> {
                *self = write_stream.try_into()?;
                Ok(())
            }
        }
        impl BasicData for Cell<$typ> {
            fn read(&self) -> impl Into<ReadData<'_>> {
                self.get()
            }

            fn write(&mut self, write_stream: WriteStream<'_>) -> Result<(), ODError> {
                self.set(write_stream.try_into()?);
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

impl BasicData for &[u8] {
    fn read(&self) -> impl Into<ReadData<'_>> {
        *self
    }

    fn write(&mut self, _: WriteStream<'_>) -> Result<(), ODError> {
        Err(ODError::ReadOnlyError)
    }
}

impl BasicData for &str {
    fn read(&self) -> impl Into<ReadData<'_>> {
        *self
    }

    fn write(&mut self, _: WriteStream<'_>) -> Result<(), ODError> {
        Err(ODError::ReadOnlyError)
    }
}

pub enum ReadData<'a> {
    B1([u8; 1]),
    B2([u8; 2]),
    B4([u8; 4]),
    Bytes(&'a [u8]),
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
    ($typ:ty, $variant:path, @primitive) => {
        impl From<$typ> for ReadData<'_> {
            #[inline]
            fn from(val: $typ) -> Self {
                $variant(val.to_le_bytes())
            }
        }
    };

    ($typ:ty, $variant:path, @array) => {
        impl From<$typ> for ReadData<'_> {
            #[inline]
            fn from(val: $typ) -> Self {
                $variant(val)
            }
        }
    };
}

read_data_from_impl!(u8, ReadData::B1, @primitive);
read_data_from_impl!(i8, ReadData::B1, @primitive);
read_data_from_impl!(u16, ReadData::B2, @primitive);
read_data_from_impl!(i16, ReadData::B2, @primitive);
read_data_from_impl!(u32, ReadData::B4, @primitive);
read_data_from_impl!(i32, ReadData::B4, @primitive);
read_data_from_impl!(f32, ReadData::B4, @primitive);

read_data_from_impl!([u8; 1], ReadData::B1, @array);
read_data_from_impl!([u8; 2], ReadData::B2, @array);
read_data_from_impl!([u8; 4], ReadData::B4, @array);

impl From<bool> for ReadData<'_> {
    #[inline]
    fn from(val: bool) -> Self {
        ReadData::from(val as u8)
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

/// A read-only view of [`WriteData`]
///
/// Can be directly consumed into basic objects.
/// ```
/// # use canopen::objectdictionary::datalink::WriteStream;
/// # fn demo(write_stream: WriteStream){
///  let data: [u8; 4] = write_stream.try_into().unwrap();
/// # }
/// # fn demo2(write_stream: WriteStream){
///  let data: i32 = write_stream.try_into().unwrap();
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
    ($typ:ty, @primitive) => {
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

    ($typ:ty, @array) => {
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

try_from_impl!([u8; 1], @array);
try_from_impl!([u8; 2], @array);
try_from_impl!([u8; 4], @array);

try_from_impl!(u8, @primitive);
try_from_impl!(i8, @primitive);
try_from_impl!(u16, @primitive);
try_from_impl!(i16, @primitive);
try_from_impl!(u32, @primitive);
try_from_impl!(i32, @primitive);
try_from_impl!(f32, @primitive);

impl<'a> TryFrom<WriteStream<'a>> for bool {
    type Error = ODError;

    fn try_from(write_stream: WriteStream<'a>) -> Result<Self, Self::Error> {
        match u8::try_from(write_stream)? {
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
