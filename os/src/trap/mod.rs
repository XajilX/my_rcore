use core::arch::{global_asm, asm};
use riscv::register::{stvec, utvec::TrapMode, scause::{self, Trap, Exception, Interrupt}, stval, sie};

use crate::{syscall::syscall, timer::set_trig, task::{suspend_curr_task, processor::{curr_trap_cx, curr_atp_token}, exit_curr_task}, config::{ADDR_TRAMPOLINE, ADDR_TRAPCONTEXT}};

pub mod context;
global_asm!(include_str!("trap.S"));

pub fn init() {
    set_user_trap_entry();
}

pub fn enable_timer_int() {
    unsafe { sie::set_stimer(); }
}

fn set_kern_trap_entry() {
    unsafe {
        stvec::write(trap_from_kern as usize, TrapMode::Direct);
    }
}
fn set_user_trap_entry() {
    unsafe {
        stvec::write(ADDR_TRAMPOLINE, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_from_kern() -> ! {
    let scause_v = scause::read();
    panic!("Trap from kernel caused by {:?}, shutdown", scause_v.cause());
}

#[no_mangle]
pub fn trap_handler() -> ! {
    set_kern_trap_entry();
    let scause_v = scause::read();
    let stval_v = stval::read();
    match scause_v.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            let mut cx = curr_trap_cx();
            cx.sepc += 4;
            let ret = syscall(
                cx.reg[17],
                [cx.reg[10], cx.reg[11], cx.reg[12]]
            ) as usize;
            //  cx may change when calling sys_exec
            cx = curr_trap_cx();
            cx.reg[10] = ret;
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_trig();
            suspend_curr_task()
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) |
        Trap::Exception(Exception::LoadFault) |
        Trap::Exception(Exception::LoadPageFault) |
        Trap::Exception(Exception::InstructionFault) |
        Trap::Exception(Exception::InstructionPageFault) => {
            println!("[kernel] {:?} in app, bad addr = {:#x}, kernel execution. ", scause_v.cause(), stval_v);
            exit_curr_task(-2)
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in app, kernel execution. ");
            exit_curr_task(-3)
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}",
                scause_v.cause(),
                stval_v
            )
        }
    };
    trap_ret();
}

#[no_mangle]
pub fn trap_ret() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = ADDR_TRAPCONTEXT;
    let user_satp = curr_atp_token();
    extern "C" {
        fn __trap_entry();
        fn __restore();
    }
    let res_va = __restore as usize - __trap_entry as usize + ADDR_TRAMPOLINE;
    unsafe { asm!(
        "fence.i",
        "jr {res_va}",
        res_va = in(reg) res_va,
        in("a0") trap_cx_ptr,
        in("a1") user_satp,
        options(noreturn)
    )}
}