use alloc::sync::{Arc, Weak};

use crate::sync::UThrRefMut;
use crate::{
    mm::address::PhysPageNum,
    trap::context::TrapContext,
    sync::UThrCell
};

use super::kern_stack::kstack_alloc;
use super::proc::ProcControlBlock;
use super::task_user_res::TaskUserRes;
use super::{context::TaskContext, kern_stack::KernStack};

#[derive(Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Blocked
}

pub struct TaskControlBlock {
    pub proc: Weak<ProcControlBlock>,
    pub kern_stack: KernStack,
    mut_part: UThrCell<TCBMut>
}
impl TaskControlBlock {
    pub fn new(
        proc: Arc<ProcControlBlock>,
        ustack_base: usize,
        is_alloc_user_res: bool
    ) -> Self {
        let res = TaskUserRes::new(proc.clone(), ustack_base, is_alloc_user_res);
        let trap_cx_ppn = res.trap_cx_ppn();
        let kern_stack = kstack_alloc();
        let kstack_top = kern_stack.get_top();
        Self {
            proc: Arc::downgrade(&proc),
            kern_stack,
            mut_part: unsafe {
                UThrCell::new(TCBMut {
                    res: Some(res),
                    trap_cx_ppn,
                    task_cx: TaskContext::to_trap_ret(kstack_top),
                    task_status: TaskStatus::Ready,
                    exit_code: None,
                })
            }
        }
    }
    pub fn get_mutpart(&self) -> UThrRefMut<'_, TCBMut> {
        self.mut_part.get_refmut()
    }
}

pub struct TCBMut {
    pub res: Option<TaskUserRes>,
    pub trap_cx_ppn: PhysPageNum,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub exit_code: Option<i32>
}
impl TCBMut {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
}