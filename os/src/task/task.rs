use core::cell::RefMut;

use alloc::{sync::{Weak, Arc}, vec::Vec};
use log::debug;

use crate::{
    mm::{
        memset::{MemSet, KERN_SPACE},
        address::{PhysPageNum, VirtAddr},
    },
    config::ADDR_TRAPCONTEXT,
    trap::{context::TrapContext, trap_handler}, uthr::UThrCell,
};

use super::{context::TaskContext, kern_stack::KernStack, pid::{PidHandle, pid_alloc}};

#[derive(Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie
}

pub struct ProcControlBlock {
    pub pid: PidHandle,
    pub kern_stack: KernStack,
    mut_part: UThrCell<PCBMut>
}
impl ProcControlBlock {
    pub fn get_mutpart(&self) -> RefMut<'_, PCBMut> {
        self.mut_part.get_refmut()
    }
    pub fn getpid(&self) -> usize {
        *self.pid
    }
    pub fn new(elf_data: &[u8]) -> Self {
        let (memset, user_sp, entry_point) = MemSet::from_elf(elf_data);
        debug!("elf data parsed successfully");
        let trap_cx_ppn = memset
            .find(VirtAddr::from(ADDR_TRAPCONTEXT).into())
            .unwrap()
            .ppn();
        let pid = pid_alloc();
        let kern_stack = KernStack::new(&pid);
        debug!("Kernel stack dispatched");
        let kern_stack_top = kern_stack.get_top();
        let tcb = Self {
            pid,
            kern_stack,
            mut_part: unsafe {
                UThrCell::new(PCBMut {
                    trap_cx_ppn,
                    base_size: user_sp,
                    task_cx: TaskContext::to_trap_ret(kern_stack_top),
                    task_status: TaskStatus::Ready,
                    memset,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0
                })
            }
        };
        let trap_cx: &mut TrapContext = trap_cx_ppn.get_mut();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERN_SPACE.get_refmut().get_atp_token(),
            kern_stack_top,
            trap_handler as usize
        );
        tcb
    }
    pub fn fork(self: &Arc<ProcControlBlock>) -> Arc<ProcControlBlock> {
        let mut par_mut = self.get_mutpart();
        let memset = par_mut.memset.clone();
        let trap_cx_ppn = memset.find(VirtAddr::from(ADDR_TRAPCONTEXT).into()).unwrap().ppn();
        let pid = pid_alloc();
        let kern_stack = KernStack::new(&pid);
        let kern_sp = kern_stack.get_top();
        let pcb = Arc::new(Self {
            pid,
            kern_stack,
            mut_part: unsafe {
                UThrCell::new(PCBMut {
                    trap_cx_ppn,
                    base_size: par_mut.base_size,
                    task_cx: TaskContext::to_trap_ret(kern_sp),
                    task_status: TaskStatus::Ready,
                    memset,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0
                })
            }
        });
        par_mut.children.push(pcb.clone());
        let trap_cx = pcb.get_mutpart().get_trap_cx();
        trap_cx.kern_sp = kern_sp;
        pcb
    }
    pub fn exec(&self, elf_data: &[u8]) {
        let (memset, user_sp, entry_pt) = MemSet::from_elf(elf_data);
        let trap_cx_ppn = memset.find(VirtAddr::from(ADDR_TRAPCONTEXT).vpn_floor()).unwrap().ppn();
        let mut self_mut = self.get_mutpart();
        self_mut.memset = memset;
        self_mut.trap_cx_ppn = trap_cx_ppn;
        self_mut.base_size = user_sp;
        let trap_cx = trap_cx_ppn.get_mut();
        *trap_cx = TrapContext::app_init_context(
            entry_pt,
            user_sp,
            KERN_SPACE.get_refmut().get_atp_token(),
            self.kern_stack.get_top(),
            trap_handler as usize
        );
    }
}

pub struct PCBMut {
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub memset: MemSet,
    pub parent: Option<Weak<ProcControlBlock>>,
    pub children: Vec<Arc<ProcControlBlock>>,
    pub exit_code: i32
}
impl PCBMut {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_atp_token(&self) -> usize {
        self.memset.get_atp_token()
    }
    pub fn is_zombie(&self) -> bool {
        self.task_status == TaskStatus::Zombie
    }
}