use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapContext {
    pub reg: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    pub kern_satp: usize,
    pub kern_sp: usize,
    pub handler_entry: usize
}

impl TrapContext {
    pub fn app_init_context(sepc: usize, sp: usize, kern_satp: usize, kern_sp: usize, handler_entry: usize) -> Self {
        let mut sstatus_v = sstatus::read();
        sstatus_v.set_spp(SPP::User);
        let mut reg = [0usize; 32];
        reg[2] = sp;
        Self {
            reg,
            sstatus: sstatus_v,
            sepc,
            kern_satp,
            kern_sp,
            handler_entry
        }
    }
}