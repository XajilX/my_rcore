use core::{cmp::min, ffi::CStr};

use alloc::{sync::Arc, vec::Vec};

use crate::{BLOCK_SIZE, BlockDevice, cache_man::get_block_cache};

const EZFS_MAGIC: u32 = 0x53465A45;     //  b"EZFS"
const INODE_DIRECT_COUNT: usize = 28;

#[repr(C)]
pub struct SuperBlock {
    magic_num: u32,
    pub total_blocks: u32,
    pub inode_bitmap_blocks: u32,
    pub inode_area_blocks: u32,
    pub data_bitmap_blocks: u32,
    pub data_area_blocks: u32
}
impl SuperBlock {
    pub fn init(
        &mut self,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
        inode_area_blocks: u32,
        data_bitmap_blocks: u32,
        data_area_blocks: u32
    ) {
        *self = Self {
            magic_num: EZFS_MAGIC,
            total_blocks,
            inode_bitmap_blocks,
            inode_area_blocks,
            data_bitmap_blocks,
            data_area_blocks
        }
    }
    
    pub fn is_valid(&self) -> bool {
        self.magic_num == EZFS_MAGIC
    }
}

#[derive(PartialEq)]
pub enum DiskInodeType {
    File,
    Dir
}

#[repr(C)]
pub struct DiskInode {
    pub size: u32,
    pub direct: [u32; INODE_DIRECT_COUNT],
    pub indirect: [u32; 2],
    ty_inode: DiskInodeType
}

const INODE_INDIRECT_COUNT: usize = BLOCK_SIZE / 4;
const INODE_MAX_COUNT: usize = INODE_DIRECT_COUNT + INODE_INDIRECT_COUNT + INODE_INDIRECT_COUNT * INODE_INDIRECT_COUNT;
type IndirectBlock = [u32; BLOCK_SIZE / 4];
type DataBlock = [u8; BLOCK_SIZE];
impl DiskInode {
    pub fn init(&mut self, ty_inode: DiskInodeType) {
        self.size = 0;
        self.direct.iter_mut().for_each(|v| *v = 0);
        self.indirect = [0, 0];
        self.ty_inode = ty_inode;
    }
    pub fn is_dir(&self) -> bool { self.ty_inode == DiskInodeType::Dir }
    pub fn is_file(&self) -> bool { self.ty_inode == DiskInodeType::File }
    pub fn get_block_id(&self, inner_id: u32, block_dev: &Arc<dyn BlockDevice>) -> u32 {
        let inner_id = inner_id as usize;
        if inner_id < INODE_DIRECT_COUNT {
            self.direct[inner_id]
        } else if inner_id < INODE_DIRECT_COUNT + INODE_INDIRECT_COUNT {
            get_block_cache(self.indirect[0] as usize, Arc::clone(block_dev))
                .lock()
                .read(0, |indir_blk: &IndirectBlock| {
                    indir_blk[inner_id - INODE_DIRECT_COUNT]
                })
        } else {
            let tail = inner_id - INODE_DIRECT_COUNT - INODE_INDIRECT_COUNT;
            let indir1 = get_block_cache(self.indirect[1] as usize, Arc::clone(block_dev))
                .lock()
                .read(0, |indir_blk: &IndirectBlock| {
                    indir_blk[tail / INODE_INDIRECT_COUNT]
                });
            get_block_cache(indir1 as usize, Arc::clone(block_dev))
                .lock()
                .read(0, |indir_blk: &IndirectBlock| {
                    indir_blk[tail % INODE_INDIRECT_COUNT]
                })
        }
    }
    pub fn data_blocks(&self) -> u32 {
        (self.size + BLOCK_SIZE as u32 - 1) / BLOCK_SIZE as u32
    }
    fn _data_blocks(size: u32) -> u32 {
        (size + BLOCK_SIZE as u32 - 1) / BLOCK_SIZE as u32
    }
    pub fn total_blocks(size: u32) -> u32 {
        let blks = Self::_data_blocks(size) as usize;
        let total: usize = blks + 
            if blks > INODE_DIRECT_COUNT { 1 } else { 0 } +
            if blks > INODE_DIRECT_COUNT + INODE_INDIRECT_COUNT { 
                (blks - INODE_DIRECT_COUNT - INODE_INDIRECT_COUNT + INODE_INDIRECT_COUNT - 1) / INODE_INDIRECT_COUNT + 1
            } else { 0 };
        total as u32
    }
    pub fn blocks_needed(&self, new_size: u32) -> u32 {
        assert!(new_size >= self.size);
        Self::total_blocks(new_size) - Self::total_blocks(self.size)
    }




    pub fn increase_size(&mut self, new_size: u32, new_blocks: Vec<u32>, block_dev: &Arc<dyn BlockDevice>) {
        let mut curr_blks = self.data_blocks();
        let mut goal_blks = Self::_data_blocks(new_size);
        let mut iter_blks = new_blocks.into_iter();
        assert!(goal_blks <= INODE_MAX_COUNT as u32, "new_size too large");
        // direct
        while curr_blks < min(goal_blks, INODE_DIRECT_COUNT as u32) {
            self.direct[curr_blks as usize] = iter_blks.next()
                .expect("Not enough block id for increase_size");
            curr_blks += 1;
        }
        if goal_blks <= INODE_DIRECT_COUNT as u32 {
            self.size = new_size;
            return;
        }
        // indirect 1
        self.indirect[0] = iter_blks.next()
            .expect("Not enough block id for increase_size");
        curr_blks -= INODE_DIRECT_COUNT as u32;
        goal_blks -= INODE_DIRECT_COUNT as u32;
        get_block_cache(self.indirect[0] as usize, Arc::clone(block_dev))
            .lock()
            .modify(0, |indir_blk: &mut IndirectBlock| {
                while curr_blks < min(goal_blks, INODE_INDIRECT_COUNT as u32) {
                    indir_blk[curr_blks as usize] = iter_blks.next()
                        .expect("Not enough block id for increase_size");
                    curr_blks += 1;
                }
            });
        if goal_blks <= INODE_INDIRECT_COUNT as u32 {
            self.size = new_size;
            return;
        }
        // indirect 2
        self.indirect[1] = iter_blks.next()
            .expect("Not enough block id for increase_size");
        curr_blks -= INODE_INDIRECT_COUNT as u32;
        goal_blks -= INODE_INDIRECT_COUNT as u32;
        let mut a0 = curr_blks as usize / INODE_INDIRECT_COUNT;
        let a1 = goal_blks as usize / INODE_INDIRECT_COUNT;
        let b1 = goal_blks as usize % INODE_INDIRECT_COUNT;
        get_block_cache(self.indirect[1] as usize, Arc::clone(block_dev))
            .lock()
            .modify(0, |indir2:&mut IndirectBlock| {
                while a0 <= a1 {
                    if a0 == a1 && b1 == 0 { break; }
                    indir2[a0] = iter_blks.next()
                        .expect("Not enough block id for increase_size");
                    // second layer
                    get_block_cache(indir2[a0] as usize, Arc::clone(block_dev))
                        .lock()
                        .modify(0, |indir: &mut IndirectBlock| {
                            for b0 in 0..(
                                if a0 < a1 { INODE_INDIRECT_COUNT } else { b1 }
                            ) {
                                indir[b0] = iter_blks.next()
                                    .expect("Not enough block id for increase_size");
                            }
                        });
                    a0 += 1;
                }
            });
        self.size = new_size;
    }



    pub fn clear_size(&mut self, block_dev: &Arc<dyn BlockDevice>) -> Vec<u32> {
        let mut v: Vec<u32> = Vec::new();
        let mut tot_blks = self.data_blocks() as usize;
        let mut curr_blks = 0usize;
        // direct
        while curr_blks < min(tot_blks, INODE_DIRECT_COUNT) {
            v.push(self.direct[curr_blks]);
            self.direct[curr_blks] = 0;
            curr_blks += 1;
        }
        if tot_blks <= INODE_DIRECT_COUNT {
            self.size = 0;
            return v;
        }
        // indirect 1
        v.push(self.indirect[0]);
        tot_blks -= INODE_DIRECT_COUNT;
        curr_blks = 0;
        get_block_cache(self.indirect[0] as usize, Arc::clone(block_dev))
            .lock()
            .modify(0, |indir: &mut IndirectBlock| {
                while curr_blks < min(tot_blks, INODE_INDIRECT_COUNT) {
                    v.push(indir[curr_blks]);
                    curr_blks += 1;
                }
            });
        self.indirect[0] = 0;
        if tot_blks <= INODE_INDIRECT_COUNT {
            self.size = 0;
            return v;
        }
        // indirect 2
        v.push(self.indirect[1]);
        tot_blks -= INODE_INDIRECT_COUNT;
        let a1 = tot_blks / INODE_INDIRECT_COUNT;
        let b1 = tot_blks % INODE_INDIRECT_COUNT;
        get_block_cache(self.indirect[1] as usize, Arc::clone(block_dev))
            .lock()
            .modify(0, |indir2: &mut IndirectBlock| {
                for x in indir2.iter_mut().take(a1) {
                    v.push(*x);
                    get_block_cache(*x as usize, Arc::clone(block_dev))
                        .lock()
                        .modify(0, |indir: &mut IndirectBlock| {
                            for x in indir.iter() {
                                v.push(*x);
                            }
                        })
                }
                // recycle last l1 indirect block
                if b1 > 0 {
                    v.push(indir2[a1]);
                    get_block_cache(indir2[a1] as usize, Arc::clone(block_dev))
                        .lock()
                        .modify(0, |indir: &mut IndirectBlock| {
                            for x in indir.iter().take(b1) {
                                v.push(*x);
                            }
                        })
                }
            });
        self.indirect[1] = 0;
        self.size = 0;
        v
    }

    pub fn read_at(&self, offset: usize, buf: &mut [u8], block_dev: &Arc<dyn BlockDevice>) -> usize {
        let mut start = offset;
        let end = min(offset + buf.len(), self.size as usize);
        if start >= end {
            return 0;
        }
        let mut start_blk = start / BLOCK_SIZE;
        let mut read_size = 0usize;
        loop {
            let mut curr_blk_end = (start / BLOCK_SIZE + 1) * BLOCK_SIZE;
            curr_blk_end = min(curr_blk_end, end);
            let curr_blk_size = curr_blk_end - start;
            let dst = &mut buf[read_size..read_size + curr_blk_size];
            get_block_cache(
                self.get_block_id(start_blk as u32, block_dev) as usize,
                Arc::clone(block_dev))
                .lock()
                .read(0, |blk: &DataBlock| {
                    let src = &blk[start % BLOCK_SIZE..start % BLOCK_SIZE + curr_blk_size];
                    dst.copy_from_slice(src);
                });
            read_size += curr_blk_size;
            if curr_blk_end >= end {
                break;
            }
            start_blk += 1;
            start += curr_blk_size;
        }
        read_size
    }

    pub fn write_at(&mut self, offset: usize, buf: &[u8], block_dev: &Arc<dyn BlockDevice>) -> usize {
        let mut start = offset;
        let end = min(offset + buf.len(), self.size as usize);
        assert!(start <= end);
        let mut start_blk = start / BLOCK_SIZE;
        let mut write_size = 0usize;
        loop {
            let mut curr_blk_end = (start / BLOCK_SIZE + 1) * BLOCK_SIZE;
            curr_blk_end = min(curr_blk_end, end);
            let curr_blk_size = curr_blk_end - start;
            get_block_cache(
                self.get_block_id(start_blk as u32, block_dev) as usize,
                Arc::clone(block_dev))
                .lock()
                .modify(0, |blk: &mut DataBlock| {
                    let src = &buf[write_size..write_size + curr_blk_size];
                    let dst = &mut blk[start % BLOCK_SIZE..start % BLOCK_SIZE + curr_blk_size];
                    dst.copy_from_slice(src);
                });
            write_size += curr_blk_size;
            if curr_blk_end >= end {
                break;
            }
            start_blk += 1;
            start += curr_blk_size;
        }
        write_size
    }
}

const FILENAME_LIM: usize = 27;
pub const DIRENT_SIZE: usize = 32;
#[repr(C)]
pub struct DirEntry {
    name: [u8; FILENAME_LIM + 1],   // +1 for \0
    inode: u32
}

impl DirEntry {
    pub fn new() -> Self {
        Self {
            name: [0u8; FILENAME_LIM + 1],
            inode: 0
        }
    }
    pub fn with_name_inode(name: &str, inode: u32) -> Self {
        let mut bytes = [0u8; FILENAME_LIM + 1];
        let bname = name.as_bytes();
        bytes[..bname.len()].copy_from_slice(bname);
        Self {
            name: bytes,
            inode
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self as *const _ as usize as *const u8, DIRENT_SIZE)
        }
    }
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(self as *mut _ as usize as *mut u8, DIRENT_SIZE)
        }
    }

    pub fn name(&self) -> &str {
        CStr::from_bytes_until_nul(&self.name).unwrap().to_str().unwrap()
    }
    pub fn inode(&self) -> u32 { self.inode }
}
