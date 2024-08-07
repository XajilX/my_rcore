use core::{ops::Range, arch::asm};
use alloc::{vec::Vec, sync::Arc};

use elf::{ElfBytes, endian::AnyEndian, abi::{PT_LOAD, PF_W, PF_R, PF_X}};
use lazy_static::lazy_static;
use riscv::register::satp;

use crate::{config::{ADDR_TRAMPOLINE, MEM_END, MMIO, PAGE_SIZE}, sync::UThrCell};
use super::{address::{VirtAddr, PhysAddr, VirtPageNum}, pagetab::{PageTab, PTEFlags, PageTabEntry}, memarea::*};

extern "C" {
    fn stext(); fn etext();
    fn srodata(); fn erodata();
    fn sdata(); fn edata();
    fn sbss_stack(); fn ebss();
    fn ekernel();
    fn strampoline();
}

pub struct MemSet {
    pagetab: PageTab,
    areas: Vec<MapArea>
}
impl MemSet {
    pub fn new_empty() -> Self {
        Self {
            pagetab: PageTab::new(),
            areas: Vec::new()
        }
    }
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.ins(&mut self.pagetab);
        if let Some(data) = data {
            map_area.copy_data(&mut self.pagetab, data);
        }
        self.areas.push(map_area);
    }
    pub fn insert_framed_area(
        &mut self,
        va_range: Range<VirtAddr>,
        perm: MapPermission
    ) {
        self.push(MapArea::new(
            va_range,
            MapType::Framed,
            perm
        ), None);
    }
    fn map_trampoline(&mut self) {
        self.pagetab.ins(
            VirtAddr::from(ADDR_TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X
        );
    }
    pub fn new_kernel() -> Self {
        let mut memset = Self::new_empty();

        //  trampoline (Not collected by memarea)
        memset.map_trampoline();

        //  text
        memset.push(MapArea::new(
            (stext as usize).into()..(etext as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::X
        ), None);

        //  rodata
        memset.push(MapArea::new(
            (srodata as usize).into()..(erodata as usize).into(),
            MapType::Identical,
            MapPermission::R
        ), None);

        //  data
        memset.push(MapArea::new(
            (sdata as usize).into()..(edata as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W
        ), None);

        //  bss
        memset.push(MapArea::new(
            (sbss_stack as usize).into()..(ebss as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W
        ), None);

        //  physical mem
        memset.push(MapArea::new(
            (ekernel as usize).into()..(MEM_END).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W
        ), None);

        // MMIO
        for pair in MMIO {
            let va_start: VirtAddr = (*pair).0.into();
            let va_end: VirtAddr = ((*pair).0 + (*pair).1).into();
            memset.push(MapArea::new(
                va_start..va_end, 
                MapType::Identical,
                MapPermission::R | MapPermission::W
                ), None
            );
        }
        memset
    }
    //  Memset, user_sp, entry point
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memset = Self::new_empty();
        //  trampoline
        memset.map_trampoline();

        //  elf sections
        let mut elf_end_vpn = VirtPageNum(0);
        let elf = ElfBytes::<AnyEndian>::minimal_parse(elf_data)
            .expect("[kernel] parsing error encountered for elf.");
        for phdr in elf
            .segments()
            .unwrap()
            .iter()
            .filter(|phdr| phdr.p_type == PT_LOAD)
        {
            let va_start:VirtAddr = (phdr.p_vaddr as usize).into();
            let va_end:VirtAddr = ((phdr.p_vaddr + phdr.p_memsz) as usize).into();
            let map_perm = MapPermission::from_bits_truncate(
                (1u8 << 4) |    //  MapPermission::U
                (((phdr.p_flags & PF_R) >> 1) |
                 ((phdr.p_flags & PF_W) << 1) |
                 ((phdr.p_flags & PF_X) << 3)) as u8
            );
            let map_area = MapArea::new(
                va_start..va_end,
                MapType::Framed,
                map_perm
            );
            elf_end_vpn = va_end.vpn_ceil();
            memset.push(map_area, Some(elf.segment_data(&phdr)
                .expect("[kernel] parsing error encountered for elf."))
            );
        }

        let user_stack_bottom = VirtAddr::from(elf_end_vpn).0 + PAGE_SIZE;
        (
            memset,
            user_stack_bottom,          //  user_stack_bottom
            elf.ehdr.e_entry as usize   //  entry_point
        )
    }
    pub fn activate(&self) {
        let satp = self.pagetab.get_atp_token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }
    pub fn mem_recycle(&mut self) {
        while let Some(mut mem_area) = self.areas.pop() {
            mem_area.del(&mut self.pagetab);
        }
    }
    pub fn find(&self, vpn: VirtPageNum) -> Option<PageTabEntry> {
        self.pagetab.find(vpn)
    }
    pub fn get_atp_token(&self) -> usize {
        self.pagetab.get_atp_token()
    }
    pub fn del_area_by_start_vpn(&mut self, start_vpn: VirtPageNum) {
        if let Some((idx, area)) = self
            .areas
            .iter_mut()
            .enumerate()
            .find(|(_, area)| area.get_start_vpn() == start_vpn)
        {
            area.del(&mut self.pagetab);
            self.areas.swap_remove(idx);
        }
    }
}
impl Clone for MemSet {
    fn clone(&self) -> Self {
        let mut memset = Self::new_empty();
        memset.map_trampoline();
        for area in self.areas.iter() {
            let new_area = area.clone();
            memset.push(new_area, None);
            for vpn in area.get_vpn_range() {
                let src_ppn = self.find(vpn).unwrap().ppn();
                let dst_ppn = memset.find(vpn).unwrap().ppn();
                dst_ppn.get_bytes().copy_from_slice(&src_ppn.get_bytes());
            }
        }
        memset
    }
}

lazy_static! {
    pub static ref KERN_SPACE: Arc<UThrCell<MemSet>> = Arc::new(
        unsafe {
            UThrCell::new(MemSet::new_kernel())
        }
    );
}

pub fn kern_mem_init() {
    KERN_SPACE.get_refmut().activate();
}
