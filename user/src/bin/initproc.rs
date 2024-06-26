#![no_std]
#![no_main]

#[macro_use]
extern crate user;

use user::{fork, exec, wait, yield_};

#[no_mangle]
fn main() -> i32 {
    if fork() == 0 {
        exec("ushell", &["ushell\0".as_ptr(), 0 as *const u8]);
        unreachable!()
    }
    println!("[ init ] start");
    loop {
        let mut exit_code = 0;
        match wait(&mut exit_code) {
            -1 => { yield_(); }
            pid => {
                println!(
                    "[ init ] Released a zombie process, pid={}, exit_code={}",
                    pid, exit_code
                )
            }
        }
    }
}