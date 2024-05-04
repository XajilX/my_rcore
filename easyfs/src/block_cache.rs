use core::mem::size_of;

use alloc::sync::Arc;

use crate::{BLOCK_SIZE, BlockDevice};

pub struct BlockCache {
    cache: [u8; BLOCK_SIZE],
    block_id: usize,
    block_dev: Arc<dyn BlockDevice>,
    modified: bool
}
impl BlockCache {
    pub fn new(block_id: usize, block_dev: Arc<dyn BlockDevice>) -> Self {
        let mut cache = [0u8; BLOCK_SIZE];
        block_dev.read_block(block_id, &mut cache);
        Self {
            cache,
            block_id,
            block_dev,
            modified: false
        }
    }
    fn addr_of_offset(&self, offset: usize) -> usize {
        &self.cache[offset] as *const _ as usize
    }
    pub fn get_ref<T>(&self, offset: usize) -> &T where
        T: Sized
    {
        let ty_size = size_of::<T>();
        assert!(offset + ty_size <= BLOCK_SIZE);
        let addr = self.addr_of_offset(offset);
        unsafe {
            & *(addr as *const T)
        }
    }
    pub fn get_mut<T>(&mut self, offset: usize) -> &mut T where
        T: Sized
    {
        let ty_size = size_of::<T>();
        assert!(offset + ty_size <= BLOCK_SIZE);
        let addr = self.addr_of_offset(offset);
        self.modified = true;
        unsafe {
            &mut *(addr as *mut T)
        }
    }
    pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        f(self.get_ref(offset))
    }
    pub fn modify<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(self.get_mut(offset))
    }
    pub fn sync(&mut self) {
        if self.modified {
            self.modified = false;
            self.block_dev.write_block(self.block_id, &self.cache);
        }
    }
}

impl Drop for BlockCache {
    fn drop(&mut self) {
        self.sync()
    }
}