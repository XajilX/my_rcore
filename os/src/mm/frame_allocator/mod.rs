mod stack_frame_alloc;
use core::{ops::Deref, fmt::{Debug, Formatter, self}};

use lazy_static::lazy_static;

use crate::{sync::UThrCell, config::MEM_END, mm::address::PhysAddr};

use super::address::PhysPageNum;
use self::stack_frame_alloc::StackFrameAlloc;

pub trait FrameAlloc {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}
pub struct FrameTracker {
    pub ppn: PhysPageNum
}
impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        let bytes = ppn.get_bytes();
        for i in bytes {
            *i = 0;
        }
        Self { ppn }
    }
}
impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}
impl Deref for FrameTracker {
    type Target = PhysPageNum;
    fn deref(&self) -> &Self::Target {
        &self.ppn
    }
}

type FrameAllocImpl = StackFrameAlloc;

lazy_static! {
    pub static ref FRAME_ALLOC: UThrCell<FrameAllocImpl> = unsafe {
        UThrCell::new(FrameAllocImpl::new())
    };
}

pub fn init_frame_alloc() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOC.get_refmut().init(
        PhysAddr::from(ekernel as usize).ppn_ceil(),
        PhysAddr::from(MEM_END).ppn_floor()
    )
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOC.get_refmut().alloc()
        .map(|ppn| FrameTracker::new(ppn))
}
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOC.get_refmut().dealloc(ppn);
}