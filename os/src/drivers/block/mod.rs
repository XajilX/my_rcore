use alloc::sync::Arc;
use easyfs::BlockDevice;
use lazy_static::lazy_static;

use crate::drivers::block::virtio_blk::VirtIOBlock;

pub mod virtio_blk;

lazy_static! {
    pub static ref BLOCK_DEV: Arc<dyn BlockDevice> = {
        Arc::new(VirtIOBlock::new())
    };
}