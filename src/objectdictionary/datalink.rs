use crate::sdo::SDOAbortCode;
use core::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicU8, Ordering};

pub trait DataLink {
    fn size(&self) -> usize;
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<(), SDOAbortCode>;
    fn write(&self, data: &[u8], offset: usize, no_more_data: bool) -> Result<(), SDOAbortCode>;
}

impl DataLink for AtomicBool {
    fn size(&self) -> usize {
        1
    }
    fn read(&self, buf: &mut [u8], _offset: usize) -> Result<(), SDOAbortCode> {
        buf[0] = self.load(Ordering::Relaxed) as u8;
        Ok(())
    }

    fn write(&self, data: &[u8], _offset: usize, no_more_data: bool) -> Result<(), SDOAbortCode> {
        if data.len() != 1 {
            Err(SDOAbortCode::WrongLength)
        } else if data[0] > 1 {
            Err(SDOAbortCode::InvalidValue)
        } else {
            self.store(data[0] > 0, Ordering::Relaxed);
            Ok(())
        }
    }
}

impl DataLink for AtomicU8 {
    fn size(&self) -> usize {
        1
    }
    fn read(&self, buf: &mut [u8], _offset: usize) -> Result<(), SDOAbortCode> {
        // TODO check len, offset
        buf[0] = self.load(Ordering::Relaxed);
        Ok(())
    }

    fn write(&self, data: &[u8], _offset: usize, no_more_data: bool) -> Result<(), SDOAbortCode> {
        if data.len() != 1 {
            Err(SDOAbortCode::WrongLength)
        } else {
            self.store(data[0], Ordering::Relaxed);
            Ok(())
        }
    }
}

impl DataLink for AtomicU16 {
    fn size(&self) -> usize {
        2
    }
    fn read(&self, buf: &mut [u8], _offset: usize) -> Result<(), SDOAbortCode> {
        buf.copy_from_slice(&self.load(Ordering::Relaxed).to_le_bytes());
        Ok(())
    }

    fn write(&self, data: &[u8], _offset: usize, no_more_data: bool) -> Result<(), SDOAbortCode> {
        if data.len() != 2 {
            Err(SDOAbortCode::WrongLength)
        } else {
            let data = data.try_into().unwrap();
            self.store(u16::from_le_bytes(data), Ordering::Relaxed);
            Ok(())
        }
    }
}

impl DataLink for AtomicU32 {
    fn size(&self) -> usize {
        4
    }
    fn read(&self, buf: &mut [u8], _offset: usize) -> Result<(), SDOAbortCode> {
        buf.copy_from_slice(&self.load(Ordering::Relaxed).to_le_bytes());
        Ok(())
    }

    fn write(&self, data: &[u8], _offset: usize, no_more_data: bool) -> Result<(), SDOAbortCode> {
        if data.len() != 4 {
            Err(SDOAbortCode::WrongLength)
        } else {
            let data = data.try_into().unwrap();
            self.store(u32::from_le_bytes(data), Ordering::Relaxed);
            Ok(())
        }
    }
}
