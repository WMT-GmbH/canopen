use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::objectdictionary::Variable;
use crate::sdo::errors::SDOAbortCode::ResourceNotAvailable;
use crate::sdo::SDOAbortCode;
use core::cell::RefCell;

pub enum DataLink {
    RefCellDataLink(RefCell<Vec<u8>>), // TODO will have to be mutable
    CallbackDataLink,
}

#[derive(Default)]
pub struct DataStore(BTreeMap<u32, DataLink>);

impl DataStore {
    pub fn add_data_link(&mut self, variable: &Variable, data_link: DataLink) {
        self.0.insert(variable.unique_id, data_link);
    }

    pub fn get_data_link(&self, variable: &Variable) -> Option<&DataLink> {
        self.0.get(&variable.unique_id)
    }

    pub fn get_data(&self, variable: &Variable) -> Result<Vec<u8>, SDOAbortCode> {
        match self.get_data_link(variable) {
            None => Err(ResourceNotAvailable),
            Some(DataLink::RefCellDataLink(cell)) => Ok(cell.borrow().clone()),
            Some(DataLink::CallbackDataLink) => Ok(Vec::new()),
        }
        // TODO check length, readable, clone, default
    }

    pub fn set_data(&mut self, variable: &Variable, data: Vec<u8>) -> Result<(), SDOAbortCode> {
        match self
            .0
            .entry(variable.unique_id)
            .or_insert(DataLink::RefCellDataLink(RefCell::new(vec![])))
        {
            DataLink::RefCellDataLink(cell) => {
                *cell.borrow_mut() = data;
            }
            DataLink::CallbackDataLink => {}
        };
        Ok(())
    }
}
