mod heap_allocator;
pub mod address;
pub mod pagetab;
pub mod frame_allocator;
pub mod memset;
pub mod memarea;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_alloc();
    memset::kern_mem_init();
}
