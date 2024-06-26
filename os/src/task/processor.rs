use alloc::sync::Arc;
use lazy_static::lazy_static;
use log::trace;

use crate::{sync::UThrCell, trap::context::TrapContext};

use super::{context::TaskContext, manager::fetch_task, proc::ProcControlBlock, switch::__switch, task::{TaskControlBlock, TaskStatus}};

pub struct Processor {
    curr_task: Option<Arc<TaskControlBlock>>,
    idle_task_cx: TaskContext
}
impl Processor {
    pub fn new() -> Self {
        Self {
            curr_task: None,
            idle_task_cx: TaskContext::zeros()
        }
    }
    pub fn take_curr_task(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.curr_task.take()
    }
    pub fn curr_task(&self) -> Option<Arc<TaskControlBlock>> {
        self.curr_task.as_ref().map(|proc| Arc::clone(proc))
    }
}

lazy_static! {
    pub static ref PROCESSOR: UThrCell<Processor> = unsafe {
        UThrCell::new(Processor::new())
    };
}

pub fn take_curr_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.get_refmut().take_curr_task()
}

pub fn curr_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.get_refmut().curr_task()
}

pub fn curr_proc() -> Arc<ProcControlBlock> {
    curr_task().unwrap().proc.upgrade().unwrap()
}

pub fn curr_atp_token() -> usize {
    curr_proc().get_mutpart().get_atp_token()
}

pub fn curr_trap_cx() -> &'static mut TrapContext {
    curr_task().unwrap().get_mutpart().get_trap_cx()
}

pub fn curr_trap_va() -> usize {
    curr_task().unwrap().get_mutpart().res.as_ref().unwrap()
        .trap_cx_va()
}

pub fn run_task() {
    loop {
        let mut procor = PROCESSOR.get_refmut();
        if let Some(task) = fetch_task() {
            trace!("switch to next task. pid: {}, tid: {}",
                task.proc.upgrade().unwrap().getpid(),
                task.get_mutpart().res.as_ref().unwrap().tid
            );
            let idle_task_cx_ptr = &mut procor.idle_task_cx as *mut _;
            let mut task_mut = task.get_mutpart();
            let next_task_cx_ptr = &task_mut.task_cx as *const _;
            task_mut.task_status = TaskStatus::Running;
            drop(task_mut);
            procor.curr_task = Some(task);
            drop(procor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        }
    }
}

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let procor = PROCESSOR.get_refmut();
    let idle_task_cx_ptr = &procor.idle_task_cx as *const _;
    drop(procor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr)
    }
}