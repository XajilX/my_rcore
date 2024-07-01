use core::ptr::write_volatile;

use log::debug;

use crate::{drivers::GPU_DEV, sync::UThrCell};

use super::File;

pub struct Fb {
    offset: UThrCell<usize>,
}

impl Fb {
    pub fn new() -> Self { Self { 
        offset: unsafe {
            UThrCell::new(0)
        }
    } }
}

impl File for Fb {
    fn readable(&self) -> bool { true }

    fn writable(&self) -> bool { true }

    fn seekable(&self) -> bool { true }

    fn read(&self, buf: crate::mm::pagetab::UserBuffer) -> usize {
        let gpu_mut = GPU_DEV.get_refmut();
        let gpu = gpu_mut.as_ref().unwrap().clone();
        let fbuf = gpu.get_framebuf();
        let mut offset = self.offset.get_refmut();
        let mut tot_read = 0;
        for b in buf.into_iter() {
            if *offset >= fbuf.len() {
                break;
            }
            unsafe {
                write_volatile(b, fbuf[*offset]);
            }
            tot_read += 1;
            *offset += 1;
        }
        tot_read
    }

    fn write(&self, buf: crate::mm::pagetab::UserBuffer) -> usize {
        let gpu_mut = GPU_DEV.get_refmut();
        let gpu = gpu_mut.as_ref().unwrap().clone();
        let (xres, yres) = gpu.resolution();
        let len = xres * yres * 4;
        let fbuf = gpu.get_framebuf();
        let mut offset = self.offset.get_refmut();
        let mut tot_writ = 0;
        for b in buf.into_iter() {
            if *offset >= fbuf.len() {
                break;
            }
            unsafe {
                fbuf[*offset] = *b;
            }
            tot_writ += 1;
            *offset += 1;
        }
        debug!("gpu fb offset: {}", *offset);
        if *offset >= len as usize {
            debug!("gpu flush");
            *offset = 0;
            gpu.flush();
        }
        tot_writ
    }
    fn seek(&self, offset: isize, whence: usize) {
        let mut inner = self.offset.get_refmut();
        if whence == 0 {
            if offset < 0 {
                *inner = 0;
            } else {
                *inner = offset as usize;
            }
        } else {
            if offset < -(*inner as isize) {
                *inner = 0;
            } else {
                *inner += offset as usize;
            }
        }
        let gpu_mut = GPU_DEV.get_refmut();
        let gpu = gpu_mut.as_ref().unwrap().clone();
        let (xres, yres) = gpu.resolution();
        let len = xres * yres * 4;
        if *inner >= len as usize {
            debug!("gpu flush");
            *inner = 0;
            gpu.flush();
        }
        debug!("gpu fb offset: {}", *inner);
    }
}