use core::{arch::asm, ptr};

use log::warn;

pub unsafe fn print_stack_trace() {
    let mut fp: *const usize;
    unsafe { asm!("mv {}, fp", out(reg) fp); }
    warn!("=== STACK TRACE START ===");
    while fp != ptr::null() {
        let saved_ra = *fp.sub(1);
        let saved_fp = *fp.sub(2);

        println!("0x{:016x}, fp = 0x{:016x}", saved_ra, saved_fp);
        fp = saved_fp as *const usize;
    }
    warn!("=== STACK TRACE END ===")
}