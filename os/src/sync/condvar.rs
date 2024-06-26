use alloc::{collections::VecDeque, sync::Arc};

use crate::task::{block_curr_task, block_without_schd, context::TaskContext, processor::curr_task, task::TaskControlBlock, wakeup_task};

use super::{Mutex, UThrCell};

pub struct CondVar {
    inner: UThrCell<CondvarMut>
}

pub struct CondvarMut {
    wait_queue: VecDeque<Arc<TaskControlBlock>>
}

impl CondVar {
    pub fn new() -> Self {
        Self { inner: unsafe {
            UThrCell::new(CondvarMut {
                wait_queue: VecDeque::new()
            })
        }}
    }

    // for create Interrupt-free environment
    pub fn wait_without_schd(&self) -> *mut TaskContext {
        self.inner.then(|i| {
            i.wait_queue.push_back(curr_task().unwrap());
        });
        block_without_schd()
    }

    pub fn wait_with_mutex(&self, mutex: Arc<Mutex>) {
        mutex.unlock();
        self.inner.then(|i| {
            i.wait_queue.push_back(curr_task().unwrap());
        });
        block_curr_task();
        mutex.lock();
    }

    pub fn signal(&self) {
        let mut cond_mut = self.inner.get_refmut();
        if let Some(task) = cond_mut.wait_queue.pop_front() {
            wakeup_task(task);
        }
    }
}