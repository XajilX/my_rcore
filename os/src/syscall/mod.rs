mod process;
mod gui;
use log::trace;
use process::*;

mod fs;
use fs::*;

mod thread;
use thread::*;

mod sync;
use sync::*;

mod input;
use input::*;

use crate::syscall::gui::{sys_get_fbfd, sys_get_gpures};


const SYSCALL_DUP: usize = 24;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_SEEK: usize = 62;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_SLEEP: usize = 101;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_KILL: usize = 129;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_EVENTFD: usize = 290;

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
const SYSCALL_GETGPURES: usize = 2000;
const SYSCALL_GETFBFD: usize = 2001;
const SYSCALL_INPUTEVENT: usize = 3000;


pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    trace!("syscall catched, id: {syscall_id}");
    match syscall_id {
        SYSCALL_DUP => sys_dup(args[0]),
        SYSCALL_OPEN => sys_open(args[0] as *const u8, args[1] as u32),
        SYSCALL_CLOSE => sys_close(args[0] as usize),
        SYSCALL_PIPE => sys_pipe(args[0] as *mut usize),
        SYSCALL_SEEK => sys_seek(args[0], args[1] as isize, args[2]),
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_SLEEP => sys_sleep(args[0]),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_KILL => sys_kill(args[0] as usize, args[1] as i32),
        SYSCALL_GET_TIME => sys_get_time(),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8, args[1] as *const usize),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_EVENTFD => sys_eventfd(args[0] as u32, args[1] as i32),
        SYSCALL_THRDCREATE => sys_thrdcreate(args[0], args[1]),
        SYSCALL_GETTID => sys_gettid(),
        SYSCALL_WAITTID => sys_waittid(args[0]) as isize,
        SYSCALL_MUTEXCREATE => sys_mutex_create(),
        SYSCALL_MUTEXLOCK => sys_mutex_lock(args[0]),
        SYSCALL_MUTEXUNLOCK => sys_mutex_unlock(args[0]),
        SYSCALL_SEMACREATE => sys_sema_create(args[0]),
        SYSCALL_SEMAPLUS => sys_sema_plus(args[0]),
        SYSCALL_SEMAMINUS => sys_sema_minus(args[0]),
        SYSCALL_CONDVCREATE => sys_condv_create(),
        SYSCALL_CONDVSIGNAL => sys_condv_signal(args[0]),
        SYSCALL_CONDVWAIT => sys_condv_wait(args[0], args[1]),
        SYSCALL_GETGPURES => sys_get_gpures(),
        SYSCALL_GETFBFD => sys_get_fbfd(),
        SYSCALL_INPUTEVENT => sys_input_event(),
        _ => panic!("Unsupported syscall_id: {}", syscall_id)
    }
}