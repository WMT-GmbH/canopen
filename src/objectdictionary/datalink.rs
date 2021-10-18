use core::num::NonZeroUsize;
use core::sync::atomic::*;

use crate::sdo::SDOAbortCode;

pub trait DataLink: Sync {
    fn size(&self) -> Option<NonZeroUsize>;
    fn read(&self, read_stream: &mut ReadStream<'_>) -> Result<(), SDOAbortCode>; // TODO switch to ODError
    fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode>;
}

pub struct WriteStream<'a> {
    pub index: u16,
    pub subindex: u8,
    pub new_data: &'a [u8],
    pub offset: usize,
    pub is_last_segment: bool,
}

pub struct ReadStream<'a> {
    pub index: u16,
    pub subindex: u8,
    pub buf: &'a mut [u8],
    pub total_bytes_read: &'a mut usize,
    pub is_last_segment: bool,
}

macro_rules! atomic_impl {
    ($typ:ty, $backing_typ:ty) => {
        impl DataLink for $typ {
            fn size(&self) -> Option<NonZeroUsize> {
                NonZeroUsize::new(core::mem::size_of::<$typ>())
            }
            fn read(&self, read_stream: &mut ReadStream<'_>) -> Result<(), SDOAbortCode> {
                read_stream
                    .buf
                    .copy_from_slice(&self.load(Ordering::Relaxed).to_le_bytes());
                Ok(())
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
            fn size(&self) -> Option<NonZeroUsize> {
                NonZeroUsize::new(core::mem::size_of::<$typ>())
            }
            fn read(&self, read_stream: &mut ReadStream<'_>) -> Result<(), SDOAbortCode> {
                read_stream.buf.copy_from_slice(&self.to_le_bytes());
                Ok(())
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

impl DataLink for AtomicBool {
    fn size(&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(1)
    }
    fn read(&self, read_stream: &mut ReadStream<'_>) -> Result<(), SDOAbortCode> {
        read_stream.buf[0] = self.load(Ordering::Relaxed) as u8;
        Ok(())
    }
    fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        if write_stream.new_data[0] > 0 {
            Err(SDOAbortCode::InvalidValue)
        } else {
            self.store(write_stream.new_data[0] > 0, Ordering::Relaxed);
            Ok(())
        }
    }
}

impl DataLink for bool {
    fn size(&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(1)
    }
    fn read(&self, read_stream: &mut ReadStream<'_>) -> Result<(), SDOAbortCode> {
        read_stream.buf[0] = *self as u8;
        Ok(())
    }
    fn write(&self, _write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        Err(SDOAbortCode::ReadOnlyError)
    }
}

impl DataLink for &str {
    fn size(&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(self.len())
    }
    fn read(&self, read_stream: &mut ReadStream<'_>) -> Result<(), SDOAbortCode> {
        self.as_bytes().read(read_stream)
    }
    fn write(&self, _write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        Err(SDOAbortCode::ReadOnlyError)
    }
}

impl DataLink for &[u8] {
    fn size(&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(self.len())
    }
    fn read(&self, read_stream: &mut ReadStream<'_>) -> Result<(), SDOAbortCode> {
        let unread_data = &self[*read_stream.total_bytes_read..];

        let new_data_len = if unread_data.len() <= read_stream.buf.len() {
            read_stream.is_last_segment = true;
            unread_data.len()
        } else {
            read_stream.buf.len()
        };

        read_stream.buf[..new_data_len].copy_from_slice(&unread_data[..new_data_len]);
        *read_stream.total_bytes_read += new_data_len;

        Ok(())
    }
    fn write(&self, _write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        Err(SDOAbortCode::ReadOnlyError)
    }
}

/*
impl<T: DataLink, const N: usize> DataLink for [T; N] {
    fn size(&self) -> Option<NonZeroUsize> {
        todo!()
    }

    fn read(&self, read_stream: &mut ReadStream<'_>) -> Result<(), SDOAbortCode> {
        self[read_stream.subindex as usize].read(read_stream)
    }

    fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        self[write_stream.subindex as usize].write(write_stream)
    }
}
*/
