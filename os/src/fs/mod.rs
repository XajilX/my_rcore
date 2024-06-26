pub mod inode;
pub mod stdio;
pub mod pipe;
pub mod eventfd;
use crate::{fs::inode::ROOT_INODE, mm::pagetab::UserBuffer};

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}

pub fn list_apps() {
    println!("/**** APPS ****");
    for app in ROOT_INODE.ls().iter() {
        println!("{}", app);
    }
    println!("**************/");
}