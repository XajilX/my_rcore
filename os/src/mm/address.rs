use core::ops::Range;
use core::fmt::{self, Debug, Formatter};

use crate::config::{PAGE_SIZE_BITS, PAGE_SIZE};

use super::pagetab::PageTabEntry;

//  SV39
pub const PA_WIDTH: usize = 56;
pub const PPN_WIDTH: usize = PA_WIDTH - PAGE_SIZE_BITS;
pub const PA_MASK: usize = (1 << PA_WIDTH) - 1;
pub const PPN_MASK: usize = (1 << PPN_WIDTH) - 1;

pub const VA_WIDTH: usize = 39;
pub const VPN_WIDTH: usize = VA_WIDTH - PAGE_SIZE_BITS;
pub const VA_MASK: usize = (1 << VA_WIDTH) - 1;
pub const VPN_MASK: usize = (1 << VPN_WIDTH) - 1;

pub const PAGE_OFFSET_MASK: usize = PAGE_SIZE - 1;

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub struct PhysAddr(pub usize);

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub struct VirtAddr(pub usize);

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub struct PhysPageNum(pub usize);

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub struct VirtPageNum(pub usize);

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}
impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}
impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
    }
}

//  range from [l, r)
#[derive(Clone, Copy)]
pub struct VirtPageRange {
    pub start: VirtPageNum,
    pub end: VirtPageNum,
}
impl Iterator for VirtPageRange {
    type Item = VirtPageNum;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else {
            let ret = self.start;
            self.start.0 += 1;
            Some(ret)
        }
    }
}
impl From<Range<usize>> for VirtPageRange {
    fn from(value: Range<usize>) -> Self {
        Self {
            start: VirtPageNum(value.start),
            end: VirtPageNum(value.end)
        }
    }
}
impl VirtPageRange {
    pub fn from_va_range(va_range: Range<VirtAddr>) -> Self {
        Self { start: va_range.start.vpn_floor(), end: va_range.end.vpn_ceil() }
    }
}


impl PhysAddr {
    pub fn page_offset(&self) -> usize { self.0 & PAGE_OFFSET_MASK }
    pub fn ppn_floor(&self) -> PhysPageNum { (self.0 >> PAGE_SIZE_BITS).into() }
    pub fn ppn_ceil(&self) -> PhysPageNum { ((self.0 + PAGE_OFFSET_MASK) >> PAGE_SIZE_BITS).into() }
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
}
impl VirtAddr {
    pub fn page_offset(&self) -> usize { self.0 & PAGE_OFFSET_MASK }
    pub fn vpn_floor(&self) -> VirtPageNum { (self.0 >> PAGE_SIZE_BITS).into() }
    pub fn vpn_ceil(&self) -> VirtPageNum { ((self.0 + PAGE_OFFSET_MASK) >> PAGE_SIZE_BITS).into() }
}
impl PhysPageNum {
    pub fn get_ptes(&self) -> &'static mut [PageTabEntry] {
        let pa: PhysAddr = (*self).into();
        unsafe {
            core::slice::from_raw_parts_mut(
                pa.0 as *mut PageTabEntry,
                PAGE_SIZE
            )
        }
    }
    pub fn get_bytes(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        unsafe {
            core::slice::from_raw_parts_mut(
                pa.0 as *mut u8,
                PAGE_SIZE
            )
        }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        pa.get_mut()
    }
}
impl VirtPageNum {
    pub fn indexes(&self) -> [usize; 3] {
        [
            (self.0 >> 18) & 0x1ff,
            (self.0 >> 9) & 0x1ff,
            self.0 & 0x1ff
        ]
    }
}



impl From<usize> for PhysAddr {
    fn from(value: usize) -> Self { Self(value & PA_MASK) }
}
impl From<PhysAddr> for usize {
    fn from(value: PhysAddr) -> Self { value.0 }
}
impl From<PhysAddr> for PhysPageNum {
    fn from(value: PhysAddr) -> Self {
        assert_eq!(value.page_offset(), 0);
        value.ppn_floor()
    }
}
impl From<PhysPageNum> for PhysAddr {
    fn from(value: PhysPageNum) -> Self { Self(value.0 << PAGE_SIZE_BITS) }
}
impl From<usize> for PhysPageNum {
    fn from(value: usize) -> Self { Self(value & PPN_MASK) }
}
impl From<PhysPageNum> for usize {
    fn from(value: PhysPageNum) -> Self { value.0 }
}



impl From<usize> for VirtAddr {
    fn from(value: usize) -> Self { Self(value & VA_MASK) }
}
impl From<VirtAddr> for usize {
    fn from(value: VirtAddr) -> Self { value.0 }
}
impl From<VirtAddr> for VirtPageNum {
    fn from(value: VirtAddr) -> Self {
        assert_eq!(value.page_offset(), 0);
        Self(value.0 >> PAGE_SIZE_BITS)
    }
}
impl From<VirtPageNum> for VirtAddr {
    fn from(value: VirtPageNum) -> Self { Self(value.0 << PAGE_SIZE_BITS) }
}
impl From<usize> for VirtPageNum {
    fn from(value: usize) -> Self { Self(value & VPN_MASK) }
}
impl From<VirtPageNum> for usize {
    fn from(value: VirtPageNum) -> Self { value.0 }
}