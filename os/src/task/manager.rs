use alloc::{collections::VecDeque, sync::Arc};

use lazy_static::lazy_static;

use crate::uthr::UThrCell;

use super::task::ProcControlBlock;


pub struct ProcMan {
    ready_queue: VecDeque<Arc<ProcControlBlock>>
}
impl ProcMan {
    pub fn new() -> Self {
        Self { ready_queue: VecDeque::new() }
    }
    pub fn add(&mut self, proc: Arc<ProcControlBlock>) {
        self.ready_queue.push_back(proc);
    }
    pub fn fetch(&mut self) -> Option<Arc<ProcControlBlock>> {
        self.ready_queue.pop_front()
    }
}

lazy_static! {
    pub static ref PROCMAN: UThrCell<ProcMan> = unsafe {
        UThrCell::new(ProcMan::new())
    };
}

pub fn add_proc(proc: Arc<ProcControlBlock>) {
    PROCMAN.get_refmut().add(proc);
}
pub fn fetch_proc() -> Option<Arc<ProcControlBlock>> {
    PROCMAN.get_refmut().fetch()
}
