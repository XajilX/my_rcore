use alloc::sync::Arc;
use easyfs::BlockDev;
use crate::sync::UThrCell;

use super::Device;
use lazy_static::lazy_static;

pub mod virtio_blk;

pub trait BlockDevice: BlockDev + Device {}

lazy_static! {
    pub static ref BLOCK_DEV: UThrCell<Option<Arc<dyn BlockDevice>>> = unsafe {
        UThrCell::new(None)
    };
}