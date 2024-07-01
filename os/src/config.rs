pub const USER_STACK_SIZE: usize = 0x1000;
pub const KERN_STACK_SIZE: usize = 0x2000;

pub const KERN_HEAP_SIZE: usize = 0x300000;
pub const MEM_END: usize = 0x81000000;

pub const PAGE_SIZE_BITS: usize = 12;
pub const PAGE_SIZE: usize = 0x1000;

pub const ADDR_TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const ADDR_TRAPCONTEXT: usize = ADDR_TRAMPOLINE - PAGE_SIZE;

pub const CLOCK_FREQ: usize = 12500000;

pub const VIRT_PLIC: usize = 0x0c00_0000;
pub const VIRT_UART: usize = 0x1000_0000;
pub const VIRT_MMIO1: usize = 0x10001000;

pub const MMIO: &[(usize, usize)] = &[
    (0x2000000, 0x10000),
    (0x0c00_0000, 0x600000),    // PLIC
    (0x1000_0000, 0x9000),      // VIRTIO MMIO 
];
