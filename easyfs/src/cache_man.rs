use alloc::{collections::VecDeque, sync::Arc};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{block_cache::BlockCache, BlockDev};

const MAX_CACHE_NUM: usize = 16;

pub struct BlockCacheMan {
    queue: VecDeque<(usize, Arc<Mutex<BlockCache>>)>
}
impl BlockCacheMan {
    pub fn new() -> Self {
        Self { queue: VecDeque::new() }
    }
    pub fn get_block_cache(&mut self, block_id: usize, block_dev: Arc<dyn BlockDev>) -> Arc<Mutex<BlockCache>> {
        if let Some((_, bc)) = self.queue
            .iter()
            .find(|(id, _)| *id == block_id)
        {
            return Arc::clone(bc);
        }
        if self.queue.len() >= MAX_CACHE_NUM {
            if let Some((idx, _)) = self.queue
                .iter()
                .enumerate()
                .find(|(_, (_, bc))| Arc::strong_count(bc) == 1)
            {
                self.queue.drain(idx..=idx);
            } else {
                panic!("Run out of BlockCache! ");
            }
        }
        let bc = Arc::new(Mutex::new(
            BlockCache::new(block_id, Arc::clone(&block_dev))
        ));
        self.queue.push_back((block_id, Arc::clone(&bc)));
        bc
    }
}

lazy_static! {
    pub static ref BLOCK_CACHE_MAN: Mutex<BlockCacheMan> = Mutex::new(
        BlockCacheMan::new()
    );
}

pub fn get_block_cache(block_id: usize, block_dev: Arc<dyn BlockDev>) -> Arc<Mutex<BlockCache>> {
    BLOCK_CACHE_MAN.lock().get_block_cache(block_id, block_dev)
}

pub fn sync_block_cache() {
    let cache_man = BLOCK_CACHE_MAN.lock();
    for (_, cache) in cache_man.queue.iter() {
        cache.lock().sync();
    }
}