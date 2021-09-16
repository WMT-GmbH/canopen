pub mod data_store;
pub mod datatypes;
pub mod variable;

pub use data_store::DataLink::{CallbackDataLink, RefCellDataLink};
pub use data_store::DataStore;
pub use datatypes::CANOpenDataType;
pub use variable::Variable;

use alloc::vec::Vec;

pub struct ObjectDictionary {
    // TODO use fixed sized array and slice binary_search_by instead of BTreeMap
    pub objects: [Object; 36],
}

impl ObjectDictionary {
    pub fn get(&self, index: u16) -> Option<&Object> {
        if let Ok(pos) = self.objects.binary_search_by_key(&index, |obj| match obj {
            Object::Variable(obj) => obj.index,
            Object::Array(obj) => obj.index,
            Object::Record(obj) => obj.index,
        }) {
            Some(&self.objects[pos])
        } else {
            None
        }
    }
}

/* TODO lifetime stuff
use core::ops::Index;

impl Index<u16> for ObjectDictionary{
    type Output = Option<&Object>;

    fn index(&self, index: u16) -> Self::Output {
        self.indices.get(&index)
    }
}*/

pub enum Object {
    Variable(Variable),
    Array(Array),
    Record(Record),
}

pub struct Array {
    pub index: u16,
    pub members: Vec<Variable>,
}

impl Array {
    pub fn get(&self, subindex: u8) -> Option<&Variable> {
        self.members.get(subindex as usize)
    }
}

pub struct Record {
    pub index: u16,
    pub members: Vec<Variable>,
}

impl Record {
    pub fn get(&self, subindex: u8) -> Option<&Variable> {
        self.members.get(subindex as usize)
    }
}
