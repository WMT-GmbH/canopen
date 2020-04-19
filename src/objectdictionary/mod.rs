pub mod data_store;

pub use data_store::DataLink::{CallbackDataLink, RefCellDataLink};
pub use data_store::DataStore;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

#[derive(Default)]
pub struct ObjectDictionary {
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
}

pub struct Variable {
    pub index: u16,
    pub subindex: u8,
}

impl Variable {
    pub fn get_unique_id(&self) -> u32 {
        ((self.index as u32) << 8) + self.subindex as u32
    }
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
