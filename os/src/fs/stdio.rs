use core::str::from_utf8;

use crate::drivers::SERIAL_DEV;

use super::File;

pub struct Stdin;

pub struct Stdout;

impl File for Stdin {
    fn readable(&self) -> bool { true }
    
    fn writable(&self) -> bool { false }
    
    fn seekable(&self) -> bool { false }

    fn read(&self, mut buf: crate::mm::pagetab::UserBuffer) -> usize {
        assert_eq!(buf.len(), 1);
        let ch = SERIAL_DEV.read();
        unsafe {
            buf.buffers[0].as_mut_ptr().write_volatile(ch);
        }
        1
    }
    
    fn write(&self, _buf: crate::mm::pagetab::UserBuffer) -> usize {
        panic!("Cannot write to stdin!");
    }

    fn seek(&self, _offset: isize, _whence: usize) {
        panic!("Unable to seek");
    }
}

impl File for Stdout {
    fn readable(&self) -> bool { false }

    fn writable(&self) -> bool { true }

    fn seekable(&self) -> bool { false }

    fn read(&self, _buf: crate::mm::pagetab::UserBuffer) -> usize {
        panic!("Cannot read from stdout!");
    }

    fn write(&self, buf: crate::mm::pagetab::UserBuffer) -> usize {
        for slice in buf.buffers.iter() {
            print!("{}", from_utf8(*slice).unwrap())
        }
        buf.len()
    }

    fn seek(&self, _offset: isize, _whence: usize) {
        panic!("Unable to seek");
    }
}