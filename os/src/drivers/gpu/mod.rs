pub mod virtio_gpu;
use core::any::Any;

use crate::sync::UThrCell;

use alloc::sync::Arc;
use lazy_static::lazy_static;

pub trait GpuDevice: Sync + Send + Any {
    fn get_framebuf(&self) -> &mut [u8];
    fn flush(&self);
    fn resolution(&self) -> (u32, u32);
}

lazy_static! {
    pub static ref GPU_DEV: UThrCell<Option<Arc<dyn GpuDevice>>> = unsafe {
        UThrCell::new(None)
    };
}