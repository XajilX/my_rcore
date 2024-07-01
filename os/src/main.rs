#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(trait_upcasting)]
extern crate alloc;

#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod logging;
mod task;
mod sync;
mod trap;
mod syscall;
mod config;
mod timer;
mod mm;
mod drivers;
mod fs;

use core::arch::global_asm;

use lazy_static::lazy_static;
use sync::UThrCell;
global_asm!(include_str!("entry.asm"));

lazy_static! {
    pub static ref DEV_NONBLOCKING_ACCESS: UThrCell<bool> = {
        unsafe { UThrCell::new(false) }
    };
}

#[no_mangle]
pub fn rust_main() -> ! {
    clr_bss();
    logging::init();
    mm::init();
    drivers::device_init();
    println!("[kernel] Devices init success");
    trap::init();
    println!("[kernel] trap entry set");
    trap::enable_timer_int();
    timer::set_trig();
    println!("[kernel] timer interrupt enabled");
    fs::list_apps();
    task::add_initproc();
    *DEV_NONBLOCKING_ACCESS.get_refmut() = true;
    task::processor::run_task();
    unreachable!()
}

fn clr_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    for a in sbss as usize..ebss as usize {
        unsafe { (a as *mut u8).write_volatile(0) }
    }
}
