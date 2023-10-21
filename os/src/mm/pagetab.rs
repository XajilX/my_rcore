use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;
use log::debug;

use crate::config::PAGE_SIZE;

use super::{address::{PhysPageNum, PPN_MASK, VirtPageNum, VirtAddr}, frame_allocator::{FrameTracker, frame_alloc}};

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
        debug!("new pagetab at root {:#x}", frame.0);
        Self {
            root_ppn: frame.ppn,
            frames: vec![frame]
        }
    }
    pub fn ins(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} already exists in page table. ", vpn);
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
    pub fn trans_bytes_buffer(&self, ptr: *const u8, len: usize) -> Vec<&'static [u8]> {
        let mut va_start = VirtAddr::from(ptr as usize);
        let va_end = VirtAddr(va_start.0 + len);
        let mut ret: Vec<&[u8]> = Vec::new();
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