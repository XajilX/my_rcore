use core::str::from_utf8;

use crate::{sbi::cgetchar, task::suspend_curr_task};

use super::File;

pub struct Stdin;

pub struct Stdout;

impl File for Stdin {
    fn readable(&self) -> bool { true }
    
    fn writable(&self) -> bool { false }
    
    fn read(&self, mut buf: crate::mm::pagetab::UserBuffer) -> usize {
        assert_eq!(buf.len(), 1);
        let mut c: usize;
        loop {
            c = cgetchar();
            if c == 0 {
                suspend_curr_task();
            } else {
                break;
            }
        }
        let ch = c as u8;
        unsafe {
            buf.buffers[0].as_mut_ptr().write_volatile(ch);
        }
        1
    }
    
    fn write(&self, _buf: crate::mm::pagetab::UserBuffer) -> usize {
        panic!("Cannot write to stdin!");
    }
}

impl File for Stdout {
    fn readable(&self) -> bool { false }

    fn writable(&self) -> bool { true }

    fn read(&self, _buf: crate::mm::pagetab::UserBuffer) -> usize {
        panic!("Cannot read from stdout!");
    }

    fn write(&self, buf: crate::mm::pagetab::UserBuffer) -> usize {
        for slice in buf.buffers.iter() {
            print!("{}", from_utf8(*slice).unwrap())
        }
        buf.len()
    }
}