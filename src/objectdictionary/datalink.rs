use core::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicU8, Ordering};

pub trait DataLink {
    fn size(&self) -> usize;
    fn read(&self, buf: &mut [u8], offset: usize);
    fn write(&self, data: &[u8], offset: usize);
}

impl DataLink for AtomicBool {
    fn size(&self) -> usize {
        1
    }
    fn read(&self, buf: &mut [u8], _offset: usize) {
        buf[0] = self.load(Ordering::Relaxed) as u8
    }

    fn write(&self, data: &[u8], _offset: usize) {
        // TODO check bool
        self.store(data[0] > 0, Ordering::Relaxed);
    }
}

impl DataLink for AtomicU8 {
    fn size(&self) -> usize {
        1
    }
    fn read(&self, buf: &mut [u8], _offset: usize) {
        // TODO check len, offset
        buf[0] = self.load(Ordering::Relaxed)
    }

    fn write(&self, data: &[u8], _offset: usize) {
        // TODO check len, offset
        self.store(data[0], Ordering::Relaxed);
    }
}

impl DataLink for AtomicU16 {
    fn size(&self) -> usize {
        2
    }
    fn read(&self, buf: &mut [u8], _offset: usize) {
        buf.copy_from_slice(&self.load(Ordering::Relaxed).to_le_bytes());
    }

    fn write(&self, data: &[u8], _offset: usize) {
        let data = data.try_into().unwrap();
        self.store(u16::from_le_bytes(data), Ordering::Relaxed);
    }
}

impl DataLink for AtomicU32 {
    fn size(&self) -> usize {
        4
    }
    fn read(&self, buf: &mut [u8], _offset: usize) {
        buf.copy_from_slice(&self.load(Ordering::Relaxed).to_le_bytes());
    }

    fn write(&self, data: &[u8], _offset: usize) {
        let data = data.try_into().unwrap();
        self.store(u32::from_le_bytes(data), Ordering::Relaxed);
    }
}
