use alloc::{collections::VecDeque, sync::Arc};

use crate::task::{block_curr_task, processor::curr_task, task::TaskControlBlock, wakeup_task};

use super::UThrCell;

pub struct Mutex {
    inner: UThrCell<MutexMut>
}

pub struct MutexMut {
    locked: bool,
    wait_queue: VecDeque<Arc<TaskControlBlock>>
}

impl Mutex {
    pub fn new() -> Self {
        Self { inner: unsafe {
            UThrCell::new(MutexMut {
                locked: false, 
                wait_queue: VecDeque::new()
            })
        }}
    }

    pub fn lock(&self) {
        let mut mutex_mut = self.inner.get_refmut();
        if mutex_mut.locked {
            mutex_mut.wait_queue.push_back(curr_task().unwrap());
            drop(mutex_mut);
            block_curr_task();
        } else {
            mutex_mut.locked = true;
        }
    }

    pub fn unlock(&self) {
        let mut mutex_mut = self.inner.get_refmut();
        assert!(mutex_mut.locked);
        if let Some(task) = mutex_mut.wait_queue.pop_front() {
            wakeup_task(task)
        } else {
            mutex_mut.locked = false;
        }
    }
}