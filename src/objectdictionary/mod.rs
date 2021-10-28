pub use datatypes::CANOpenDataType;
pub use variable::Variable;

pub mod datalink;
pub mod datatypes;
pub mod variable;

pub type ObjectDictionary<'a> = &'a [Variable<'a>];

pub trait ObjectDictionaryExt<'a> {
    fn find(&self, index: u16, subindex: u8) -> Result<&'a Variable<'a>, ODError>;
}

impl<'a> ObjectDictionaryExt<'a> for ObjectDictionary<'a> {
    fn find(&self, index: u16, subindex: u8) -> Result<&'a Variable<'a>, ODError> {
        match self.binary_search_by(|obj| (obj.index, obj.subindex).cmp(&(index, subindex))) {
            Ok(position) => Ok(&self[position]),
            Err(position) => {
                // If there is an object with the same index but different subindex
                // we need to return ODError::SubindexDoesNotExist.

                // Binary search will return the position at which one could insert
                // the searched for variable.
                // If an object with the same index exists, the returned position will point into
                // or just past such an object.

                // So if the variables at position and position - 1 do not match,
                // such an object cannot exist.
                if position < self.len() && self[position].index == index {
                    return Err(ODError::SubindexDoesNotExist);
                }
                if position != 0 && self[position - 1].index == index {
                    return Err(ODError::SubindexDoesNotExist);
                }
                Err(ODError::IndexDoesNotExist)
            }
        }
    }
}

pub enum ODError {
    IndexDoesNotExist,
    SubindexDoesNotExist,
}
