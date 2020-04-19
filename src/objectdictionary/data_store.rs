use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::objectdictionary::Variable;
use crate::sdo::errors::SDOAbortCode::ResourceNotAvailable;
use crate::sdo::SDOAbortCode;
use core::cell::RefCell;

pub enum DataLink {
    Value(RefCell<Vec<u8>>), // TODO will have to be mutable
    Callbacks,
}

#[derive(Default)]
pub struct DataStore(BTreeMap<u32, DataLink>);

impl DataStore {
    pub fn add_data_link(&mut self, variable: &Variable, data_link: DataLink) {
        self.0.insert(variable.get_unique_id(), data_link);
    }

    pub fn get_data_link(&self, variable: &Variable) -> Option<&DataLink> {
        self.0.get(&variable.get_unique_id())
    }

    pub fn get_data(&self, variable: &Variable) -> Result<Vec<u8>, SDOAbortCode> {
        match self.get_data_link(variable) {
            None => Err(ResourceNotAvailable),
            Some(DataLink::Value(cell)) => Ok(cell.borrow().clone()),
            Some(DataLink::Callbacks) => Ok(Vec::new()),
        }
        // TODO check length, readable, clone, default
    }

    pub fn set_data(&mut self, variable: &Variable, data: Vec<u8>) -> Result<(), SDOAbortCode> {
        match self
            .0
            .entry(variable.get_unique_id())
            .or_insert(DataLink::Value(RefCell::new(vec![])))
        {
            DataLink::Value(cell) => {
                *cell.borrow_mut() = data;
            }
            DataLink::Callbacks => {}
        };
        Ok(())
    }
}
