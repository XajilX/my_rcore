use alloc::{collections::VecDeque, sync::Arc};

use crate::task::{block_curr_task, processor::curr_task, task::TaskControlBlock, wakeup_task};

use super::UThrCell;


pub struct Semaphore {
    inner: UThrCell<SemaphoreMut>
}

pub struct SemaphoreMut {
    count: isize,
    wait_queue: VecDeque<Arc<TaskControlBlock>>
}

impl Semaphore {
    pub fn new(count: usize) -> Self {
        Self { inner: unsafe {
            UThrCell::new(SemaphoreMut {
                count: count as isize,
                wait_queue: VecDeque::new()
            })
        }}
    } 

    pub fn plus(&self) {
        let mut sema_mut = self.inner.get_refmut();
        sema_mut.count += 1;
        if sema_mut.count <= 0 {
            if let Some(task) = sema_mut.wait_queue.pop_front() {
                wakeup_task(task);
            }
        }
    }

    pub fn minus(&self) {
        let mut sema_mut = self.inner.get_refmut();
        sema_mut.count -= 1;
        if sema_mut.count < 0 {
            sema_mut.wait_queue.push_back(curr_task().unwrap());
            drop(sema_mut);
            block_curr_task();
        }
    }
}