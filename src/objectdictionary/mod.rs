pub use datatypes::CANOpenDataType;
pub use variable::Variable;

pub mod datalink;
pub mod datatypes;
pub mod variable;

#[derive(Copy, Clone)]
pub struct ObjectDictionary<'a> {
    pub objects: &'a [Variable<'a>],
}

pub enum ODError {
    IndexDoesNotExist,
    SubindexDoesNotExist,
}

impl ObjectDictionary<'_> {
    pub const fn new<'a>(objects: &'a [Variable<'_>]) -> ObjectDictionary<'a> {
        ObjectDictionary { objects }
    }

    pub fn get(&self, index: u16, subindex: u8) -> Result<&Variable<'_>, ODError> {
        let position = self.get_position(index, subindex)?;
        Ok(&self.objects[position])
    }

    pub fn get_position(&self, index: u16, subindex: u8) -> Result<usize, ODError> {
        match self
            .objects
            .binary_search_by(|obj| (obj.index, obj.subindex).cmp(&(index, subindex)))
        {
            Ok(position) => Ok(position),
            Err(position) => {
                // If there is an object with the same index but different subindex
                // we need to return ODError::SubindexDoesNotExist.

                // Binary search will return the position at which one could insert
                // the searched for variable.
                // If an object with the same index exists, the returned position will point into
                // or just past such an object.

                // So if the variables at position and position - 1 do not match,
                // such an object cannot exist.
                if position < self.objects.len() && self.objects[position].index == index {
                    return Err(ODError::SubindexDoesNotExist);
                }
                if position != 0 && self.objects[position - 1].index == index {
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
