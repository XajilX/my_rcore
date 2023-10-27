use core::ops::Range;
use alloc::collections::BTreeMap;
use bitflags::bitflags;

use crate::config::PAGE_SIZE;
use super::{address::{VirtPageNum, VirtPageRange, VirtAddr, PhysPageNum}, frame_allocator::{FrameTracker, frame_alloc}, pagetab::{PageTab, PTEFlags}};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MapType {
    Identical,
    Framed
}

bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

pub struct MapArea {
    range: VirtPageRange,
    frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission
}
impl MapArea {
    pub fn new(
        va_range: Range<VirtAddr>,
        map_type: MapType,
        map_perm: MapPermission
    ) -> Self {
        let range = VirtPageRange::from_va_range(va_range);
        Self {
            range,
            frames: BTreeMap::new(),
            map_type,
            map_perm
        }
    }
    fn ins_one(&mut self, pagetab: &mut PageTab, vpn: VirtPageNum) {
        let ppn: PhysPageNum;
        match self.map_type {
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0);
            }
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.frames.insert(vpn, frame);
            }
        }
        let flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        pagetab.ins(vpn, ppn, flags)
    }
    fn del_one(&mut self, pagetab: &mut PageTab, vpn: VirtPageNum) {
        if let MapType::Framed = self.map_type {
            self.frames.remove(&vpn);
        }
        pagetab.del(vpn)
    }
    pub fn ins(&mut self, pagetab: &mut PageTab) {
        for vpn in self.range {
            self.ins_one(pagetab, vpn);
        }
    }
    pub fn del(&mut self, pagetab: &mut PageTab) {
        for vpn in self.range {
            self.del_one(pagetab, vpn);
        }
    }
    pub fn copy_data(&mut self, pagetab: &mut PageTab, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut curr_vpn = self.range.start;
        let len = data.len();
        loop {
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut pagetab.find(curr_vpn)
                .unwrap().ppn().get_bytes()[..src.len()];
            dst.copy_from_slice(src);
            start += src.len();
            if start >= len {
                break
            }
            curr_vpn.0 += 1;
        }
    }
    pub fn get_vpn_range(&self) -> VirtPageRange {
        self.range
    }
    pub fn get_start_vpn(&self) -> VirtPageNum {
        self.range.start
    }
}
impl Clone for MapArea {
    fn clone(&self) -> Self {
        Self {
            range: self.range.clone(),
            frames: BTreeMap::new(),
            map_type: self.map_type,
            map_perm: self.map_perm
        }
    }
}