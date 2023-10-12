use core::{arch::asm, mem::size_of};
use crate::{trap::context::TrapContext, sbi::shutdown};

const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDR: usize = 0x80400000;
const APP_SIZE_LIM: usize = 0x20000;

const USER_STACK_SIZE: usize = 0x2000;
const KERN_STACK_SIZE: usize = 0x2000;

#[repr(align(4096))]
struct KernStack {
    data: [u8; KERN_STACK_SIZE]
}
impl KernStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERN_STACK_SIZE
    }
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
            cx_ptr.as_mut().unwrap()
        }
    }
}
static KERN_STACK: KernStack = KernStack {
    data: [0; KERN_STACK_SIZE]
};

#[repr(align(4096))]
struct UserStack {
    data: [u8; KERN_STACK_SIZE]
}
impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}
static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE]
};

struct AppMan {
    num_app: usize,
    curr_app: usize,
    app_start: [usize; MAX_APP_NUM + 1]
}

impl AppMan {
    fn print_app_info(&self) {
        println!("[kernel] {} apps in os. ", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} range: [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i+1]
            )
        }
    }
    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            panic!("Invalid app_id")
        }
        println!("[kernel] Loading app_{}", app_id);
        core::slice::from_raw_parts_mut(
            APP_BASE_ADDR as *mut u8, APP_SIZE_LIM
        ).fill(0);
        let app_len = self.app_start[app_id + 1] - self.app_start[app_id];
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            app_len
        );
        core::slice::from_raw_parts_mut(
            APP_BASE_ADDR as *mut u8, app_len
        ).copy_from_slice(app_src);
        asm!{"fence.i"};
    }
    pub fn get_curr_app(&self) -> usize { self.curr_app }
    pub fn next_app(&mut self) { self.curr_app += 1; }
    pub fn is_no_app(&self) -> bool { self.curr_app >= self.num_app }
    pub fn mem_range(&self, app_id: usize) -> (usize, usize) {
        (self.app_start[app_id], self.app_start[app_id + 1])
    }
}

use lazy_static::lazy_static;
use crate::uthr::UThrCell;
use core::slice::from_raw_parts;
lazy_static! {
    static ref APP_MAN: UThrCell<AppMan> = unsafe {
        extern "C" { fn _num_app(); }
        let num_app_ptr = _num_app as usize as * const usize;
        let num_app = num_app_ptr.read_volatile();
        let mut app_start = [0usize; MAX_APP_NUM + 1];
        let app_start_raw = from_raw_parts(
            num_app_ptr.add(1), num_app + 1
        );
        app_start[..=num_app].copy_from_slice(app_start_raw);
        UThrCell::new(AppMan {
            num_app,
            curr_app: 0,
            app_start
        })
    };
}

pub fn init() {
    APP_MAN.get_refmut().print_app_info();
}

pub fn run_app() -> ! {
    let mut app_man = APP_MAN.get_refmut();
    if app_man.is_no_app() {
        println!("[kernel] All apps are executed, shutdown. ");
        shutdown();
    }
    let curr_app = app_man.get_curr_app();
    unsafe { app_man.load_app(curr_app); }
    app_man.next_app();
    drop(app_man);
    extern "C" {
        fn __restore(cx_addr: usize);
    }
    unsafe {
        __restore(KERN_STACK.push_context(TrapContext::app_init_context(
            APP_BASE_ADDR,
            USER_STACK.get_sp()
        )) as *const _ as usize);
    }
    unreachable!()
}

pub fn mem_range_curr_app() -> (usize, usize) {
    let app_man = APP_MAN.get_refmut();
    app_man.mem_range(app_man.get_curr_app())
}