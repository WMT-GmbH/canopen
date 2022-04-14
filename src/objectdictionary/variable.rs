use crate::objectdictionary::datalink::{AtomicDataLink, DataLink};
use crate::objectdictionary::odcell::ODCell;
use core::num::NonZeroU8;

#[derive(Clone)]
pub struct Variable<'a> {
    pub index: u16,
    pub subindex: u8,
    pub pdo_size: Option<NonZeroU8>,
    pub data: CANOpenData<'a>,
}

impl<'a> Variable<'a> {
    #[inline]
    pub fn new<T: Into<CANOpenData<'a>>>(index: u16, subindex: u8, data: T) -> Self {
        Variable {
            index,
            subindex,
            pdo_size: None,
            data: data.into(),
        }
    }
    #[inline]
    pub const fn new_datalink_cell(
        index: u16,
        subindex: u8,
        data: &'a ODCell<dyn DataLink>,
    ) -> Self {
        Variable {
            index,
            subindex,
            pdo_size: None,
            data: CANOpenData::DataLinkCell(data),
        }
    }
    #[inline]
    pub const fn new_datalink_ref(
        index: u16,
        subindex: u8,
        data: &'a dyn AtomicDataLink,
        pdo_size: Option<NonZeroU8>,
    ) -> Self {
        Variable {
            index,
            subindex,
            pdo_size,
            data: CANOpenData::DataLinkRef(data),
        }
    }
}

#[derive(Clone)]
pub enum CANOpenData<'a> {
    B1([u8; 1]),
    B2([u8; 2]),
    B4([u8; 4]),
    Bytes(&'a [u8]),
    DataLinkRef(&'a dyn AtomicDataLink),
    DataLinkCell(&'a ODCell<dyn DataLink>),
    ResourceNotAvailable,
}

macro_rules! from_impl {
    ($typ:ty, $variant:path) => {
        impl From<$typ> for CANOpenData<'_> {
            #[inline]
            fn from(val: $typ) -> Self {
                $variant(val.to_le_bytes())
            }
        }
    };
}

from_impl!(u8, CANOpenData::B1);
from_impl!(i8, CANOpenData::B1);
from_impl!(u16, CANOpenData::B2);
from_impl!(i16, CANOpenData::B2);
from_impl!(u32, CANOpenData::B4);
from_impl!(i32, CANOpenData::B4);

impl From<bool> for CANOpenData<'_> {
    #[inline]
    fn from(val: bool) -> Self {
        CANOpenData::B1([val as u8])
    }
}

impl<'a> From<&'a [u8]> for CANOpenData<'a> {
    #[inline]
    fn from(val: &'a [u8]) -> Self {
        CANOpenData::Bytes(val)
    }
}

impl<'a> From<&'a str> for CANOpenData<'a> {
    #[inline]
    fn from(val: &'a str) -> Self {
        CANOpenData::Bytes(val.as_bytes())
    }
}
