use core::arch::{global_asm, asm};
use context::TrapContext;
use riscv::register::{scause::{self, Exception, Interrupt, Trap}, sie, sscratch, sstatus, stval, stvec, utvec::TrapMode};

use crate::{config::ADDR_TRAMPOLINE, syscall::syscall, task::{exit_curr_task, processor::{curr_atp_token, curr_proc, curr_trap_cx, curr_trap_va}, send_signal_curr_proc, signal::SignalFlags, suspend_curr_task}, timer::{set_trig, sleep::check_sleeptimer}};

pub mod context;
global_asm!(include_str!("trap.S"));

pub fn init() {
    set_kern_trap_entry();
}

pub fn enable_timer_int() {
    unsafe { sie::set_stimer(); }
}

fn enable_supervisor_interrupt() {
    unsafe { sstatus::set_sie(); }
}

fn disable_supervisor_interrupt() {
    unsafe { sstatus::clear_sie(); }
}

fn set_kern_trap_entry() {
    extern "C" {
        fn __trap_entry_k();
        fn __trap_entry();
    }
    let entry_k_va = __trap_entry_k as usize - __trap_entry as usize + ADDR_TRAMPOLINE;
    unsafe {
        stvec::write(entry_k_va, TrapMode::Direct);
        sscratch::write(trap_from_kern as usize);
    }
}
fn set_user_trap_entry() {
    unsafe {
        stvec::write(ADDR_TRAMPOLINE, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_from_kern(_trap_cx: &TrapContext) {
    let scause_v = scause::read();
    let stval_v = stval::read();
    match scause_v.cause() {
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            crate::drivers::irq_handler();
        },
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_trig();
            check_sleeptimer();
        },
        _ => {
            panic!(
                "Unsupported trap from kernel {:?}, stval = {:#x}",
                scause_v.cause(),
                stval_v
            );
        }
    }
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
            enable_supervisor_interrupt();
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
            check_sleeptimer();
            suspend_curr_task();
        },
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            crate::drivers::irq_handler();
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) |
        Trap::Exception(Exception::LoadFault) |
        Trap::Exception(Exception::LoadPageFault) |
        Trap::Exception(Exception::InstructionFault) |
        Trap::Exception(Exception::InstructionPageFault) => {
            /*
            println!("[kernel] {:?} in app, bad addr = {:#x}, kernel execution. ", scause_v.cause(), stval_v);
            exit_curr_proc(-2)
            */
            send_signal_curr_proc(SignalFlags::SIGSEGV)
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            /*
            println!("[kernel] IllegalInstruction in app, kernel execution. ");
            exit_curr_proc(-3)
            */
            send_signal_curr_proc(SignalFlags::SIGILL)
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}",
                scause_v.cause(),
                stval_v
            )
        }
    };
    if let Some((errno, msg)) = curr_proc().get_mutpart()
        .signals.check_error()
    {
        println!("[kernel] {}", msg);
        exit_curr_task(errno);
    }
    trap_ret();
}

#[no_mangle]
pub fn trap_ret() -> ! {
    disable_supervisor_interrupt();
    set_user_trap_entry();
    let trap_cx_ptr = curr_trap_va();
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