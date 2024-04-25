use crate::objectdictionary::OdPosition;

#[derive(Clone, Debug)]
pub struct ObjectInfo {
    pub index: u16,
    pub subindex: u8,
    pub flags: ObjectFlags,
    pub od_position: OdPosition,
}

#[derive(Clone, Copy)]
pub struct ObjectFlags(u8);

impl ObjectFlags {
    const READ_ONLY_FLAG: u8 = 0b0001_0000;
    const WRITE_ONLY_FLAG: u8 = 0b0010_0000;

    pub const fn empty() -> Self {
        ObjectFlags(0)
    }
    pub const fn set_read_only(mut self) -> Self {
        assert!(
            !self.is_write_only(),
            "Object cannot be both read-only and write-only"
        );
        self.0 |= Self::READ_ONLY_FLAG;
        self
    }
    pub const fn set_write_only(mut self) -> Self {
        assert!(
            !self.is_read_only(),
            "Object cannot be both read-only and write-only"
        );
        self.0 |= Self::WRITE_ONLY_FLAG;
        self
    }
    pub const fn set_pdo_size(mut self, size: PdoSize) -> Self {
        self.0 |= size as u8;
        self
    }

    pub const fn is_read_only(&self) -> bool {
        self.0 & Self::READ_ONLY_FLAG != 0
    }
    pub const fn is_write_only(&self) -> bool {
        self.0 & Self::WRITE_ONLY_FLAG != 0
    }
    pub const fn pdo_size(&self) -> Option<PdoSize> {
        match self.0 & 0b0000_0111 {
            0 => None,
            1 => Some(PdoSize::One),
            2 => Some(PdoSize::Two),
            4 => Some(PdoSize::Four),
            _ => unsafe { core::hint::unreachable_unchecked() }, // Other bit patterns cannot be created
        }
    }
}

impl Default for ObjectFlags {
    fn default() -> Self {
        Self::empty()
    }
}

impl core::fmt::Debug for ObjectFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ObjectFlags")
            .field("read_only", &self.is_read_only())
            .field("write_only", &self.is_write_only())
            .field("pdo_size", &self.pdo_size())
            .finish()
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PdoSize {
    One = 1,
    Two = 2,
    Four = 4,
}

impl PdoSize {
    pub const fn new(val: u8) -> Option<Self> {
        match val {
            1 => Some(PdoSize::One),
            2 => Some(PdoSize::Two),
            4 => Some(PdoSize::Four),
            _ => None,
        }
    }
    pub const fn get(self) -> u8 {
        self as u8
    }
}
