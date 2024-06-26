use core::convert::identity;

use alloc::sync::Arc;

use crate::{mm::memset::KERN_SPACE, task::{add_task, processor::{curr_proc, curr_task}, task::TaskControlBlock}, trap::{context::TrapContext, trap_handler}};

pub fn sys_thrdcreate(entry: usize, arg: usize) -> isize {
    let task = curr_task().unwrap();
    let proc = curr_proc();
    let new_task = Arc::new(TaskControlBlock::new(
        proc.clone(),
        task.get_mutpart().res.as_ref().unwrap().ustack_base,
        true
    ));
    add_task(new_task.clone());
    let new_task_mut = new_task.get_mutpart();
    let new_task_res = new_task_mut.res.as_ref().unwrap();
    let new_task_tid = new_task_res.tid;
    let mut proc_mut = proc.get_mutpart();
    while proc_mut.tasks.len() <= new_task_tid {
        proc_mut.tasks.push(None);
    }
    proc_mut.tasks[new_task_tid] = Some(new_task.clone());
    let new_trap_cx = new_task_mut.get_trap_cx();
    *new_trap_cx = TrapContext::app_init_context(
        entry, 
        new_task_res.ustack_top(),
        KERN_SPACE.get_refmut().get_atp_token(),
        new_task.kern_stack.get_top(),
        trap_handler as usize
    );
    (*new_trap_cx).reg[10] = arg;
    new_task_tid as isize
}

pub fn sys_gettid() -> isize {
    let task = curr_task().unwrap();
    let task_mut = task.get_mutpart();
    task_mut.res.as_ref().unwrap().tid  as isize
}

pub fn sys_waittid(tid: usize) -> i32 {
    let task = curr_task().unwrap();
    let proc = task.proc.upgrade().unwrap();
    let task_mut = task.get_mutpart();
    let mut proc_mut = proc.get_mutpart();
    if task_mut.res.as_ref().unwrap().tid == tid {
        return -1;
    }
    let ret = if let Some(Some(waited_task)) = proc_mut.tasks.get(tid) {
        if let Some(exit_code) = waited_task.get_mutpart().exit_code {
            Ok(exit_code)
        } else {
            Err(-2)
        }
    } else {
        Err(-1)
    };
    if ret.is_ok() {
        proc_mut.tasks[tid] = None;
    }
    ret.map_or_else(identity, identity)
}