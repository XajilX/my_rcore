use alloc::{vec, string::String};
use alloc::vec::Vec;
use bitflags::*;

use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};

use super::{address::{PhysPageNum, PPN_MASK, VirtPageNum, VirtAddr, PhysAddr}, frame_allocator::{FrameTracker, frame_alloc}};

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1<<0;
        const R = 1<<1;
        const W = 1<<2;
        const X = 1<<3;
        const U = 1<<4;
        const G = 1<<5;
        const A = 1<<6;
        const D = 1<<7;
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct PageTabEntry {
    pub bits: usize
}
impl PageTabEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self { bits: ppn.0 << 10 | (flags.bits as usize) }
    }
    pub fn zeros() -> Self { Self { bits: 0 } }
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & PPN_MASK).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

pub struct PageTab {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>
}
impl PageTab {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        Self {
            root_ppn: frame.ppn,
            frames: vec![frame]
        }
    }
    pub fn ins(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} already exists in page table, ppn = {:?}, flag = {:?}. ", vpn, pte.ppn(), pte.flags());
        *pte = PageTabEntry::new(ppn, PTEFlags::V | flags);
    }
    pub fn del(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} already invalid", vpn);
        *pte = PageTabEntry::zeros();
    }
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new()
        }
    }
    pub fn find(&self, vpn: VirtPageNum) -> Option<PageTabEntry> {
        self.find_pte(vpn).map(|pte| pte.clone())
    }
    pub fn trans_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        let ppn = self.find(va.vpn_floor()).map(|pte| pte.ppn())?;
        let offset = va.page_offset();
        Some(PhysAddr::from((ppn.0 << PAGE_SIZE_BITS) | offset))
    }
    pub fn trans_cstr(&self, ptr: *const u8) -> String {
        let mut string = String::new();
        let mut va = VirtAddr::from(ptr as usize);
        loop {
            let ch: u8 = *(self.trans_va(va).unwrap().get_mut());
            if ch == 0 {
                break;
            } else {
                string.push(ch as char);
                va.0 += 1;
            }
        }
        string
    }

    #[allow(unused)]
    pub fn trans_ref<T>(&self, ptr: *const T) -> &'static T {
        self.trans_va((ptr as usize).into()).unwrap().get_ref()
    }
    pub fn trans_mut<T>(&self, ptr: *mut T) -> &'static mut T {
        self.trans_va((ptr as usize).into()).unwrap().get_mut()
    }

    pub fn trans_bytes_buffer(&self, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
        let mut va_start = VirtAddr::from(ptr as usize);
        let va_end = VirtAddr(va_start.0 + len);
        let mut ret = Vec::new();
        while va_start < va_end {
            let vpn = va_start.vpn_floor();
            let ppn = self
                .find(vpn)
                .unwrap()
                .ppn();
            let va_page_end: VirtAddr = (VirtAddr::from(vpn).0 + PAGE_SIZE).into();
            if va_page_end >= va_end {
                ret.push(&mut ppn.get_bytes()[
                    va_start.page_offset()..va_end.page_offset()
                ]);
            } else {
                ret.push(&mut ppn.get_bytes()[
                    va_start.page_offset()..
                ])
            }
            va_start = va_page_end;
        }
        ret
    }
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTabEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut res: Option<&mut PageTabEntry> = None;
        for i in 0..3 {
            let pte = &mut ppn.get_ptes()[idxs[i]];
            if i == 2 {
                res = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                *pte = PageTabEntry::new(*frame, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        res
    }
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTabEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut res: Option<&mut PageTabEntry> = None;
        for idx in idxs {
            let pte = &mut ppn.get_ptes()[idx];
            if !pte.is_valid() {
                res = None;
                break;
            }
            ppn = pte.ppn();
            res = Some(pte);
        }
        res
    }
    pub fn get_atp_token(&self) -> usize {
        8usize << 60 |      //  mode SV39
        self.root_ppn.0
    }
}

pub struct UserBuffer {
    pub buffers: Vec<&'static mut [u8]>
}
impl UserBuffer {
    #[allow(unused)]
    pub fn from(buffers: Vec<&'static mut[u8]>) -> Self {
        Self { buffers }
    }

    pub fn len(&self) -> usize {
        self.buffers.iter()
            .fold(
                0, 
                |acc, arr| {
                    acc + arr.len()
                }
            )
    }
}

impl IntoIterator for UserBuffer {
    type Item = *mut u8;

    type IntoIter = UserBufferIterator;

    fn into_iter(self) -> Self::IntoIter {
        UserBufferIterator {
            buffers: self.buffers,
            curr_buf: 0,
            curr_idx: 0
        }
    }
}

pub struct UserBufferIterator {
    buffers: Vec<&'static mut[u8]>,
    curr_buf: usize,
    curr_idx: usize
}
impl Iterator for UserBufferIterator {
    type Item = *mut u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_buf >= self.buffers.len() {
            None
        } else {
            let r = &mut self.buffers[self.curr_buf][self.curr_idx] as *mut _;
            self.curr_idx += 1;
            if self.curr_idx >= self.buffers[self.curr_buf].len() {
                self.curr_idx = 0;
                self.curr_buf += 1;
            }
            Some(r)
        }
    }
}