use alloc::sync::{Arc, Weak};

use crate::{config::{ADDR_TRAPCONTEXT, PAGE_SIZE, USER_STACK_SIZE}, mm::{address::{PhysPageNum, VirtAddr}, memarea::MapPermission}};

use super::proc::ProcControlBlock;

pub struct TaskUserRes {
    pub tid: usize,
    pub ustack_base: usize,
    pub proc: Weak<ProcControlBlock>,
}

impl TaskUserRes {
    pub fn new(
        proc: Arc<ProcControlBlock>,
        ustack_base: usize,
        is_alloc_user_res: bool
    ) -> Self {
        let tid = proc.get_mutpart().alloc_tid();
        let ret = Self {
            tid,
            ustack_base,
            proc: Arc::downgrade(&proc)
        };
        if is_alloc_user_res {
            ret.alloc_user_res();
        }
        ret
    }

    pub fn alloc_user_res(&self) {
        let proc = self.proc.upgrade().unwrap();
        let mut inner = proc.get_mutpart();

        let ustack_bottom: VirtAddr = (self.ustack_base + self.tid * (PAGE_SIZE + USER_STACK_SIZE)).into();
        let ustack_top: VirtAddr = (ustack_bottom.0 + USER_STACK_SIZE).into();
        inner.memset.insert_framed_area(
            ustack_bottom..ustack_top,
            MapPermission::R | MapPermission::W | MapPermission::U
        );

        let trap_cx_bottom: VirtAddr = (ADDR_TRAPCONTEXT - self.tid * PAGE_SIZE).into();
        let trap_cx_top: VirtAddr = (trap_cx_bottom.0 + PAGE_SIZE).into();
        inner.memset.insert_framed_area(
            trap_cx_bottom..trap_cx_top,
            MapPermission::R | MapPermission::W
        );
    }

    fn dealloc_user_res(&self) {
        let proc = self.proc.upgrade().unwrap();
        let mut inner = proc.get_mutpart();

        let ustack_bottom: VirtAddr = (self.ustack_base + self.tid * (PAGE_SIZE + USER_STACK_SIZE)).into();
        inner.memset.del_area_by_start_vpn(ustack_bottom.vpn_floor());

        let trap_cx_bottom: VirtAddr = (ADDR_TRAPCONTEXT - self.tid * PAGE_SIZE).into();
        inner.memset.del_area_by_start_vpn(trap_cx_bottom.vpn_floor());
    }
    fn dealloc_tid(&self) {
        let proc = self.proc.upgrade().unwrap();
        let mut inner = proc.get_mutpart();
        inner.dealloc_tid(self.tid);
    }

    pub fn trap_cx_ppn(&self) -> PhysPageNum {
        let proc = self.proc.upgrade().unwrap();
        let inner = proc.get_mutpart();
        let trap_cx_bottom: VirtAddr = (ADDR_TRAPCONTEXT - self.tid * PAGE_SIZE).into();
        inner.memset.find(trap_cx_bottom.vpn_floor()).unwrap().ppn()
    }

    pub fn trap_cx_va(&self) -> usize {
        ADDR_TRAPCONTEXT - self.tid * PAGE_SIZE
    }

    pub fn ustack_top(&self) -> usize {
        self.ustack_base + self.tid * (USER_STACK_SIZE + PAGE_SIZE) + USER_STACK_SIZE
    }
}

impl Drop for TaskUserRes {
    fn drop(&mut self) {
        self.dealloc_tid();
        self.dealloc_user_res();
    }
}

