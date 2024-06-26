#![no_std]
#![no_main]

#[macro_use]
extern crate user;

extern crate alloc;

use alloc::vec;
use user::exit;
use user::{semaphore_create, semaphore_down, semaphore_up};
use user::{sleep, thread_create, waittid};

const SEM_SYNC: usize = 0;

unsafe fn first() -> ! {
    sleep(10);
    println!("First work and wakeup Second");
    semaphore_up(SEM_SYNC);
    exit(0)
}

unsafe fn second() -> ! {
    println!("Second want to continue,but need to wait first");
    semaphore_down(SEM_SYNC);
    println!("Second can work now");
    exit(0)
}

#[no_mangle]
pub fn main() -> i32 {
    // create semaphores
    println!("check1");
    assert_eq!(semaphore_create(0) as usize, SEM_SYNC);
    // create threads
    println!("check2");
    let threads = vec![
        thread_create(first as usize, 0),
        thread_create(second as usize, 0),
    ];
    println!("check3");
    // wait for all threads to complete
    for thread in threads.iter() {
        waittid(*thread as usize);
    }
    println!("sync_sem passed!");
    0
}
