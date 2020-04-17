use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::network::Network;
use crate::objectdictionary;
use crate::objectdictionary::Object;
use crate::sdo::errors::SDOAbortCode;

pub struct Node<'a> {
    pub network: &'a dyn Network,
    pub od: objectdictionary::ObjectDictionary,
    pub data_store: BTreeMap<u32, Vec<u8>>,
}

impl<'a> Node<'a> {
    pub fn new(network: &'a dyn Network, od: objectdictionary::ObjectDictionary) -> Node<'a> {
        Node {
            network,
            od,
            data_store: BTreeMap::new(),
        }
    }

    pub fn get_data(&self, index: u16, subindex: u8) -> Result<Vec<u8>, SDOAbortCode> {
        let _variable = self.find_variable(index, subindex)?;
        // TODO check if readable
        if index == 1 {
            return Ok(vec![1, 2, 3, 4]);
        }
        Ok(vec![1, 2, 3, 4, 5])
    }

    pub fn set_data(
        &mut self,
        index: u16,
        subindex: u8,
        data: Vec<u8>,
    ) -> Result<(), SDOAbortCode> {
        // TODO check if writable
        let variable = self.find_variable(index, subindex)?;
        let id = variable.get_unique_id();
        self.data_store.insert(id, data);
        Ok(())
    }

    fn find_variable(
        &self,
        index: u16,
        subindex: u8,
    ) -> Result<&objectdictionary::Variable, SDOAbortCode> {
        let object = self.od.get(index).ok_or(SDOAbortCode::ObjectDoesNotExist)?;

        match object {
            Object::Variable(variable) => Ok(variable),
            Object::Array(array) => Ok(array
                .get(subindex)
                .ok_or(SDOAbortCode::SubindexDoesNotExist)?),
        }
    }
}
