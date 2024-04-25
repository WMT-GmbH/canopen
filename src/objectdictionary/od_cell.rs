pub struct OdCell<T> {
    pub value: T,
    pub locked: bool,
}

impl<T> OdCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            locked: false,
        }
    }
    pub fn is_locked(&self) -> bool {
        self.locked
    }
    // TODO docs about partial write going on
    pub fn get(&self) -> &T {
        &self.value
    }
    pub fn get_mut(&mut self) -> &mut T {
        self.locked = false;
        &mut self.value
    }
    pub(crate) fn get_mut_unchecked(&mut self) -> &mut T {
        &mut self.value
    }
    pub(crate) fn lock(&mut self) {
        self.locked = true;
    }
    pub(crate) fn unlock(&mut self) {
        self.locked = false;
    }
}
