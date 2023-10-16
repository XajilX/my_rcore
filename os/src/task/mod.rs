pub mod switch;
pub mod context;
pub mod task;

use lazy_static::lazy_static;
use log::info;
use crate::loader::{get_num_app, init_app_cx};
use crate::task::context::TaskContext;
use crate::task::task::TaskStatus;
use crate::uthr::UThrCell;
use crate::config::MAX_APP_NUM;
use self::switch::__switch;
use self::task::TaskControlBlock;

pub struct TaskMan {
    num_app: usize,
    mut_part: UThrCell<TaskManMutPart>
}
struct TaskManMutPart {
    curr_task: usize,
    tcb: [TaskControlBlock; MAX_APP_NUM]
}
impl TaskMan {
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
        let mut mut_part = self.mut_part.get_refmut();
        mut_part.tcb[0].task_status = TaskStatus::Running;
        let next_task_cx_ptr = &mut_part.tcb[0].task_cx as *const TaskContext;
        let mut _plh = TaskContext::zeros();
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
        let mut tasks = [
            TaskControlBlock {
                task_status: TaskStatus::Uninit,
                task_cx: TaskContext::zeros()
            };
            MAX_APP_NUM
        ];
        for i in 0..num_app {
            tasks[i].task_cx = TaskContext::to_restore(init_app_cx(i));
            tasks[i].task_status = TaskStatus::Ready;
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
    TASK_MAN.run_next();
}

pub fn run_first_task() {
    TASK_MAN.run_first();
}
