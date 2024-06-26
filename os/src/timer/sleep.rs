use alloc::{collections::BinaryHeap, sync::Arc};
use lazy_static::lazy_static;

use crate::{task::{task::TaskControlBlock, wakeup_task}, sync::UThrCell};

use super::get_time_ms;

pub struct SleepTimer {
    pub expire_ms: usize,
    pub task: Arc<TaskControlBlock>
}

impl PartialEq for SleepTimer {
    fn eq(&self, other: &Self) -> bool {
        self.expire_ms == other.expire_ms
    }
}
impl Eq for SleepTimer {}
impl PartialOrd for SleepTimer {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let (a, b) = (-(self.expire_ms as isize), -(other.expire_ms as isize));
        Some(a.cmp(&b))
    }
}
impl Ord for SleepTimer {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

lazy_static! {
    static ref SLEEPTIMERS: UThrCell<BinaryHeap<SleepTimer>> = unsafe {
        UThrCell::new(BinaryHeap::<SleepTimer>::new())
    };
}

pub fn add_sleeptimer(expire_ms: usize, task: Arc<TaskControlBlock>) {
    let mut timers = SLEEPTIMERS.get_refmut();
    timers.push(SleepTimer { expire_ms, task });
}

pub fn remove_sleeptimer(task: Arc<TaskControlBlock>) {
    let mut timers = SLEEPTIMERS.get_refmut();
    timers.retain(|tm| {
        Arc::as_ptr(&tm.task) != Arc::as_ptr(&task)
    });
}

pub fn check_sleeptimer() {
    let curr_ms = get_time_ms();
    let mut timers = SLEEPTIMERS.get_refmut();
    while let Some(tm) = timers.peek() {
        if tm.expire_ms <= curr_ms {
            wakeup_task(tm.task.clone());
            timers.pop();
        } else {
            break;
        }
    }
}