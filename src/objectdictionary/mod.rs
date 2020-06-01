pub mod data_store;
pub mod datatypes;
pub mod variable;

pub use data_store::DataLink::{CallbackDataLink, RefCellDataLink};
pub use data_store::DataStore;
pub use datatypes::CANOpenDataType;
pub use variable::Variable;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

#[derive(Default)]
pub struct ObjectDictionary {
    // TODO use fixed sized array and slice binary_search_by instead of BTreeMap
    indices: BTreeMap<u16, Object>,
}

impl ObjectDictionary {
    pub fn get(&self, index: u16) -> Option<&Object> {
        self.indices.get(&index)
    }

    pub fn add_object(&mut self, object: Object) {
        let index = match &object {
            Object::Variable(variable) => variable.index,
            Object::Array(array) => array.index,
            Object::Record(record) => record.index,
        };
        self.indices.insert(index, object);
    }

    pub fn add_variable(&mut self, variable: Variable) {
        self.indices
            .insert(variable.index, Object::Variable(variable));
    }

    pub fn add_array(&mut self, array: Array) {
        self.indices.insert(array.index, Object::Array(array));
    }

    pub fn add_record(&mut self, record: Record) {
        self.indices.insert(record.index, Object::Record(record));
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
