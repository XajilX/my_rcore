mod block;
mod plic;
mod uart;

pub use block::BLOCK_DEV;
use log::debug;
pub use uart::SERIAL_DEV;
use plic::{IntrTargetPriority, Plic};

use crate::config::VIRT_PLIC;


pub fn device_init() {
    use riscv::register::sie;
    let mut plic = unsafe { Plic::new(VIRT_PLIC) };
    let hart_id = 0usize;
    let smod = IntrTargetPriority::Supervisor;
    let mmod = IntrTargetPriority::Machine;
    plic.set_threshold(hart_id, smod, 0);
    plic.set_threshold(hart_id, mmod, 1);
    // 5: keyboard, 6: mouse, 1: block dev, 10: uart
    for intr_src_id in [5usize, 6, 1, 10] {
        plic.enable(hart_id, smod, intr_src_id);
        plic.set_priority(intr_src_id, 1);
    }
    unsafe { sie::set_sext(); }
}

pub fn irq_handler() {
    debug!("IRQ Handler");
    let mut plic = unsafe { Plic::new(VIRT_PLIC) };
    let intr_src_id = plic.claim(0, IntrTargetPriority::Supervisor);
    match intr_src_id {
        1 => BLOCK_DEV.handle_irq(),
        10 => SERIAL_DEV.handle_irq(),
        _ => panic!("unsupported IRQ {}", intr_src_id)
    }
    plic.complete(0, IntrTargetPriority::Supervisor, intr_src_id);
}