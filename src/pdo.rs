use crate::ObjectDictionary;
use core::sync::atomic::AtomicU32;

struct _MappedObjects([AtomicU32; 8]);

pub struct TPDO<'a>(pub &'a ObjectDictionary<'a>);

impl TPDO<'_> {
    pub fn stuff(&mut self) {}
}
