#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
pub mod console;
pub mod signum;
mod syscall;
mod lang_items;
pub mod graphics;
mod input;

use input::InputEvent;
pub use signum::*;
pub use graphics::Display;

/// ------------------------------------------------------------
/// ------------------------------------------------------------
/// Linkage
/// ------------------------------------------------------------
/// ------------------------------------------------------------

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start(argc: usize, argv: usize) -> ! {
    unsafe {
        HEAP.lock().init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
    let v: Vec<&str> = (0..argc).map(|i| unsafe {
        CStr::from_ptr(*(argv as *const usize).add(i) as *const i8)
            .to_str().unwrap()
    }).collect();
    exit(main(argc,v.as_slice()))
}

#[linkage = "weak"]
#[no_mangle]
fn main(_argc: usize, _argv: &[&str]) -> i32 {
    panic!("Cannot find main function");
}

use core::ffi::CStr;

use bitflags::bitflags;
/// ------------------------------------------------------------
/// ------------------------------------------------------------
/// Heap allocation
/// ------------------------------------------------------------
/// ------------------------------------------------------------

use buddy_system_allocator::LockedHeap;
const USER_HEAP_SIZE: usize = 16324;
static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];
extern crate alloc;

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn alloc_err_handler(layout: core::alloc::Layout) -> ! {
    panic!("Heap alloc error! layout = {:?}", layout);
}


/// ------------------------------------------------------------
/// ------------------------------------------------------------
/// System calls
/// ------------------------------------------------------------
/// ------------------------------------------------------------



use syscall::*;
use alloc::{ffi::CString, vec::Vec};

pub fn read(fd: usize, buffer: &[u8]) -> isize {
    sys_read(fd, buffer)
}

pub fn write(fd: usize, buffer: &[u8]) -> isize {
    sys_write(fd, buffer)
}

pub fn seek(fd: usize, offset: isize, whence: usize) -> isize {
    sys_seek(fd, offset, whence)
}

pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
    unreachable!()
}

pub fn yield_() -> isize {
    sys_yield()
}

pub fn get_time() -> isize {
    sys_get_time()
}

pub fn getpid() -> isize {
    sys_getpid()
}

pub fn fork() -> isize {
    sys_fork()
}

pub fn exec(path: &str, args: &[*const u8]) -> isize {
    let path_ = CString::new(path).map_or_else(
        |_| CString::from_vec_with_nul(Vec::from(path)),
        |x| Ok(x)
    ).unwrap();
    sys_exec(&path_, args)
}

pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => { yield_(); }
            exit_pid => { return exit_pid; }
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => { yield_(); }
            exit_pid => { return exit_pid; }
        }
    }
}

pub fn sleep(ms: usize) {
    sys_sleep(ms);
} 

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

pub fn open(path: &str, flags: OpenFlags) -> isize {
    let path_ = CString::new(path).map_or_else(
        |_| CString::from_vec_with_nul(Vec::from(path)),
        |x| Ok(x)
    ).unwrap();
    sys_open(&path_, flags.bits())
}

pub fn close(fd: usize) -> isize {
    sys_close(fd)
}

pub fn pipe(pipe_fd: &mut [usize]) -> isize {
    sys_pipe(pipe_fd)
}

pub fn dup(fd: usize) -> isize {
    sys_dup(fd)
}

pub fn kill(pid: usize, signum: i32) -> isize {
    sys_kill(pid, signum)
}

#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct SignalAction {
    pub handler: usize,
    pub mask: SignalFlags
}

impl Default for SignalAction {
    fn default() -> Self {
        Self {
            handler: 0,
            mask: SignalFlags::empty()
        }
    }
}

/*
pub fn sigaction(
    signum: i32,
    action: Option<&SignalAction>,
    old_action: Option<&mut SignalAction>
) -> isize {
    sys_sigaction(
        signum,
        action.map_or(ptr::null(), |a| a),
        old_action.map_or(ptr::null_mut(), |a| a))
}

pub fn sigprocmask(mask: u32) -> isize {
    sys_sigprocmask(mask)
}

pub fn sigreturn() -> isize {
    sys_sigreturn()
}

*/

pub fn thread_create(entry: usize, arg: usize) -> isize {
    sys_thrd_create(entry, arg)
}
pub fn gettid() -> isize {
    sys_gettid()
}
pub fn waittid(tid: usize) -> isize {
    loop {
        match sys_waittid(tid) {
            -2 => {
                yield_();
            }
            exit_code => return exit_code,
        }
    }
}

pub fn mutex_create() -> isize {
    sys_mutex_create()
}
pub fn mutex_lock(mutex_id: usize) {
    sys_mutex_lock(mutex_id);
}
pub fn mutex_unlock(mutex_id: usize) {
    sys_mutex_unlock(mutex_id);
}
pub fn semaphore_create(res_count: usize) -> isize {
    sys_semaphore_create(res_count)
}
pub fn semaphore_up(sem_id: usize) {
    sys_semaphore_plus(sem_id);
}
pub fn semaphore_down(sem_id: usize) {
    sys_semaphore_minus(sem_id);
}
pub fn condvar_create() -> isize {
    sys_condvar_create()
}
pub fn condvar_signal(condvar_id: usize) {
    sys_condvar_signal(condvar_id);
}
pub fn condvar_wait(condvar_id: usize, mutex_id: usize) {
    sys_condvar_wait(condvar_id, mutex_id);
}
pub fn get_resolution() -> (u32, u32) {
    let v = sys_get_gpures();
    ((v >> 32) as u32, (v & 0xffffffff) as u32 )
}
pub fn get_fbfd() -> isize {
    sys_get_fbfd()
}
pub fn get_inputevent() -> InputEvent {
    InputEvent::from(sys_input_event() as u64)
}