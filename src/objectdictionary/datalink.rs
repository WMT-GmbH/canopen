use crate::sdo::SDOAbortCode;
use core::num::NonZeroUsize;
use core::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicU8, Ordering};

pub trait DataLink {
    fn size(&self) -> Option<NonZeroUsize>;
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<(), SDOAbortCode>;
    fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode>;
}

pub struct WriteStream<'a> {
    pub index: u16,
    pub subindex: u8,
    pub new_data: &'a [u8],
    pub offset: usize,
    pub is_last_segment: bool,
}

impl DataLink for AtomicBool {
    fn size(&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(1)
    }
    fn read(&self, buf: &mut [u8], _offset: usize) -> Result<(), SDOAbortCode> {
        buf[0] = self.load(Ordering::Relaxed) as u8;
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

impl DataLink for AtomicU8 {
    fn size(&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(1)
    }
    fn read(&self, buf: &mut [u8], _offset: usize) -> Result<(), SDOAbortCode> {
        // TODO check len, offset
        buf[0] = self.load(Ordering::Relaxed);
        Ok(())
    }
    fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        self.store(write_stream.new_data[0], Ordering::Relaxed);
        Ok(())
    }
}

impl DataLink for AtomicU16 {
    fn size(&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(2)
    }
    fn read(&self, buf: &mut [u8], _offset: usize) -> Result<(), SDOAbortCode> {
        buf.copy_from_slice(&self.load(Ordering::Relaxed).to_le_bytes());
        Ok(())
    }
    fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        if let Ok(data) = write_stream.new_data.try_into() {
            self.store(u16::from_le_bytes(data), Ordering::Relaxed);
        }
        Ok(())
    }
}

impl DataLink for AtomicU32 {
    fn size(&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(4)
    }
    fn read(&self, buf: &mut [u8], _offset: usize) -> Result<(), SDOAbortCode> {
        buf.copy_from_slice(&self.load(Ordering::Relaxed).to_le_bytes());
        Ok(())
    }

    fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        if let Ok(data) = write_stream.new_data.try_into() {
            self.store(u32::from_le_bytes(data), Ordering::Relaxed);
        }
        Ok(())
    }
}
