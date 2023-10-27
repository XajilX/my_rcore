#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
extern crate alloc;

#[macro_use]
mod console;
mod lang_items;
mod stack_trace;
mod sbi;
mod logging;
mod loader;
mod task;
mod uthr;
mod trap;
mod syscall;
mod config;
mod timer;
mod mm;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn rust_main() -> ! {
    clr_bss();
    println!("[kernel] Hello, world!");
    logging::init();
    println!("[kernel] start memory initialize");
    mm::init();
    println!("[kernel] memory space initialized successfully");
    mm::remap_test();
    task::add_initproc();
    println!("[kernel] initproc added");
    trap::init();
    println!("[kernel] trap entry set");
    trap::enable_timer_int();
    timer::set_trig();
    println!("[kernel] timer interrupt enabled");
    loader::list_apps();
    task::processor::run_proc();
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
