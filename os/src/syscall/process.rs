use alloc::{string::String, sync::Arc, vec::Vec};
use log::{debug, error};

use crate::{fs::inode::{OSInode, OpenFlag}, mm::pagetab::PageTab, task::{exit_curr_task, get_proc, processor::{curr_atp_token, curr_proc}, signal::{SignalFlags, MAX_SIG}, suspend_curr_task}, timer::get_time_ms};

pub fn sys_yield() -> isize {
    suspend_curr_task();
    0
}

pub fn sys_exit(xstate: i32) -> ! {
    exit_curr_task(xstate);
    unreachable!()
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    curr_proc().getpid() as isize
}

pub fn sys_fork() -> isize {
    let curr = curr_proc();
    let new_proc = curr.fork();
    let new_pid = new_proc.getpid();
    let new_proc_mut = new_proc.get_mutpart();
    let task = new_proc_mut.tasks[0].as_ref().unwrap();
    let new_trap_cx = task.get_mutpart().get_trap_cx();
    new_trap_cx.reg[10] = 0;
    new_pid as _
}

pub fn sys_exec(path: *const u8, mut args: *const usize) -> isize {
    let token = curr_atp_token();
    let path = PageTab::from_token(token).trans_cstr(path);
    debug!("sys_exec {}", path);
    if let Some(inode) = OSInode::open(path.as_str(), OpenFlag::RDONLY) {
        let data = inode.read_app();
        let proc = curr_proc();
        let mut vec_args: Vec<String> = Vec::new();
        unsafe { loop {
            let arg_ptr = *(PageTab::from_token(token).trans_ref(args));
            if arg_ptr == 0 {
                break;
            }
            vec_args.push(PageTab::from_token(token).trans_cstr(arg_ptr as *const u8));
            args = args.add(1);
        }};
        proc.exec(data.as_slice(), &vec_args);
        // return argc to reg a0 as first argument to user function
        vec_args.len() as isize
    } else {
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let proc = curr_proc();
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
            (pid == -1 || pc.getpid() == pid as usize) && 
            pc.get_mutpart().is_zombie
        });
    if let Some((idx, _)) = pair {
        let child = proc_mut.children.swap_remove(idx);
        assert_eq!(Arc::strong_count(&child), 1);
        let child_pid = child.getpid();
        let exit_code = child.get_mutpart().exit_code;
        *(PageTab::from_token(proc_mut.get_atp_token())
            .trans_mut(exit_code_ptr)
        ) = exit_code;
        child_pid as isize
    } else {
        -2
    }
}

pub fn sys_kill(pid: usize, signum: i32) -> isize {
    if signum as usize > MAX_SIG {
        error!("Signal ID incorrect");
        return -1;
    }
    let flag = SignalFlags::from_bits(1 << signum).unwrap();
    if let Some(proc) = get_proc(pid) {
        let mut inner = proc.get_mutpart();
        if !inner.signals.contains(flag) {
            inner.signals.insert(flag);
            0
        } else {
            error!("Already got the same signal");
            -1
        }
    } else {
        error!("PID incorrect");
        -1
    }
}
