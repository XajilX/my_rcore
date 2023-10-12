use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
pub struct TrapContext {
    pub reg: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize
}

impl TrapContext {
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus_v = sstatus::read();
        sstatus_v.set_spp(SPP::User);
        let mut reg = [0usize; 32];
        reg[2] = sp;
        Self {
            reg,
            sstatus: sstatus_v,
            sepc: entry
        }
    }
}