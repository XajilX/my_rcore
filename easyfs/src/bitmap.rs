use alloc::sync::Arc;

use crate::{BLOCK_SIZE, BlockDev, cache_man::get_block_cache};

pub const BLOCK_BITS: usize = BLOCK_SIZE * 8;
pub struct Bitmap {
    start_block_id: usize,
    blocks: usize
}
impl Bitmap {
    pub fn new(start_block_id: usize, blocks: usize) -> Self {
        Self {
            start_block_id, blocks
        }
    }
    pub fn alloc(&self, block_dev: &Arc<dyn BlockDev>) -> Option<usize> {
        (0..self.blocks).find_map(|id|
            get_block_cache(id + self.start_block_id, Arc::clone(block_dev))
                .lock()
                .modify(
                    0, 
                    |bitmap_block: &mut BitmapBlock| bitmap_block
                        .iter_mut()
                        .enumerate()
                        .find(|(_, bits)| **bits != u64::MAX)
                        .map(|(bits_pos, bits)| {
                            let inner_pos = bits.trailing_ones();
                            *bits |= 1u64 << inner_pos;
                            id * BLOCK_BITS + bits_pos * 64 + inner_pos as usize
                        })
                )
        )
    }
    fn pos_decomp(pos: usize) -> (usize, usize, usize) {
        let (block_pos, inner_pos) = (pos / BLOCK_BITS, pos % BLOCK_BITS);
        (block_pos, inner_pos >> 6, inner_pos & 63)
    }
    pub fn dealloc(&self, block_dev: &Arc<dyn BlockDev>, pos: usize) {
        let (block_pos, bits_pos, inner_pos) = Self::pos_decomp(pos);
        get_block_cache(block_pos + self.start_block_id, Arc::clone(block_dev))
            .lock()
            .modify(
                0,
                |bitmap_block: &mut BitmapBlock| {
                    assert!((bitmap_block[bits_pos] & (1u64 << inner_pos)) > 0, "Block dealloc before alloc");
                    bitmap_block[bits_pos] -= 1u64 << inner_pos;
                }
            );
    }
}

type BitmapBlock = [u64; 64];