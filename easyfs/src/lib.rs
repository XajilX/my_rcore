#![no_std]

extern crate alloc;

pub mod block_dev;
pub mod block_cache;
pub mod cache_man;
pub mod layout;
pub mod efs;
pub mod vfs;
mod bitmap;

pub use block_dev::*;
pub use efs::EzFileSys;

pub const BLOCK_SIZE: usize = 512;