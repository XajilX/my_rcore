
use alloc::{string::String, sync::Arc, vec::Vec};
use spin::{Mutex, MutexGuard};

use crate::{cache_man::{get_block_cache, sync_block_cache}, efs::EzFileSys, layout::{DirEntry, DiskInode, DiskInodeType, DIRENT_SIZE}, BlockDev};

pub struct VirtInode {
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<EzFileSys>>,
    block_dev: Arc<dyn BlockDev>
}

impl VirtInode {
    pub fn new(
        block_id: u32,
        block_offset: usize,
        fs: Arc<Mutex<EzFileSys>>,
        block_dev: Arc<dyn BlockDev>
    ) -> Self {
        Self {
            block_id: block_id as usize,
            block_offset,
            fs,
            block_dev
        }
    }
    fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
        get_block_cache(self.block_id as usize, Arc::clone(&self.block_dev))
            .lock()
            .read(self.block_offset, f)
    }
    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
        get_block_cache(self.block_id as usize, Arc::clone(&self.block_dev))
            .lock()
            .modify(self.block_offset, f)
    }
    
    fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
        assert!(disk_inode.is_dir());
        let file_cnt = (disk_inode.size as usize) / DIRENT_SIZE;
        let mut dirent = DirEntry::new();
        for i in 0..file_cnt {
            assert_eq!(
                disk_inode.read_at(DIRENT_SIZE * i, dirent.as_bytes_mut(), &self.block_dev),
                DIRENT_SIZE
            );
            if dirent.name() == name {
                return Some(dirent.inode());
            }
        }
        None
    }

    pub fn find(&self, name: &str) -> Option<Arc<VirtInode>> {
        let fs = self.fs.lock();
        self.read_disk_inode(|inode: &DiskInode| {
            self.find_inode_id(name, inode).map(|inode_id| {
                let (block_id, block_offset) = fs.inode_pos(inode_id);
                Arc::new(Self::new(
                    block_id,
                    block_offset,
                    self.fs.clone(),
                    self.block_dev.clone()
                ))
            })
        })
    }

    pub fn ls(&self) -> Vec<String> {
        let _fs = self.fs.lock();
        self.read_disk_inode(|inode: &DiskInode| {
            let file_cnt = (inode.size as usize) / DIRENT_SIZE;
            let mut v: Vec<String> = Vec::new();
            for i in 0..file_cnt {
                let mut dirent = DirEntry::new();
                assert_eq!(
                    inode.read_at(i * DIRENT_SIZE, dirent.as_bytes_mut(), &self.block_dev),
                    DIRENT_SIZE
                );
                v.push(String::from(dirent.name()));
            }
            v
        })
    }

    fn increase_size(
        &self,
        new_size: u32,
        inode: &mut DiskInode,
        fs: &mut MutexGuard<EzFileSys>
    ) {
        if new_size < inode.size {
            return;
        }
        let blk_needed = inode.blocks_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blk_needed {
            v.push(fs.alloc_data());
        }
        assert!(v.len() == blk_needed as usize);
        inode.increase_size(new_size, v, &self.block_dev);
    }

    pub fn create(&self, name: &str) -> Option<Arc<VirtInode>> {
        let mut fs = self.fs.lock();
        if self.read_disk_inode(|inode| {
            assert!(inode.is_dir());
            self.find_inode_id(name, inode)
        }).is_some() {
            None
        } else {
            let new_inode_id = fs.alloc_inode();
            let (new_inode_block_id, new_inode_offset) = fs.inode_pos(new_inode_id);
            get_block_cache(new_inode_block_id as usize, Arc::clone(&self.block_dev))
                .lock().modify(new_inode_offset, |inode: &mut DiskInode| {
                    inode.init(DiskInodeType::File)
                });
            self.modify_disk_inode(|inode| {
                let file_cnt = (inode.size as usize) / DIRENT_SIZE;
                let new_sz = (file_cnt + 1) * DIRENT_SIZE;
                self.increase_size(new_sz as u32, inode, &mut fs);
                let dirent = DirEntry::with_name_inode(name, new_inode_id);
                inode.write_at(
                    file_cnt * DIRENT_SIZE, 
                    dirent.as_bytes(), 
                    &self.block_dev
                )
            });
            sync_block_cache();
            Some(Arc::new(Self::new(
                new_inode_block_id,
                new_inode_offset, 
                self.fs.clone(),
                self.block_dev.clone()
            )))
        }
    }

    pub fn clear(&self) {
        let mut fs = self.fs.lock();
        self.modify_disk_inode(|inode| {
            let sz = inode.size;
            let data_dealloc = inode.clear_size(&self.block_dev);
            assert!(data_dealloc.len() == DiskInode::total_blocks(sz) as usize);
            for block in data_dealloc.into_iter() {
                fs.dealloc_data(block)
            }
        });
        sync_block_cache();
    }

    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let _fs = self.fs.lock();
        self.read_disk_inode(|inode| {
            inode.read_at(offset, buf, &self.block_dev)
        })
    }

    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.lock();
        let size = self.modify_disk_inode(|inode| {
            self.increase_size((offset + buf.len()) as u32, inode, &mut fs);
            inode.write_at(offset, buf, &self.block_dev)
        });
        sync_block_cache();
        size
    }
}