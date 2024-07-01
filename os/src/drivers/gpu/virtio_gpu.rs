use core::slice::from_raw_parts_mut;

use virtio_drivers::{device::gpu::VirtIOGpu, transport::mmio::MmioTransport};

use crate::{drivers::VirtioHal, sync::UThrCell};

use super::GpuDevice;

pub struct VirtioGpu {
    gpu: UThrCell<VirtIOGpu<VirtioHal, MmioTransport>>,
    fb: &'static [u8]
}

impl VirtioGpu {
    pub fn new(transport: MmioTransport) -> Self {
        unsafe {
            let mut gpu = VirtIOGpu::<VirtioHal,MmioTransport>::new(transport).unwrap();
            let fbuf = gpu.setup_framebuffer().unwrap();
            let (ptr, len) = (fbuf.as_mut_ptr(), fbuf.len());
            let fb = from_raw_parts_mut(ptr, len);

            let mut im_cur = [0u8; 64 * 64 * 4];
            for i in 0..64 * 64 {
                im_cur[i * 4]     = 0x88;
                im_cur[i * 4 + 1] = 0x88;
            }
            gpu.setup_cursor(im_cur.as_slice(), 50, 50, 50, 50).unwrap();
            Self {
                gpu: UThrCell::new(gpu),
                fb
            }
        }
    }
}

impl GpuDevice for VirtioGpu {
    fn get_framebuf(&self) -> &mut [u8] {
        unsafe {
            let ptr = self.fb.as_ptr() as *mut u8;
            from_raw_parts_mut(ptr, self.fb.len())
        }
    }

    fn flush(&self) {
        self.gpu.get_refmut().flush().unwrap()
    }

    fn resolution(&self) -> (u32, u32) {
        //self.gpu.get_refmut().resolution().unwrap()
        (1280, 800)
    }
}