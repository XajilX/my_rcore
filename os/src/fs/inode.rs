use alloc::{sync::Arc, vec::Vec};
use bitflags::bitflags;
use lazy_static::lazy_static;
use crate::{drivers::BLOCK_DEV, uthr::UThrCell};
use easyfs::{vfs::VirtInode, EzFileSys};

use super::File;

pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: UThrCell<OSInodeInner>
}

pub struct OSInodeInner {
    offset: usize,
    inode: Arc<VirtInode>
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Arc<VirtInode>) -> Self {
        Self {
            readable, writable,
            inner: unsafe { UThrCell::new(OSInodeInner {
                offset: 0, inode
            })}
        }
    }
    pub fn read_app(&self) -> Vec<u8> {
        let mut inner = self.inner.get_refmut();
        let mut buf = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inner.inode.read_at(inner.offset, &mut buf);
            if len == 0 {
                break;
            }
            inner.offset += len;
            v.extend_from_slice(&buf[..len])
        }
        v
    }
    pub fn open(name: &str, flags: OpenFlag) -> Option<Arc<OSInode>> {
        let (readable, writable) = flags.into_readwrite();
        if flags.contains(OpenFlag::CREATE) {
            if let Some(inode) = ROOT_INODE.find(name) {
                inode.clear();
                Some(Arc::new(OSInode::new(
                    readable,
                    writable,
                    inode
                )))
            } else {
                ROOT_INODE.create(name)
                    .map(|inode| {
                        Arc::new(OSInode::new(
                            readable,
                            writable,
                            inode
                        ))
                    })
            }
        } else {
            ROOT_INODE.find(name)
                .map(|inode| {
                    if flags.contains(OpenFlag::TRUNC) {
                        inode.clear()
                    }
                    Arc::new(OSInode::new(
                        readable,
                        writable,
                        inode
                    ))
                })
        }
    }
}

impl File for OSInode {
    fn readable(&self) -> bool { self.readable }
    fn writable(&self) -> bool { self.writable }
    fn read(&self, mut buf: crate::mm::pagetab::UserBuffer) -> usize {
        let mut inner = self.inner.get_refmut();
        let mut tot_read_sz = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_sz = inner.inode.read_at(inner.offset, *slice);
            if read_sz == 0 {
                break;
            }
            inner.offset += read_sz;
            tot_read_sz += read_sz;
        }
        tot_read_sz
    }
    fn write(&self, buf: crate::mm::pagetab::UserBuffer) -> usize {
        let mut inner = self.inner.get_refmut();
        let mut tot_write_sz = 0usize;
        for slice in buf.buffers.iter() {
            let write_sz = inner.inode.write_at(inner.offset, *slice);
            assert_eq!(write_sz, slice.len());
            inner.offset += write_sz;
            tot_write_sz += write_sz;
        }
        tot_write_sz
    }
}

lazy_static! {
    pub static ref ROOT_INODE: Arc<VirtInode> = {
        let efs = EzFileSys::from_device(BLOCK_DEV.clone());
        Arc::new(EzFileSys::root_vinode(&efs))
    };
}


bitflags! {
    pub struct OpenFlag: u32 {
        const RDONLY = 0;
        const WRONLY = 1;
        const RDWR   = 2;
        const CREATE = 1 << 9;
        const TRUNC  = 1 << 10;
    }
}

impl OpenFlag {
    pub fn into_readwrite(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}
