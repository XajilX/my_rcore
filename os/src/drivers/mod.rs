mod block;
mod plic;
mod uart;
mod input;
mod gpu;

use core::{any::Any, ptr::NonNull};
use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;

use log::{debug, info, warn};
pub use block::BLOCK_DEV;
pub use uart::SERIAL_DEV;
pub use input::INPUT_DEV;
pub use gpu::GPU_DEV;
use plic::{IntrTargetPriority, Plic};
use virtio_drivers::{transport::{mmio::{MmioTransport, VirtIOHeader}, DeviceType, Transport}, Hal};

use crate::{config::{VIRT_MMIO1, VIRT_PLIC}, drivers::{block::virtio_blk::VirtioBlock, gpu::virtio_gpu::VirtioGpu, input::virtio_input::VirtioInput}, mm::{address::PhysAddr, frame_allocator::{frame_alloc, frame_dealloc, FrameTracker}, memset::KERN_SPACE, pagetab::PageTab}, sync::UThrCell};

lazy_static! {
    static ref VIRTIO_REGDEV: UThrCell<Vec<Option<Arc<dyn Device>>>> = unsafe {
        UThrCell::new(Vec::new())
    };
}

pub trait Device: Sync + Send + Any {
    fn handle_irq(&self);
}

pub fn device_init() {
    use riscv::register::sie;
    let mut plic = unsafe { Plic::new(VIRT_PLIC) };
    let hart_id = 0usize;
    let smod = IntrTargetPriority::Supervisor;
    let mmod = IntrTargetPriority::Machine;
    plic.set_threshold(hart_id, smod, 0);
    plic.set_threshold(hart_id, mmod, 1);
    // UART: 10
    plic.enable(hart_id, smod, 10);
    plic.set_priority(10, 1);
    SERIAL_DEV.init();
    // VIRTIO
    for i in 0..8 {
        let addr = VIRT_MMIO1 + (i * 0x1000);
        let header = NonNull::new(addr as *mut VirtIOHeader).unwrap();
        VIRTIO_REGDEV.get_refmut().push(None);
        if let Ok(transport) = unsafe { MmioTransport::new(header) } {
            info!(
                "Detected virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}, MMIO addr: {:#X}",
                transport.vendor_id(),
                transport.device_type(),
                transport.version(),
                addr
            );
            match transport.device_type() {
                DeviceType::Block => {
                    plic.enable(hart_id, smod, i + 1);
                    plic.set_priority(i + 1, 1);
                    let block_dev = Arc::new(VirtioBlock::new(transport));
                    *BLOCK_DEV.get_refmut() = Some(block_dev.clone());
                    VIRTIO_REGDEV.get_refmut()[i] = Some(block_dev.clone());
                }
                DeviceType::Input => {
                    plic.enable(hart_id, smod, i + 1);
                    plic.set_priority(i + 1, 1);
                    let input_dev = Arc::new(VirtioInput::new(transport));
                    INPUT_DEV.get_refmut().push(input_dev.clone());
                    VIRTIO_REGDEV.get_refmut()[i] = Some(input_dev.clone());
                }
                DeviceType::GPU => {
                    *GPU_DEV.get_refmut() = Some(Arc::new(VirtioGpu::new(transport)))
                }
                t => {
                    warn!("Unsupported device type {:?}", t);
                }
            }
        }
    }
    unsafe { sie::set_sext(); }
}

pub fn irq_handler() {
    debug!("IRQ Handler");
    let mut plic = unsafe { Plic::new(VIRT_PLIC) };
    let intr_src_id = plic.claim(0, IntrTargetPriority::Supervisor);
    if intr_src_id == 10 {
        SERIAL_DEV.handle_irq();
    } else {
        if let Some(dev) = VIRTIO_REGDEV.get_refmut()
            .get(intr_src_id as usize - 1)
            .expect("Unsupported IRQ")
        {
            dev.handle_irq();
        } else {
            panic!("Unsupported IRQ {}", intr_src_id);
        }
    }
    plic.complete(0, IntrTargetPriority::Supervisor, intr_src_id);
}

lazy_static! {
    static ref QUEUE_FRAMES: UThrCell<Vec<FrameTracker>> = unsafe { UThrCell::new(Vec::new()) };
}

pub struct VirtioHal;

unsafe impl Hal for VirtioHal {
    fn dma_alloc(pages: usize, _direction: virtio_drivers::BufferDirection) -> (virtio_drivers::PhysAddr, core::ptr::NonNull<u8>) {
        let frame = frame_alloc().unwrap();
        let ppn_base = frame.ppn;
        QUEUE_FRAMES.get_refmut().push(frame);

        for i in 1..pages {
            let frame = frame_alloc().unwrap();
            assert_eq!(frame.ppn.0, ppn_base.0 + i);
            QUEUE_FRAMES.get_refmut().push(frame);
        }
        let pa = PhysAddr::from(ppn_base).0;
        let ptr = NonNull::new(pa as _).unwrap();
        (pa, ptr)
    }

    unsafe fn dma_dealloc(paddr: virtio_drivers::PhysAddr, _vaddr: core::ptr::NonNull<u8>, pages: usize) -> i32 {
        let mut ppn_base = PhysAddr::from(paddr).ppn_floor();
        for _ in 0..pages {
            frame_dealloc(ppn_base);
            ppn_base.0 += 1;
        }
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: virtio_drivers::PhysAddr, _size: usize) -> core::ptr::NonNull<u8> {
        NonNull::new_unchecked(paddr as _)
    }

    unsafe fn share(buffer: core::ptr::NonNull<[u8]>, _direction: virtio_drivers::BufferDirection) -> virtio_drivers::PhysAddr {
        let va = buffer.as_ptr() as *mut u8 as usize;
        PageTab::from_token(KERN_SPACE.get_refmut().get_atp_token())
            .trans_va(va.into()).unwrap().0
    }

    unsafe fn unshare(_paddr: virtio_drivers::PhysAddr, _buffer: core::ptr::NonNull<[u8]>, _direction: virtio_drivers::BufferDirection) {
    }
}