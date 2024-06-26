use core::ops::Deref;

use lazy_static::lazy_static;

use crate::{task::allocator::IdAlloc, sync::UThrCell};

pub struct PidHandle(pub usize);

lazy_static! {
    static ref PID_ALLOC: UThrCell<IdAlloc> = unsafe {
        UThrCell::new(IdAlloc::new())
    };
}

pub fn pid_alloc() -> PidHandle {
    PidHandle(PID_ALLOC.get_refmut().alloc())
}
impl Deref for PidHandle {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Drop for PidHandle {
    fn drop(&mut self) {
        PID_ALLOC.get_refmut().dealloc(self.0);
    }
}
