pub mod virtio_input;

use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;

use crate::sync::UThrCell;

use super::Device;

pub trait InputDevice: Device {
    fn read_event(&self) -> u64;
    fn is_empty(&self) -> bool;
}

lazy_static! {
    pub static ref INPUT_DEV: UThrCell<Vec<Arc<dyn InputDevice>>> = unsafe {
        UThrCell::new(Vec::new())
    };
}