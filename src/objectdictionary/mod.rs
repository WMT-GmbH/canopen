use alloc::collections::BTreeMap;

#[derive(Default)]
pub struct ObjectDictionary{
    indices: BTreeMap<u16, Object>,
}

impl ObjectDictionary{
    pub fn get(&self, index: u16) -> Option<&Object>{
        self.indices.get(&index)
    }

    pub fn add_object(&mut self, object: Object){
        let index = match object {
            Object::Variable(index, _)=> index
        };
        self.indices.insert(index, object);
    }
}

/* TODO lifetime stuff
use core::ops::Index;

impl Index<u16> for ObjectDictionary{
    type Output = Option<&Object>;

    fn index(&self, index: u16) -> Self::Output {
        self.indicies.get(&index)
    }
}*/

pub enum Object {
    Variable(u16, u8)
}

impl Object{
    pub fn get_index(&self) -> u16 {
        match self { Object::Variable(index, _) => *index, }
    }
}

struct VariableObject{
    index: u16,
    subindex: u8
}
