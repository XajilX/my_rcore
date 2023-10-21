pub mod switch;
pub mod context;
pub mod task;

use alloc::vec::Vec;
use lazy_static::lazy_static;
use log::{info, debug};
use crate::loader::{get_num_app, get_app_data};
use crate::task::context::TaskContext;
use crate::task::task::TaskStatus;
use crate::trap::context::TrapContext;
use crate::uthr::UThrCell;
use self::switch::__switch;
use self::task::TaskControlBlock;

pub struct TaskMan {
    num_app: usize,
    mut_part: UThrCell<TaskManMutPart>
}
struct TaskManMutPart {
    curr_task: usize,
    tcb: Vec<TaskControlBlock>
}
impl TaskMan {
    fn get_curr_atp_token(&self) -> usize {
        let mut_part = self.mut_part.get_refmut();
        let curr = mut_part.curr_task;
        mut_part.tcb[curr].memset.get_atp_token()
    }
    fn get_curr_trap_cx(&self) -> &mut TrapContext {
        let mut_part = self.mut_part.get_refmut();
        let curr = mut_part.curr_task;
        mut_part.tcb[curr].get_trap_cx()
    }
    fn mark_curr_task(&self, mark: TaskStatus) {
        let mut mut_part = self.mut_part.get_refmut();
        let curr = mut_part.curr_task;
        mut_part.tcb[curr].task_status = mark;
    }
    fn find_next(&self) -> Option<usize> {
        let mut_part = self.mut_part.get_refmut();
        let curr = mut_part.curr_task;
        (curr + 1..curr + self.num_app + 1)
            .map(|id| { id % self.num_app })
            .find(|id| {
                mut_part.tcb[*id].task_status == TaskStatus::Ready
            })
    }
    fn mem_recycle_curr(&self) {
        let mut mut_part = self.mut_part.get_refmut();
        let curr = mut_part.curr_task;
        mut_part.tcb[curr].memset.mem_recycle()
    }
    fn run_next(&self) {
        if let Some(next_id) = self.find_next() {
            info!("[kernel] Next app id: {next_id}");
            let mut mut_part = self.mut_part.get_refmut();
            let curr = mut_part.curr_task;
            mut_part.tcb[next_id].task_status = TaskStatus::Running;
            let curr_task_cx_ptr = &mut mut_part.tcb[curr].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &mut_part.tcb[next_id].task_cx as *const TaskContext;
            mut_part.curr_task = next_id;
            drop(mut_part);
            unsafe {
                __switch(curr_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            panic!("All apps completed!");
        }
    }
    fn run_first(&self) -> ! {
        debug!("Now run first task");
        let mut mut_part = self.mut_part.get_refmut();
        mut_part.tcb[0].task_status = TaskStatus::Running;
        let next_task_cx_ptr = &mut_part.tcb[0].task_cx as *const TaskContext;
        let mut _plh = TaskContext::zeros();    //  placeholder
        drop(mut_part);
        unsafe {
            __switch(
                &mut _plh as *mut TaskContext,
                next_task_cx_ptr
            );
        }
        unreachable!()
    }
}

lazy_static! {
    pub static ref TASK_MAN: TaskMan = {
        let num_app = get_num_app();
        println!("[kernel] num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        debug!("Now load appdata");
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(
                get_app_data(i), i
            ));
        }
        TaskMan {
            num_app,
            mut_part: unsafe {
                UThrCell::new(TaskManMutPart {
                    curr_task: 0,
                    tcb: tasks
                })
            }
        }
    };
}

pub fn suspend_curr_task() {
    TASK_MAN.mark_curr_task(TaskStatus::Ready);
    TASK_MAN.run_next();
}

pub fn exit_curr_task() {
    TASK_MAN.mark_curr_task(TaskStatus::Exited);
    TASK_MAN.mem_recycle_curr();
    TASK_MAN.run_next();
}

pub fn run_first_task() {
    TASK_MAN.run_first();
}
pub fn curr_atp_token() -> usize {
    TASK_MAN.get_curr_atp_token()
}
pub fn curr_trap_cx() -> &'static mut TrapContext {
    TASK_MAN.get_curr_trap_cx()
}