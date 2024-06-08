use alloc::sync::Arc;
use log::debug;

use crate::{fs::inode::{OSInode, OpenFlag}, mm::pagetab::PageTab, task::{exit_curr_task, manager::add_proc, processor::{curr_atp_token, curr_proc}, suspend_curr_task}, timer::get_time_ms};

pub fn sys_yield() -> isize {
    suspend_curr_task();
    0
}

pub fn sys_exit(xstate: i32) -> ! {
    println!("[kernel] Application exited with code {}", xstate);
    exit_curr_task(xstate);
    unreachable!()
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    curr_proc().unwrap().getpid() as isize
}

pub fn sys_fork() -> isize {
    let curr = curr_proc().unwrap();
    let new_proc = curr.fork();
    let new_pid = new_proc.getpid();
    let new_trap_cx = new_proc.get_mutpart().get_trap_cx();
    new_trap_cx.reg[10] = 0;
    add_proc(new_proc);
    new_pid as _
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = curr_atp_token();
    let path = PageTab::from_token(token).trans_cstr(path);
    debug!("sys_exec {}", path);
    if let Some(inode) = OSInode::open(path.as_str(), OpenFlag::RDONLY) {
        let data = inode.read_app();
        let proc = curr_proc().unwrap();
        proc.exec(data.as_slice());
        0
    } else {
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let proc = curr_proc().unwrap();
    let mut proc_mut = proc.get_mutpart();
    if !proc_mut.children
        .iter()
        .any(|pc| pid == -1 || pid as usize == pc.getpid())
    {
        return -1;
    }
    let pair = proc_mut.children
        .iter()
        .enumerate()
        .find(|(_, pc)| {
            (pid == -1 || pc.getpid() == pid as usize) && pc.get_mutpart().is_zombie()
        });
    if let Some((idx, _)) = pair {
        let child = proc_mut.children.swap_remove(idx);
        assert_eq!(Arc::strong_count(&child), 1);
        let child_pid = child.getpid();
        let exit_code = child.get_mutpart().exit_code;
        *(PageTab::from_token(proc_mut.get_atp_token())
            .trans_va((exit_code_ptr as usize).into())
            .unwrap()
            .get_mut()
        ) = exit_code;
        child_pid as isize
    } else {
        -2
    }
}