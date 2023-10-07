#![no_std]
#![no_main]
#![feature(panic_info_message)]
mod lang_items;
mod sbi;
#[macro_use]
mod console;
mod logging;
use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clr_bss();
    warn!("Hello world!");
    panic!("Shutdown! ");
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
