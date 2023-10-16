use core::{arch::asm, mem::size_of};
use log::warn;

use crate::trap::context::TrapContext;

use crate::config::*;

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct KernStack {
    data: [u8; KERN_STACK_SIZE]
}
impl KernStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERN_STACK_SIZE
    }
    pub fn push_context(&self, cx: TrapContext) -> usize {
        let cx_ptr = (self.get_sp() - size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        cx_ptr as usize
    }
}
static KERN_STACK: [KernStack; MAX_APP_NUM] = [KernStack {
    data: [0; KERN_STACK_SIZE]
}; MAX_APP_NUM];

#[repr(align(4096))]
#[derive(Clone,Copy)]
struct UserStack {
    data: [u8; USER_STACK_SIZE]
}
impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}
static USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE]
}; MAX_APP_NUM];

fn get_id_base(app_id: usize) -> usize {
    APP_BASE_ADDR + app_id * APP_SIZE_LIM
}

pub fn get_num_app() -> usize {
    extern "C" { fn _num_app(); }
    unsafe {
        (_num_app as *const usize).read_volatile()
    }
}

pub fn load_apps() {
    extern "C" { fn _num_app(); }
    let app_ptr = _num_app as *const usize;
    let num_app = unsafe {app_ptr.read_volatile()};
    let app_start = unsafe {
        core::slice::from_raw_parts(app_ptr.add(1), num_app + 1)
    };
    unsafe { asm!("fence.i"); }
    for i in 0..num_app {
        let base = get_id_base(i);
        for addr in base..base + APP_SIZE_LIM {
            unsafe { (addr as *mut u8).write_volatile(0); }
        }
        let app_len = app_start[i + 1] - app_start[i];
        let src = unsafe {
            core::slice::from_raw_parts(
                app_start[i] as *const u8,
                app_len
            )
        };
        unsafe {
            core::slice::from_raw_parts_mut(
                base as *mut u8,
                app_len
            ).copy_from_slice(src);
        }
        warn!(
            "[kernel] App {} load in position [0x{:016x}, 0x{:016x})",
            i,
            base,
            base + app_len
        );
    }
}

pub fn init_app_cx(app_id: usize) -> usize {
    KERN_STACK[app_id].push_context(TrapContext::app_init_context(
        get_id_base(app_id),
        USER_STACK[app_id].get_sp()
    ))
}