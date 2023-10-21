use alloc::vec::Vec;
use crate::mm::address::PhysPageNum;
use super::*;
pub struct StackFrameAlloc {
    current: usize,
    end: usize,
    recycled: Vec<usize>
}

impl FrameAlloc for StackFrameAlloc {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new()
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else {
            if self.current == self.end {
                None
            } else {
                self.current += 1;
                Some((self.current - 1).into())
            }
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        if ppn >= self.current || self.recycled
            .iter()
            .find(|&v| {*v == ppn})
            .is_some() {
                panic!("[kernel] Physical Page {:#x} de-allocate before allocated. ", ppn);
        }
        self.recycled.push(ppn);
    }
}

impl StackFrameAlloc {
    pub fn init(&mut self, l: PhysPageNum, r:PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}
