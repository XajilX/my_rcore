use core::ops::Deref;

use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::uthr::UThrCell;

pub struct PidHandle(pub usize);

struct PidAlloc {
    curr: usize,
    recycled: Vec<usize>
}

impl PidAlloc {
    pub fn new() -> Self {
        Self { curr: 0, recycled: Vec::new() }
    }
    pub fn alloc(&mut self) -> PidHandle {
        if let Some(pid) = self.recycled.pop() {
            PidHandle(pid)
        } else {
            self.curr += 1;
            PidHandle(self.curr - 1)
        }
    }
    pub fn dealloc(&mut self, pid: usize) {
        if  pid >= self.curr ||
            self.recycled.iter().find(|&&ph| ph == pid).is_some()
        {
            panic!("pid {} js not in use before dealloc", pid);
        }
        self.recycled.push(pid);
    }
}

lazy_static! {
    static ref PID_ALLOC: UThrCell<PidAlloc> = unsafe {
        UThrCell::new(PidAlloc::new())
    };
}

pub fn pid_alloc() -> PidHandle {
    PID_ALLOC.get_refmut().alloc()
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

