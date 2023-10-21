use log::debug;

use crate::{mm::{memset::{MemSet, KERN_SPACE}, address::{PhysPageNum, VirtAddr}, memarea::MapPermission}, config::ADDR_TRAPCONTEXT, loader::kern_stack_range, trap::{context::TrapContext, trap_handler}};

use super::context::TaskContext;

#[derive(Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited
}

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub memset: MemSet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize
}
impl TaskControlBlock {
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        let (memset, user_sp, entry_point) = MemSet::from_elf(elf_data);
        debug!("elf data parsed successfully");
        let trap_cx_ppn = memset
            .find(VirtAddr::from(ADDR_TRAPCONTEXT).into())
            .unwrap()
            .ppn();
        let kern_stack_range = kern_stack_range(app_id);
        KERN_SPACE.get_refmut().insert_framed_area(
            kern_stack_range.start.into()..kern_stack_range.end.into(),
            MapPermission::R | MapPermission::W
        );
        let tcb = Self {
            task_status: TaskStatus::Ready,
            task_cx: TaskContext::to_trap_ret(kern_stack_range.end),
            memset,
            trap_cx_ppn,
            base_size: user_sp
        };
        let trap_cx = tcb.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERN_SPACE.get_refmut().get_atp_token(),
            kern_stack_range.end,
            trap_handler as usize
        );
        tcb
    }
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
}