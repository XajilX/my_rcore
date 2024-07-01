use core::mem::size_of;

use alloc::sync::Arc;
use spin::Mutex;

use crate::{BlockDev, bitmap::{Bitmap, BLOCK_BITS}, BLOCK_SIZE, layout::{DiskInode, SuperBlock, DiskInodeType}, cache_man::{get_block_cache, sync_block_cache}, vfs::VirtInode};

pub struct EzFileSys {
    pub block_dev: Arc<dyn BlockDev>,
    pub inode_bitmap: Bitmap,
    pub data_bitmap: Bitmap,
    inode_start: u32,
    data_start: u32
}

type DataBlock = [u8; BLOCK_SIZE];
impl EzFileSys {
    pub fn new(
        block_dev: Arc<dyn BlockDev>,
        total_blocks: u32,
        inode_bitmap_blocks: u32
    ) -> Arc<Mutex<Self>> {
        let inode_bitmap = Bitmap::new(1, inode_bitmap_blocks as usize);
        let inode_num = inode_bitmap_blocks * BLOCK_BITS as u32;
        let inode_blocks = ((inode_num as usize * size_of::<DiskInode>() + BLOCK_SIZE - 1) / BLOCK_SIZE) as u32;
        let inode_total = inode_bitmap_blocks + inode_blocks;
        let data_total = total_blocks - inode_total - 1;
        let data_bitmap_blocks = (data_total + 4096) / 4097;
        let data_blocks = data_total - data_bitmap_blocks;
        let data_bitmap = Bitmap::new(inode_total as usize + 1, data_bitmap_blocks as usize);
        let mut efs = Self {
            block_dev: Arc::clone(&block_dev),
            inode_bitmap,
            data_bitmap,
            inode_start: 1 + inode_bitmap_blocks,
            data_start: 1 + inode_total + data_bitmap_blocks
        };
        // init superblock
        get_block_cache(0, Arc::clone(&block_dev))
            .lock()
            .modify(0, |super_blk: &mut SuperBlock| {
                super_blk.init(
                    total_blocks,
                    inode_bitmap_blocks,
                    inode_blocks,
                    data_bitmap_blocks,
                    data_blocks
                );
            });
        // clear blocks
        (1..total_blocks as usize).for_each(|i| {
            get_block_cache(i, Arc::clone(&block_dev))
                .lock()
                .modify(0, |blk: &mut DataBlock| {
                    blk.iter_mut().for_each(|v| { *v = 0; });
                })
        });
        efs.init_root();
        sync_block_cache();
        Arc::new(Mutex::new(efs))
    }
    
    fn init_root(&mut self) {
        assert_eq!(self.alloc_inode(), 0);
        let (root_blkid, root_offset) = self.inode_pos(0);
        get_block_cache(root_blkid as usize, Arc::clone(&self.block_dev))
            .lock()
            .modify(root_offset, |inode: &mut DiskInode| {
                inode.init(DiskInodeType::Dir);
            })
    }

    pub fn from_device(block_dev: Arc<dyn BlockDev>) -> Arc<Mutex<Self>> {
        get_block_cache(0, Arc::clone(&block_dev))
            .lock()
            .read(0, |super_blk: &SuperBlock| {
                assert!(super_blk.is_valid(), "Not a valid EFS device");
                let inode_total = super_blk.inode_bitmap_blocks + super_blk.inode_area_blocks;
                let efs = Self {
                    block_dev,
                    inode_bitmap: Bitmap::new(1, super_blk.inode_bitmap_blocks as usize),
                    data_bitmap: Bitmap::new(1 + inode_total as usize, super_blk.data_bitmap_blocks as usize),
                    inode_start: 1 + super_blk.inode_bitmap_blocks,
                    data_start: 1 + inode_total + super_blk.data_bitmap_blocks
                };
                Arc::new(Mutex::new(efs))
            })
    }

    pub fn inode_pos(&self, inode_id: u32) -> (u32, usize) {
        let inode_size = size_of::<DiskInode>();
        let inode_pblk = (BLOCK_SIZE / inode_size) as u32;
        (
            self.inode_start + inode_id / inode_pblk,
            (inode_id % inode_pblk) as usize * inode_size
        )
    }
    pub fn data_block_pos(&self, data_id: u32) -> u32 {
        self.data_start + data_id
    }
    pub fn alloc_inode(&mut self) -> u32 {
        self.inode_bitmap.alloc(&self.block_dev)
            .expect("Cannot allocate block for inode") as u32
    }
    pub fn alloc_data(&mut self) -> u32 {
        self.data_start +
            self.data_bitmap
                .alloc(&self.block_dev)
                .expect("Cannot allocate block for data") as u32
    }
    pub fn dealloc_data(&mut self, block_id: u32) {
        get_block_cache(block_id as usize, Arc::clone(&self.block_dev))
            .lock()
            .modify(0, |blk: &mut DataBlock| {
                blk.iter_mut().for_each(|v| *v = 0)
            });
        self.data_bitmap.dealloc(&self.block_dev, (block_id - self.data_start) as usize);
    }

    pub fn root_vinode(efs: &Arc<Mutex<Self>>) -> VirtInode {
        let block_dev = Arc::clone(&efs.lock().block_dev);
        let (block_id, block_offset) = efs.lock().inode_pos(0);
        VirtInode::new(
            block_id,
            block_offset,
            Arc::clone(efs),
            block_dev
        )
    }
}
