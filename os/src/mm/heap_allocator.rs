use buddy_system_allocator::LockedHeap;
use crate::config::KERN_HEAP_SIZE;

#[global_allocator]
static HEAP_ALLOC: LockedHeap = LockedHeap::empty();

static mut HEAP_SPACE: [u8; KERN_HEAP_SIZE] = [0; KERN_HEAP_SIZE];

pub fn init_heap() {
    unsafe {
        HEAP_ALLOC.lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERN_HEAP_SIZE);
    }
}

#[alloc_error_handler]
pub fn alloc_error_handler(layout: core::alloc::Layout) -> !{
    panic!("Heap allocation error! layout = {:?}", layout);
}

#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    extern "C" {
        fn sbss();
        fn ebss();
    }
    //  Test Box
    let bss_range = sbss as usize..ebss as usize;
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);
    
    //  Test vec
    let mut v: Vec<usize> = Vec::new();
    (0..500).for_each(|i| v.push(i));
    for i in 0..500 {
        assert_eq!(v[i], i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    println!("[kernel] Heap test success!");
}