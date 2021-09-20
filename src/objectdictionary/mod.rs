pub use datatypes::CANOpenDataType;
pub use variable::Variable;

pub mod datalink;
pub mod datatypes;
pub mod variable;

pub struct ObjectDictionary<'a> {
    pub objects: &'a [Variable<'a>],
}

pub enum ODError {
    IndexDoesNotExist,
    SubindexDoesNotExist,
}

impl ObjectDictionary<'_> {
    pub fn new<'a>(objects: &'a [Variable<'_>]) -> ObjectDictionary<'a> {
        ObjectDictionary { objects }
    }

    pub fn get(&self, index: u16, subindex: u8) -> Result<&Variable<'_>, ODError> {
        match self
            .objects
            .binary_search_by(|obj| (obj.index, obj.subindex).cmp(&(index, subindex)))
        {
            Ok(pos) => Ok(&self.objects[pos]),
            Err(pos) => {
                // If there is an object with the same index but different subindex
                // we need to return ODError::SubindexDoesNotExist.

                // Binary search will return the index at which one could insert the searched for variable
                // If an object with the same index exists, this index with point into to or just past this object.
                // So if the variables at pos and pos-1 do not match, such an object cannot exist.
                if pos < self.objects.len() && self.objects[pos].index == index {
                    return Err(ODError::SubindexDoesNotExist);
                }
                if pos != 0 && self.objects[pos - 1].index == index {
                    return Err(ODError::SubindexDoesNotExist);
                }
                Err(ODError::IndexDoesNotExist)
            }
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
