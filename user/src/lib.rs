#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
pub mod console;
mod syscall;
mod lang_items;


/// ------------------------------------------------------------
/// ------------------------------------------------------------
/// Linkage
/// ------------------------------------------------------------
/// ------------------------------------------------------------

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    unsafe {
        HEAP.lock().init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
    exit(main())
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main function");
}

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

pub fn exec(path: &str) -> isize {
    let path_ = CString::new(path).map_or_else(
        |_| CString::from_vec_with_nul(Vec::from(path)),
        |x| Ok(x)
    ).unwrap();
    sys_exec(&path_)
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
    let start = sys_get_time();
    while sys_get_time() < start + ms as isize {
        sys_yield();
    }
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
