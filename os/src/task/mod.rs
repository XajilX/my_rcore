use alloc::sync::Arc;
use lazy_static::lazy_static;

use crate::fs::inode::{OSInode, OpenFlag};

use self::{task::{ProcControlBlock, TaskStatus}, manager::add_proc, processor::{take_curr_proc, schedule}, context::TaskContext};

pub mod switch;
pub mod context;
pub mod task;
pub mod pid;
pub mod kern_stack;
pub mod processor;
pub mod manager;

lazy_static! {
    pub static ref INITPROC: Arc<ProcControlBlock> = {
        let inode = OSInode::open("initproc", OpenFlag::RDONLY).unwrap();
        let v = inode.read_app();
        Arc::new(ProcControlBlock::new(v.as_slice()))
    };
}

pub fn add_initproc() {
    add_proc(INITPROC.clone())
}

pub fn suspend_curr_task() {
    let proc = take_curr_proc().unwrap();
    let mut proc_mut = proc.get_mutpart();
    let proc_cx_ptr = &mut proc_mut.task_cx as *mut _;
    proc_mut.task_status = TaskStatus::Ready;
    drop(proc_mut);
    add_proc(proc);
    schedule(proc_cx_ptr);
}

pub fn exit_curr_task(exit_code: i32) {
    let proc = take_curr_proc().unwrap();
    let mut proc_mut = proc.get_mutpart();
    proc_mut.task_status = TaskStatus::Zombie;
    proc_mut.exit_code = exit_code;

    let mut init_mut = INITPROC.get_mutpart();
    while let Some(child) = proc_mut.children.pop() {
        child.get_mutpart().parent = Some(Arc::downgrade(&INITPROC));
        init_mut.children.push(child);
    }
    drop(init_mut);

    proc_mut.memset.mem_recycle();
    drop(proc_mut);
    drop(proc);
    let mut _plh = TaskContext::zeros();
    schedule(&mut _plh as *mut _);
}