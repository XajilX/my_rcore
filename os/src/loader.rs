use core::ops::Range;
use log::debug;

use crate::config::*;

pub fn kern_stack_range(app_id: usize) -> Range<usize> {
    let top = ADDR_TRAMPOLINE - app_id * (KERN_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERN_STACK_SIZE;
    bottom..top
}

pub fn get_num_app() -> usize {
    extern "C" { fn _num_app(); }
    unsafe {
        (_num_app as *const usize).read_volatile()
    }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    debug!("get data in appid {}", app_id);
    extern "C" { fn _num_app(); }
    let app_ptr = _num_app as *const usize;
    let num_app = unsafe {app_ptr.read_volatile()};
    let app_start = unsafe {
        core::slice::from_raw_parts(app_ptr.add(1), num_app + 1)
    };
    assert!(app_id < num_app, "Invalid app_id {app_id}! ");
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id]
        )
    }
}
