use core::cell::Cell;
use core::num::NonZeroUsize;
use core::ops::Deref;
use core::sync::atomic::*;

use crate::sdo::SDOAbortCode;

pub trait DataLink {
    fn size(&self, index: u16, subindex: u8) -> Option<NonZeroUsize>;
    fn read<'rs>(&self, read_stream: ReadStream<'rs>) -> Result<UsedReadStream<'rs>, SDOAbortCode>; // TODO switch to ODError
    fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode>;
}

pub struct WriteStream<'a> {
    pub index: u16,
    pub subindex: u8,
    pub new_data: &'a [u8],
    pub offset: usize,
    pub is_last_segment: bool,
}

pub struct ReadStreamData<'a> {
    pub index: u16,
    pub subindex: u8,
    pub(crate) buf: &'a mut [u8],
    pub(crate) total_bytes_read: &'a mut usize,
    pub(crate) is_last_segment: bool,
}

pub struct ReadStream<'a>(pub(crate) &'a mut ReadStreamData<'a>);
pub struct UsedReadStream<'a>(pub(crate) &'a mut ReadStreamData<'a>);

impl<'a> Deref for ReadStream<'a> {
    type Target = ReadStreamData<'a>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/*impl ReadStream<'_> {
    #[inline]
    pub fn read_u8(mut self, val: u8) {
        self.buf[0] = val;
        *self.total_bytes_read += 1;
        self.is_last_segment = true;
    }
    #[inline]
    pub fn read_i8(self, val: i8) {
        self.read_u8(val as u8)
    }
    #[inline]
    pub fn read_bool(self, val: bool) {
        self.read_u8(val as u8)
    }
    #[inline]
    pub fn read_u16(mut self, val: u16) {
        self.buf[0..2].copy_from_slice(&val.to_le_bytes());
        *self.total_bytes_read += 2;
        self.is_last_segment = true;
    }
    #[inline]
    pub fn read_i16(self, val: i16) {
        self.read_u16(val as u16)
    }
    #[inline]
    pub fn read_u32(mut self, val: u32) {
        self.buf[0..4].copy_from_slice(&val.to_le_bytes());
        *self.total_bytes_read += 4;
        self.is_last_segment = true;
    }
    #[inline]
    pub fn read_i32(self, val: i32) {
        self.read_u32(val as u32)
    }
    pub fn read_bytes(mut self, data: &[u8]) {
        let unread_data = &data[*self.total_bytes_read..];

        let new_data_len = if unread_data.len() <= self.buf.len() {
            self.is_last_segment = true;
            unread_data.len()
        } else {
            self.buf.len()
        };

        self.buf[..new_data_len].copy_from_slice(&unread_data[..new_data_len]);
        *self.total_bytes_read += new_data_len;
    }
}*/

macro_rules! cell_impl {
    ($typ:ty) => {
        impl DataLink for Cell<$typ> {
            fn size(&self, _index: u16, _subindex: u8) -> Option<NonZeroUsize> {
                NonZeroUsize::new(core::mem::size_of::<$typ>())
            }
            fn read<'rs>(
                &self,
                read_stream: ReadStream<'rs>,
            ) -> Result<UsedReadStream<'rs>, SDOAbortCode> {
                self.get().read(read_stream)
            }
            fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
                if let Ok(data) = write_stream.new_data.try_into() {
                    self.set(<$typ>::from_le_bytes(data));
                }
                Ok(())
            }
        }
    };
}

cell_impl!(u8);
cell_impl!(u16);
cell_impl!(u32);

cell_impl!(i8);
cell_impl!(i16);
cell_impl!(i32);

macro_rules! atomic_impl {
    ($typ:ty, $backing_typ:ty) => {
        impl DataLink for $typ {
            fn size(&self, _index: u16, _subindex: u8) -> Option<NonZeroUsize> {
                NonZeroUsize::new(core::mem::size_of::<$typ>())
            }
            fn read<'rs>(
                &self,
                read_stream: ReadStream<'rs>,
            ) -> Result<UsedReadStream<'rs>, SDOAbortCode> {
                self.load(Ordering::Relaxed).read(read_stream)
            }
            fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
                if let Ok(data) = write_stream.new_data.try_into() {
                    self.store(<$backing_typ>::from_le_bytes(data), Ordering::Relaxed);
                }
                Ok(())
            }
        }
    };
}

atomic_impl!(AtomicU8, u8);
atomic_impl!(AtomicU16, u16);
atomic_impl!(AtomicU32, u32);

atomic_impl!(AtomicI8, i8);
atomic_impl!(AtomicI16, i16);
atomic_impl!(AtomicI32, i32);

macro_rules! readonly_impl {
    ($typ:ty) => {
        impl DataLink for $typ {
            fn size(&self, _index: u16, _subindex: u8) -> Option<NonZeroUsize> {
                NonZeroUsize::new(core::mem::size_of::<$typ>())
            }
            #[inline]
            fn read<'rs>(
                &self,
                mut read_stream: ReadStream<'rs>,
            ) -> Result<UsedReadStream<'rs>, SDOAbortCode> {
                let size = core::mem::size_of::<$typ>();
                read_stream.0.buf[0..size].copy_from_slice(&self.to_le_bytes());
                *read_stream.0.total_bytes_read += size;
                read_stream.0.is_last_segment = true;

                Ok(UsedReadStream(read_stream.0))
            }
            fn write(&self, _write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
                Err(SDOAbortCode::ReadOnlyError)
            }
        }
    };
}

readonly_impl!(u8);
readonly_impl!(u16);
readonly_impl!(u32);

readonly_impl!(i8);
readonly_impl!(i16);
readonly_impl!(i32);

impl DataLink for Cell<bool> {
    fn size(&self, _index: u16, _subindex: u8) -> Option<NonZeroUsize> {
        NonZeroUsize::new(core::mem::size_of::<bool>())
    }
    #[inline]
    fn read<'rs>(&self, read_stream: ReadStream<'rs>) -> Result<UsedReadStream<'rs>, SDOAbortCode> {
        self.get().read(read_stream)
    }
    fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        if write_stream.new_data[0] > 1 {
            Err(SDOAbortCode::InvalidValue)
        } else {
            self.set(write_stream.new_data[0] > 0);
            Ok(())
        }
    }
}

impl DataLink for AtomicBool {
    fn size(&self, _index: u16, _subindex: u8) -> Option<NonZeroUsize> {
        NonZeroUsize::new(1)
    }
    #[inline]
    fn read<'rs>(&self, read_stream: ReadStream<'rs>) -> Result<UsedReadStream<'rs>, SDOAbortCode> {
        self.load(Ordering::Relaxed).read(read_stream)
    }
    fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        if write_stream.new_data[0] > 1 {
            Err(SDOAbortCode::InvalidValue)
        } else {
            self.store(write_stream.new_data[0] > 0, Ordering::Relaxed);
            Ok(())
        }
    }
}

impl DataLink for bool {
    fn size(&self, _index: u16, _subindex: u8) -> Option<NonZeroUsize> {
        NonZeroUsize::new(1)
    }
    #[inline]
    fn read<'rs>(&self, read_stream: ReadStream<'rs>) -> Result<UsedReadStream<'rs>, SDOAbortCode> {
        (*self as u8).read(read_stream)
    }
    fn write(&self, _write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        Err(SDOAbortCode::ReadOnlyError)
    }
}

impl DataLink for &str {
    fn size(&self, _index: u16, _subindex: u8) -> Option<NonZeroUsize> {
        NonZeroUsize::new(self.len())
    }
    #[inline]
    fn read<'rs>(&self, read_stream: ReadStream<'rs>) -> Result<UsedReadStream<'rs>, SDOAbortCode> {
        self.as_bytes().read(read_stream)
    }
    fn write(&self, _write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        Err(SDOAbortCode::ReadOnlyError)
    }
}

impl DataLink for &[u8] {
    fn size(&self, _index: u16, _subindex: u8) -> Option<NonZeroUsize> {
        NonZeroUsize::new(self.len())
    }
    fn read<'rs>(
        &self,
        mut read_stream: ReadStream<'rs>,
    ) -> Result<UsedReadStream<'rs>, SDOAbortCode> {
        let unread_data = &self[*read_stream.0.total_bytes_read..];

        let new_data_len = if unread_data.len() <= read_stream.0.buf.len() {
            read_stream.0.is_last_segment = true;
            unread_data.len()
        } else {
            read_stream.0.buf.len()
        };

        read_stream.0.buf[..new_data_len].copy_from_slice(&unread_data[..new_data_len]);
        *read_stream.0.total_bytes_read += new_data_len;

        Ok(UsedReadStream(read_stream.0))
    }
    fn write(&self, _write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        Err(SDOAbortCode::ReadOnlyError)
    }
}

pub struct ResourceNotAvailable;

impl DataLink for ResourceNotAvailable {
    fn size(&self, _index: u16, _subindex: u8) -> Option<NonZeroUsize> {
        None
    }

    fn read<'rs>(
        &self,
        mut _read_stream: ReadStream<'rs>,
    ) -> Result<UsedReadStream<'rs>, SDOAbortCode> {
        Err(SDOAbortCode::ResourceNotAvailable)
    }

    fn write(&self, _write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        Err(SDOAbortCode::ResourceNotAvailable)
    }
}

/*
impl<T: DataLink, const N: usize> DataLink for [T; N] {
    fn size(&self, _index: u16, _subindex: u8) -> Option<NonZeroUsize> {
        todo!()
    }

    fn read(&self, mut read_stream: ReadStream<'_>) -> ResuUsedReadStream<(), SDOAbortCode> {
        self[read_stream.subindex as usize].read(read_stream)
    }

    fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        self[write_stream.subindex as usize].write(write_stream)
    }
}
*/
