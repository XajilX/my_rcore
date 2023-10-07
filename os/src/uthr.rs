// UniTHReadCell

use core::cell::{RefCell, RefMut};

pub struct UThrCell<T> {
    inner: RefCell<T>
}

unsafe impl<T> Sync for UThrCell<T> {}

impl<T> UThrCell<T> {
    pub unsafe fn new(value: T) -> Self {
        Self { inner: RefCell::new(value) }
    }
    pub fn get_refmut(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}