use alloc::vec::Vec;
use lazy_static::lazy_static;
use log::debug;
use crate::{mm::{address::PhysAddr, frame_allocator::frame_dealloc, memset::KERN_SPACE, pagetab::PageTab}, sync::{CondVar, UThrCell}, task::processor::schedule, DEV_NONBLOCKING_ACCESS};
use virtio_drivers::{BlkResp, Hal, RespStatus, VirtIOBlk, VirtIOHeader};
use easyfs::BlockDevice;

use crate::mm::frame_allocator::{frame_alloc, FrameTracker};

const VIRTIO0: usize = 0x10001000;

pub struct VirtIOBlock {
    virtio_blk: UThrCell<VirtIOBlk<'static, VirtIOHal>>,
    condvars: Vec<CondVar>
}

unsafe impl Sync for VirtIOBlock {}
unsafe impl Send for VirtIOBlock {}

impl VirtIOBlock {
    pub fn new() -> Self {
        let virtio_blk = unsafe {
            UThrCell::new(VirtIOBlk::<VirtIOHal>::new(
                &mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap())
        };
        let mut condvars = Vec::<CondVar>::new();
        let channels = virtio_blk.get_refmut().virt_queue_size();
        for _ in 0..channels {
            condvars.push(CondVar::new());
        }
        Self {
            virtio_blk,
            condvars 
        }
    }
}

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let flag_nb = *DEV_NONBLOCKING_ACCESS.get_refmut();
        if flag_nb {
            let mut resp = BlkResp::default();
            let task_cx = self.virtio_blk.then(|blk| {
                let token = unsafe {
                    blk.read_block_nb(block_id, buf, &mut resp).unwrap()
                };
                self.condvars[token as usize].wait_without_schd()
            });
            schedule(task_cx);
            assert_eq!(
                resp.status(), RespStatus::Ok,
                "Error when reading VirtIOBlk"
            )
        } else {
            self.virtio_blk.get_refmut()
                .read_block(block_id, buf)
                .expect("Error when reading VirtIOBlk");
        }
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let flag_nb = *DEV_NONBLOCKING_ACCESS.get_refmut();
        if flag_nb {
            let mut resp = BlkResp::default();
            let task_cx = self.virtio_blk.then(|blk| {
                let token = unsafe {
                    blk.write_block_nb(block_id, buf, &mut resp).unwrap()
                };
                self.condvars[token as usize].wait_without_schd()
            });
            schedule(task_cx);
            assert_eq!(
                resp.status(), RespStatus::Ok,
                "Error when writing VirtIOBlk"
            )
        } else {
            self.virtio_blk.get_refmut()
                .write_block(block_id, buf)
                .expect("Error when writing VirtIOBlk");
        }
    }

    fn handle_irq(&self) {
        debug!("Block Device Handling IRQ");
        self.virtio_blk.then(|blk| {
            while let Ok(token) = blk.pop_used() {
                self.condvars[token as usize].signal();
            }
        });
    }
}

lazy_static! {
    static ref QUEUE_FRAMES: UThrCell<Vec<FrameTracker>> = unsafe { UThrCell::new(Vec::new()) };
}

pub struct VirtIOHal;
impl Hal for VirtIOHal {
    #![allow(unused)]
    fn dma_alloc(pages: usize) -> virtio_drivers::PhysAddr {
        let frame = frame_alloc().unwrap();
        let mut ppn_base = frame.ppn;
        QUEUE_FRAMES.get_refmut().push(frame);

        for i in 1..pages {
            let frame = frame_alloc().unwrap();
            assert_eq!(frame.ppn.0, ppn_base.0 + i);
            QUEUE_FRAMES.get_refmut().push(frame);
        }
        let pa = PhysAddr::from(ppn_base);
        pa.0
    }

    fn phys_to_virt(paddr: virtio_drivers::PhysAddr) -> virtio_drivers::VirtAddr {
        paddr
    }
    
    fn virt_to_phys(vaddr: virtio_drivers::VirtAddr) -> virtio_drivers::PhysAddr {
        PageTab::from_token(KERN_SPACE.get_refmut().get_atp_token())
            .trans_va(vaddr.into()).unwrap().0
    }
    
    fn dma_dealloc(paddr: virtio_drivers::PhysAddr, pages: usize) -> i32 {
        let mut ppn_base = PhysAddr::from(paddr).ppn_floor();
        for _ in 0..pages {
            frame_dealloc(ppn_base);
            ppn_base.0 += 1;
        }
        0
    }
}