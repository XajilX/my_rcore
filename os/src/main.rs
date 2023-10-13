#![no_std]
#![no_main]
#![feature(panic_info_message)]
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

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn rust_main() -> ! {
    clr_bss();
    logging::init();
    trap::init();
    loader::load_apps();
    trap::enable_timer_int();
    timer::set_trig();
    task::run_first_task();
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
