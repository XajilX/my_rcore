use core::{arch::asm, ffi::CStr};

// use crate::SignalAction;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe { asm!(
        "ecall",
        inlateout("x10") args[0] => ret,
        in("x11") args[1],
        in("x12") args[2],
        in("x17") id
    )};
    ret
}

const SYSCALL_DUP: usize = 24;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_SLEEP: usize = 101;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_KILL: usize = 129;
// const SYSCALL_SIGACTION: usize = 134;
// const SYSCALL_SIGPROCMASK: usize = 135;
// const SYSCALL_SIGRETURN: usize = 139;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

const SYSCALL_THRDCREATE: usize = 1000;
const SYSCALL_GETTID: usize = 1001;
const SYSCALL_WAITTID: usize = 1002;

const SYSCALL_MUTEXCREATE: usize = 1010;
const SYSCALL_MUTEXLOCK: usize = 1011;
const SYSCALL_MUTEXUNLOCK: usize = 1012;
const SYSCALL_SEMACREATE: usize = 1020;
const SYSCALL_SEMAPLUS: usize = 1021;
const SYSCALL_SEMAMINUS: usize = 1022;
const SYSCALL_CONDVCREATE: usize = 1030;
const SYSCALL_CONDVSIGNAL: usize = 1031;
const SYSCALL_CONDVWAIT: usize = 1032;


pub fn sys_read(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_READ, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(xstate: i32) -> isize {
    syscall(SYSCALL_EXIT, [xstate as usize, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0])
}

pub fn sys_waitpid(pid: isize, xcode: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, xcode as usize, 0])
}

pub fn sys_exec(path: &CStr, args: &[*const u8]) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, args.as_ptr() as usize, 0])
}

pub fn sys_open(path: &CStr, flags: u32) -> isize {
    syscall(SYSCALL_OPEN, [path.as_ptr() as usize, flags as usize, 0])
}

pub fn sys_close(fd: usize) -> isize {
    syscall(SYSCALL_CLOSE, [fd, 0, 0])
}

pub fn sys_pipe(pipe: &mut [usize]) -> isize {
    syscall(SYSCALL_PIPE, [pipe.as_mut_ptr() as usize, 0, 0])
}

pub fn sys_dup(fd: usize) -> isize {
    syscall(SYSCALL_DUP, [fd, 0, 0])
}

pub fn sys_kill(pid: usize, signum: i32) -> isize {
    syscall(SYSCALL_KILL, [pid, signum as usize, 0])
}

/*
pub fn sys_sigaction(
    signum: i32,
    action: *const SignalAction,
    old_action: *mut SignalAction
) -> isize {
    syscall(SYSCALL_SIGACTION, [signum as usize, action as usize, old_action as usize])
}

pub fn sys_sigprocmask(mask: u32) -> isize {
    syscall(SYSCALL_SIGPROCMASK, [mask as usize, 0, 0])
}

pub fn sys_sigreturn() -> isize {
    syscall(SYSCALL_SIGRETURN, [0, 0, 0])
}
*/

pub fn sys_sleep(ms: usize) -> isize {
    syscall(SYSCALL_SLEEP, [ms, 0, 0])
}

pub fn sys_thrd_create(entry: usize, arg: usize) -> isize {
    syscall(SYSCALL_THRDCREATE, [entry, arg, 0])
}

pub fn sys_gettid() -> isize {
    syscall(SYSCALL_GETTID, [0, 0, 0])
}

pub fn sys_waittid(id: usize) -> isize {
    syscall(SYSCALL_WAITTID, [id, 0, 0])
}

pub fn sys_mutex_create() -> isize {
    syscall(SYSCALL_MUTEXCREATE, [0, 0, 0])
}

pub fn sys_mutex_lock(id: usize) -> isize {
    syscall(SYSCALL_MUTEXLOCK, [id, 0, 0])
}

pub fn sys_mutex_unlock(id: usize) -> isize {
    syscall(SYSCALL_MUTEXUNLOCK, [id, 0, 0])
}

pub fn sys_semaphore_create(count: usize) -> isize {
    syscall(SYSCALL_SEMACREATE, [count, 0, 0])
}

pub fn sys_semaphore_plus(id: usize) -> isize {
    syscall(SYSCALL_SEMAPLUS, [id, 0, 0])
}

pub fn sys_semaphore_minus(id: usize) -> isize {
    syscall(SYSCALL_SEMAMINUS, [id, 0, 0])
}

pub fn sys_condvar_create() -> isize {
    syscall(SYSCALL_CONDVCREATE, [0, 0, 0])
}

pub fn sys_condvar_signal(id: usize) -> isize {
    syscall(SYSCALL_CONDVSIGNAL, [id, 0, 0])
}

pub fn sys_condvar_wait(id: usize, mutex_id: usize) -> isize {
    syscall(SYSCALL_CONDVWAIT, [id, mutex_id, 0])
}
