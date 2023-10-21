use crate::trap::trap_ret;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    sreg: [usize; 12]
}
impl TaskContext {
    pub fn zeros() -> Self {
        Self {
            ra: 0,
            sp: 0,
            sreg: [0; 12]
        }
    }
    pub fn to_trap_ret(kern_sp: usize) -> Self {
        Self {
            ra: trap_ret as usize,
            sp: kern_sp,
            sreg: [0; 12]
        }
    }
}