use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;
use manager::{remove_task, unreg_proc};
use proc::ProcControlBlock;
use processor::curr_proc;
use signal::SignalFlags;
use task_user_res::TaskUserRes;

use crate::{fs::inode::{OSInode, OpenFlag}, timer::sleep::remove_sleeptimer};

use self::{task::{TaskControlBlock, TaskStatus}, processor::{take_curr_task, schedule}, context::TaskContext};

pub mod switch;
pub mod context;
pub mod proc;
pub mod pid;
pub mod kern_stack;
pub mod processor;
mod manager;
mod allocator;
mod task_user_res;
pub mod signal;
pub mod task;

pub use manager::{add_task, get_proc};

lazy_static! {
    pub static ref INITPROC: Arc<ProcControlBlock> = {
        let inode = OSInode::open("initproc", OpenFlag::RDONLY).unwrap();
        let v = inode.read_app();
        ProcControlBlock::new(v.as_slice())
    };
}

pub fn add_initproc() {
    let _initproc = INITPROC.clone();
}

pub fn suspend_curr_task() {
    let task = take_curr_task().unwrap();
    let mut task_mut = task.get_mutpart();
    let task_cx_ptr = &mut task_mut.task_cx as *mut _;
    task_mut.task_status = TaskStatus::Ready;
    drop(task_mut);
    add_task(task);
    schedule(task_cx_ptr);
}

pub fn block_curr_task() {
    let task = take_curr_task().unwrap();
    let mut task_mut = task.get_mutpart();
    let task_cx_ptr = &mut task_mut.task_cx as *mut _;
    task_mut.task_status = TaskStatus::Blocked;
    drop(task_mut);
    schedule(task_cx_ptr);
}

pub fn block_without_schd() -> *mut TaskContext {
    let task = take_curr_task().unwrap();
    let mut task_mut = task.get_mutpart();
    task_mut.task_status = TaskStatus::Blocked;
    &mut task_mut.task_cx as *mut TaskContext
}

pub fn wakeup_task(task: Arc<TaskControlBlock>) {
    let mut task_mut = task.get_mutpart();
    task_mut.task_status = TaskStatus::Ready;
    drop(task_mut);
    add_task(task);
}

pub fn exit_curr_task(exit_code: i32) {
    let task = take_curr_task().unwrap();
    let mut task_mut = task.get_mutpart();
    let proc = task.proc.upgrade().unwrap();
    let tid = task_mut.res.as_ref().unwrap().tid;
    task_mut.exit_code = Some(exit_code);
    task_mut.res = None;
    drop(task_mut);
    drop(task);

    if tid != 0 {
        drop(proc);
        let mut _plh = TaskContext::zeros();
        schedule(&mut _plh as *mut TaskContext);
        return;
    }

    // main thread exit, process terminate
    let pid = proc.getpid();
    unreg_proc(pid);
    let mut proc_mut = proc.get_mutpart();
    proc_mut.is_zombie = true;
    proc_mut.exit_code = exit_code;

    let mut init_mut = INITPROC.get_mutpart();
    while let Some(child) = proc_mut.children.pop() {
        child.get_mutpart().parent = Some(Arc::downgrade(&INITPROC));
        init_mut.children.push(child);
    }
    drop(init_mut);

    let mut recycle_res = Vec::<TaskUserRes>::new();
    for task in proc_mut.tasks.iter()
        .filter(|t| t.is_some())
        .map(|t| t.as_ref().unwrap())
    {
        remove_inactive_task(task.clone());
        let mut task_mut = task.get_mutpart();
        if let Some(res) = task_mut.res.take() {
            recycle_res.push(res);
        }
    }
    // task_user_res drop need to access mutpart of proc
    // drop it first to prevent borrow error
    drop(proc_mut);
    recycle_res.clear();

    let mut proc_mut = proc.get_mutpart();
    proc_mut.children.clear();
    proc_mut.memset.mem_recycle();
    proc_mut.fd_table.clear();
    // Remain the main thread, because we need the kernel stack of main thread
    // The kernel stack will be recycled by parent process using waitpid.
    while proc_mut.tasks.len() > 1 {
        proc_mut.tasks.pop();
    }

    drop(proc_mut);
    drop(proc);
    let mut _plh = TaskContext::zeros();
    schedule(&mut _plh as *mut _);
}

fn remove_inactive_task(task: Arc<TaskControlBlock>) {
    remove_task(task.clone());
    remove_sleeptimer(task.clone());
}

pub fn send_signal_curr_proc(signal: SignalFlags) {
    curr_proc().get_mutpart().signals |= signal;
}
