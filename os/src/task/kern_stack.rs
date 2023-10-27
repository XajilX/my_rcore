use core::{ops::Range, mem::size_of};

use crate::{mm::{memset::KERN_SPACE, memarea::MapPermission, address::VirtAddr}, config::*};

use super::pid::PidHandle;

pub struct KernStack {
    pid: usize
}
impl KernStack {
    pub fn new(pid_handle: &PidHandle) -> Self {
        let ks_range = kern_stack_range(pid_handle.0);
        KERN_SPACE.get_refmut().insert_framed_area(
            ks_range.start.into()..ks_range.end.into(),
            MapPermission::R | MapPermission::W
        );
        KernStack { pid: pid_handle.0 }
    }
    #[allow(unused)]
    pub fn push_on_top<T>(&self, value: T) -> *mut T where
        T: Sized
    {
        let stack_top = self.get_top();
        let ptr_mut = (stack_top - size_of::<T>()) as *mut T;
        unsafe { *ptr_mut = value; }
        ptr_mut
    }
    pub fn get_top(&self) -> usize {
        let ks_range = kern_stack_range(self.pid);
        ks_range.end
    }
}
impl Drop for KernStack {
    fn drop(&mut self) {
        let ks_range = kern_stack_range(self.pid);
        let ks_bottom_va: VirtAddr = ks_range.start.into();
        KERN_SPACE.get_refmut().del_area_by_start_vpn(ks_bottom_va.into());
    }
}

pub fn kern_stack_range(pid: usize) -> Range<usize> {
    let top = ADDR_TRAMPOLINE - pid * (KERN_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERN_STACK_SIZE;
    bottom..top
}
