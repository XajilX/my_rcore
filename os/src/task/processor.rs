use alloc::sync::Arc;
use lazy_static::lazy_static;
use log::debug;

use crate::{uthr::UThrCell, trap::context::TrapContext};

use super::{task::{ProcControlBlock, TaskStatus}, context::TaskContext, switch::__switch, manager::fetch_proc};

pub struct Processor {
    curr_proc: Option<Arc<ProcControlBlock>>,
    idle_task_cx: TaskContext
}
impl Processor {
    pub fn new() -> Self {
        Self {
            curr_proc: None,
            idle_task_cx: TaskContext::zeros()
        }
    }
    pub fn take_curr_proc(&mut self) -> Option<Arc<ProcControlBlock>> {
        self.curr_proc.take()
    }
    pub fn curr_proc(&self) -> Option<Arc<ProcControlBlock>> {
        self.curr_proc.as_ref().map(|proc| Arc::clone(proc))
    }
}

lazy_static! {
    pub static ref PROCESSOR: UThrCell<Processor> = unsafe {
        UThrCell::new(Processor::new())
    };
}

pub fn take_curr_proc() -> Option<Arc<ProcControlBlock>> {
    PROCESSOR.get_refmut().take_curr_proc()
}

pub fn curr_proc() -> Option<Arc<ProcControlBlock>> {
    PROCESSOR.get_refmut().curr_proc()
}

pub fn curr_atp_token() -> usize {
    curr_proc().unwrap().get_mutpart().get_atp_token()
}

pub fn curr_trap_cx() -> &'static mut TrapContext {
    curr_proc().unwrap().get_mutpart().get_trap_cx()
}

pub fn run_proc() {
    loop {
        let mut procor = PROCESSOR.get_refmut();
        if let Some(proc) = fetch_proc() {
            debug!("switch to next proc");
            let idle_task_cx_ptr = &mut procor.idle_task_cx as *mut _;
            let mut proc_mut = proc.get_mutpart();
            let next_task_cx_ptr = &proc_mut.task_cx as *const _;
            proc_mut.task_status = TaskStatus::Running;
            drop(proc_mut);
            procor.curr_proc = Some(proc);
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