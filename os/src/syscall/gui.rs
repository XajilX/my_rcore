use alloc::sync::Arc;

use crate::{drivers::GPU_DEV, fs::fb::Fb, task::{proc::PCBMut, processor::curr_proc}};

pub fn sys_get_gpures() -> isize {
    let (xres, yres) = GPU_DEV.get_refmut().as_ref().unwrap().resolution();
    ((xres as isize) << 32) | (yres as isize)
}

pub fn sys_get_fbfd() -> isize {
    let proc = curr_proc();
    let mut inner = proc.get_mutpart();
    let fd = PCBMut::alloc_new_id(&mut inner.fd_table);
    inner.fd_table[fd] = Some(Arc::new(Fb::new()));
    fd as isize
}