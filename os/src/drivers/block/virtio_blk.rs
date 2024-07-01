use alloc::vec::Vec;
use easyfs::BlockDev;
use log::debug;
use crate::{drivers::{Device, VirtioHal}, sync::{CondVar, UThrCell}, task::processor::schedule, DEV_NONBLOCKING_ACCESS};
use virtio_drivers::{device::blk::{BlkReq, BlkResp, VirtIOBlk}, transport::mmio::MmioTransport};


use super::BlockDevice;


pub struct VirtioBlock {
    virtio_blk: UThrCell<VirtIOBlk<VirtioHal, MmioTransport>>,
    condvars: Vec<CondVar>
}

impl VirtioBlock {
    pub fn new(transport: MmioTransport) -> Self {
        let virtio_blk = unsafe {
            UThrCell::new(VirtIOBlk::<VirtioHal, MmioTransport>::new(
                transport
            ).unwrap())
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

impl BlockDev for VirtioBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let flag_nb = *DEV_NONBLOCKING_ACCESS.get_refmut();
        if flag_nb {
            let mut resp = BlkResp::default();
            let mut req = BlkReq::default();
            let mut token = 0u16;
            let task_cx = self.virtio_blk.then(|blk| {
                token = unsafe {
                    blk.read_blocks_nb(block_id, &mut req, buf, &mut resp).unwrap()
                };
                self.condvars[token as usize].wait_without_schd()
            });
            schedule(task_cx);
            unsafe {
                self.virtio_blk.get_refmut().complete_read_blocks(token, &mut req, buf, &mut resp)
                    .expect("Error when reading VirtIOBlk");
            }
        } else {
            self.virtio_blk.get_refmut()
                .read_blocks(block_id, buf)
                .expect("Error when reading VirtIOBlk");
        }
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let flag_nb = *DEV_NONBLOCKING_ACCESS.get_refmut();
        if flag_nb {
            let mut resp = BlkResp::default();
            let mut req = BlkReq::default();
            let mut token = 0u16;
            let task_cx = self.virtio_blk.then(|blk| {
                token = unsafe {
                    blk.write_blocks_nb(block_id, &mut req, buf, &mut resp).unwrap()
                };
                self.condvars[token as usize].wait_without_schd()
            });
            schedule(task_cx);
            unsafe {
                self.virtio_blk.get_refmut().complete_write_blocks(token, &mut req, buf, &mut resp)
                    .expect("Error when writing VirtIOBlk");
            }
        } else {
            self.virtio_blk.get_refmut()
                .write_blocks(block_id, buf)
                .expect("Error when writing VirtIOBlk");
        }
    }

}

impl Device for VirtioBlock {
    fn handle_irq(&self) {
        debug!("Block Device Handling IRQ");
        self.virtio_blk.then(|blk| {
            if let Some(token) = blk.peek_used() {
                self.condvars[token as usize].signal();
            }
        });
    }
}

impl BlockDevice for VirtioBlock {}

