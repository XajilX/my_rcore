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
    pub fn to_restore(kern_ptr: usize) -> Self {
        extern "C" { fn __restore(); }
        Self {
            ra: __restore as usize,
            sp: kern_ptr,
            sreg: [0; 12]
        }
    }
}