use core::{ops::Range, mem::size_of};

use crate::{config::*, mm::{address::VirtAddr, memarea::MapPermission, memset::KERN_SPACE}, sync::UThrCell, task::allocator::IdAlloc};

use lazy_static::lazy_static;

pub struct KernStack(pub usize);

lazy_static! {
    static ref KSTACK_ALLOC: UThrCell<IdAlloc> = unsafe {
        UThrCell::new(IdAlloc::new())
    };
}
pub fn kstack_alloc() -> KernStack {
    let id = KSTACK_ALLOC.get_refmut().alloc();
    let kstack_range = kern_stack_range(id);
    KERN_SPACE.get_refmut().insert_framed_area(
        kstack_range.start.into()..kstack_range.end.into(),
        MapPermission::R | MapPermission::W
    );
    KernStack(id)
}

impl KernStack {
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
        let ks_range = kern_stack_range(self.0);
        ks_range.end
    }
}
impl Drop for KernStack {
    fn drop(&mut self) {
        let ks_range = kern_stack_range(self.0);
        let ks_bottom_va: VirtAddr = ks_range.start.into();
        KERN_SPACE.get_refmut().del_area_by_start_vpn(ks_bottom_va.into());
        KSTACK_ALLOC.get_refmut().dealloc(self.0);
    }
}

fn kern_stack_range(id: usize) -> Range<usize> {
    let top = ADDR_TRAMPOLINE - id * (KERN_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERN_STACK_SIZE;
    bottom..top
}
