#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]

#[macro_use]
pub mod console;
mod syscall;
mod lang_items;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main())
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main function");
}

fn clear_bss() {
    extern "C" {
        fn bss_start();
        fn bss_end();
    }
    for i in bss_start as usize..bss_end as usize {
        unsafe { (i as *mut u8).write_volatile(0); }
    }
}

use syscall::*;

pub fn write(fd: usize, buffer: &[u8]) -> isize {
    sys_write(fd, buffer)
}

pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
    unreachable!()
}
