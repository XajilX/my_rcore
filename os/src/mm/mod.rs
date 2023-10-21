use log::info;

mod heap_allocator;
pub mod address;
pub mod pagetab;
pub mod frame_allocator;
pub mod memset;
pub mod memarea;

pub fn init() {
    heap_allocator::init_heap();
    info!("[kernel] heap initiated");
    frame_allocator::init_frame_alloc();
    info!("[kernel] frame_alloc initiated");
    frame_alloc_test();
    memset::kern_mem_init();
    info!("[kernel] kernel space initiated");
}

#[allow(unused)]
pub fn heap_test() {
    heap_allocator::heap_test()
}

#[allow(unused)]
pub fn remap_test() {
    memset::remap_test()
}

#[allow(unused)]
pub fn frame_alloc_test() {
    frame_allocator::frame_alloc_test()
}