use core::arch::global_asm;
use riscv::register::{stvec, utvec::TrapMode, scause::{self, Trap, Exception}, stval};

use crate::{syscall::syscall, batch::run_app};

use self::context::TrapContext;
pub mod context;
global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" { fn __trap_entry(); }
    unsafe {
        stvec::write(__trap_entry as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause_v = scause::read();
    let stval_v = stval::read();
    match scause_v.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.reg[10] = syscall(
                cx.reg[17],
                [cx.reg[10], cx.reg[11], cx.reg[12]]
            ) as usize;
            cx
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in app, kernel execution. ");
            run_app()
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in app, kernel execution. ");
            run_app()
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}",
                scause_v.cause(),
                stval_v
            )
        }
    }
}