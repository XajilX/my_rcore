use alloc::sync::Arc;

use crate::{fs::eventfd::{EventFd, EventFdFlag}, sync::{CondVar, Mutex, Semaphore}, task::{block_curr_task, proc::PCBMut, processor::{curr_proc, curr_task}}, timer::{get_time_ms, sleep::add_sleeptimer}};

pub fn sys_sleep(ms: usize) -> isize {
    let exp_ms = get_time_ms() + ms;
    let task = curr_task().unwrap();
    add_sleeptimer(exp_ms, task);
    block_curr_task();
    0
}

pub fn sys_mutex_create() -> isize {
    let proc = curr_proc();
    let mutex = Arc::new(Mutex::new());
    let mut proc_mut = proc.get_mutpart();
    let id = PCBMut::alloc_new_id(&mut proc_mut.mutexes);
    proc_mut.mutexes[id] = Some(mutex);
    id as isize
}

pub fn sys_mutex_lock(id: usize) -> isize {
    let proc = curr_proc();
    let proc_mut = proc.get_mutpart();
    let mutex = proc_mut.mutexes[id].as_ref().unwrap().clone();
    drop(proc_mut);
    mutex.lock();
    0
}

pub fn sys_mutex_unlock(id: usize) -> isize {
    let proc = curr_proc();
    let proc_mut = proc.get_mutpart();
    let mutex = proc_mut.mutexes[id].as_ref().unwrap().clone();
    drop(proc_mut);
    mutex.unlock();
    0
}

pub fn sys_sema_create(count: usize) -> isize {
    let proc = curr_proc();
    let sema = Arc::new(Semaphore::new(count));
    let mut proc_mut = proc.get_mutpart();
    let id = PCBMut::alloc_new_id(&mut proc_mut.semaphores);
    proc_mut.semaphores[id] = Some(sema);
    id as isize
}

pub fn sys_sema_plus(id: usize) -> isize {
    let proc = curr_proc();
    let proc_mut = proc.get_mutpart();
    let sema = proc_mut.semaphores[id].as_ref().unwrap().clone();
    drop(proc_mut);
    sema.plus();
    0
}

pub fn sys_sema_minus(id: usize) -> isize {
    let proc = curr_proc();
    let proc_mut = proc.get_mutpart();
    let sema = proc_mut.semaphores[id].as_ref().unwrap().clone();
    drop(proc_mut);
    sema.minus();
    0
}

pub fn sys_condv_create() -> isize {
    let proc = curr_proc();
    let condv = Arc::new(CondVar::new());
    let mut proc_mut = proc.get_mutpart();
    let id = PCBMut::alloc_new_id(&mut proc_mut.condvars);
    proc_mut.condvars[id] = Some(condv);
    id as isize
}

pub fn sys_condv_signal(id: usize) -> isize {
    let proc = curr_proc();
    let proc_mut = proc.get_mutpart();
    let condv = proc_mut.condvars[id].as_ref().unwrap().clone();
    drop(proc_mut);
    condv.signal();
    0
}

pub fn sys_condv_wait(id: usize, mutex_id: usize) -> isize {
    let proc = curr_proc();
    let proc_mut = proc.get_mutpart();
    let condv = proc_mut.condvars[id].as_ref().unwrap().clone();
    let mutex = proc_mut.mutexes[mutex_id].as_ref().unwrap().clone();
    drop(proc_mut);
    condv.wait_with_mutex(mutex);
    0
}

pub fn sys_eventfd(initval: u32, flag: i32) -> isize {
    let proc = curr_proc();
    let mut proc_mut = proc.get_mutpart();
    let id = PCBMut::alloc_new_id(&mut proc_mut.fd_table);
    if let Some(flag) = EventFdFlag::from_bits(flag) {
        proc_mut.fd_table[id] = Some(Arc::new(EventFd::new(initval, flag)));
        0
    } else {
        -1
    }
}
