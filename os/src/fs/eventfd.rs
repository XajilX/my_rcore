use alloc::{collections::VecDeque, sync::Arc};
use bitflags::bitflags;

use crate::{sync::UThrCell, task::{block_curr_task, processor::curr_task, task::TaskControlBlock, wakeup_task}};

use super::File;

bitflags! {
    pub struct EventFdFlag: i32 {
        const SEMAPHORE = 1;
        const NONBLOCK  = 2048;
    }
}

enum EventFdMode {
    Semaphore,
    Normal,
}

pub struct EventFd {
    mode: EventFdMode,
    is_block: bool,
    inner: UThrCell<EventFdMut>
}

impl EventFd {
    pub fn new(initval: u32, flag: EventFdFlag) -> Self {
        let mode = if flag.contains(EventFdFlag::SEMAPHORE)
        { EventFdMode::Semaphore }
        else { EventFdMode::Normal };
        let is_block = !flag.contains(EventFdFlag::NONBLOCK);
        Self {
            mode,
            is_block,
            inner: unsafe {
                UThrCell::new(EventFdMut {
                    val: initval as u64,
                    wait_readers: VecDeque::new(),
                    wait_writers: VecDeque::new()
                })
            }
        }
    }
    fn try_read(&self) -> Option<u64> {
        let mut inner = self.inner.get_refmut();
        match self.mode {
            EventFdMode::Normal => {
                let mut var = 0u64;
                (var, inner.val) = (inner.val, var);
                if var == 0 { None } else { Some(var) }
            },
            EventFdMode::Semaphore => {
                if inner.val > 0 {
                    inner.val -= 1;
                    Some(1)
                } else {
                    None
                }
            }
        }
    }
}

impl File for EventFd {
    fn readable(&self) -> bool { true }

    fn writable(&self) -> bool { true }

    fn read(&self, buf: crate::mm::pagetab::UserBuffer) -> usize {
        if let Some(mut var) = self.try_read() {
            buf.into_iter().for_each(|b| unsafe {
                b.write_volatile((var & 255) as u8);
                var >>= 8;
            });
            let mut inner = self.inner.get_refmut();
            if let Some(task) = inner.wait_writers.pop_front() {
                wakeup_task(task);
            }
            8
        } else {
            if self.is_block {
                let task = curr_task().unwrap();
                let mut inner = self.inner.get_refmut();
                inner.wait_readers.push_back(task);
                drop(inner);
                block_curr_task();
                let mut var = self.try_read().unwrap();
                buf.into_iter().for_each(|b| unsafe {
                    b.write_volatile((var & 255) as u8);
                    var >>= 8;
                });
                let mut inner = self.inner.get_refmut();
                if let Some(task) = inner.wait_writers.pop_front() {
                    wakeup_task(task);
                }
                8
            } else {
                ((!2) + 1) as usize
            }
        }
    }

    fn write(&self, buf: crate::mm::pagetab::UserBuffer) -> usize {
        if buf.len() != 8 {
            return usize::MAX;
        }
        let mut inner = self.inner.get_refmut();
        match self.mode {
            EventFdMode::Semaphore => {
                inner.val += 1;
                if inner.val == 1 {
                    if let Some(task) = inner.wait_readers.pop_front() {
                        wakeup_task(task);
                    }
                }
                8
            },
            EventFdMode::Normal => {
                let mut var = 0u64;
                buf.into_iter().enumerate().for_each(|(i,x)| unsafe {
                    let b = x.read_volatile() as u64;
                    var |= b << (i * 8);
                });
                if var == u64::MAX {
                    return usize::MAX;
                }
                if let Some(var) = inner.val.checked_add(var)
                    .map_or(None, |x| if x == u64::MAX { None } else { Some(x) })
                {
                    inner.val = var;
                    if let Some(task) = inner.wait_readers.pop_front() {
                        wakeup_task(task);
                    }
                    8
                } else {
                    if self.is_block {
                        let task = curr_task().unwrap();
                        inner.wait_writers.push_back(task);
                        drop(inner);
                        block_curr_task();
                        let mut inner = self.inner.get_refmut();
                        inner.val += var;
                        if let Some(task) = inner.wait_readers.pop_front() {
                            wakeup_task(task);
                        }
                        8
                    } else {
                        ((!2) + 1) as usize
                    }
                }
            }
        }
    }
}

struct EventFdMut {
    val: u64,
    wait_readers: VecDeque<Arc<TaskControlBlock>>,
    wait_writers: VecDeque<Arc<TaskControlBlock>>
}
