use crate::{task::{suspend_curr_task, exit_curr_task}, timer::get_time_ms};

pub fn sys_yield() -> isize {
    suspend_curr_task();
    0
}

pub fn sys_exit(xstate: i32) -> ! {
    println!("[kernel] Application exited with code {}", xstate);
    exit_curr_task();
    unreachable!()
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

