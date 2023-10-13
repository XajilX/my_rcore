use log::debug;

use crate::task::{mem_range_curr_task, exit_curr_task};

const FD_COUT: usize = 1;
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_COUT => {
            //write_mem_check(buf, len);
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            debug!("{:?}",slice);
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        },
        _ => {
            panic!("Unsupported fd in sys_write")
        }
    }
}

fn write_mem_check(buf: *const u8, len: usize) {
    let (buf_s, buf_e) = (buf as usize, buf as usize + len);
    let (prog_s, prog_e) = mem_range_curr_task();
    if  buf_s < prog_s || buf_s >= prog_e ||
        buf_e <= prog_s || buf_e > prog_e {
            println!("[kernel] Memory violation in sys_write, kernel execution");
            exit_curr_task()
        }
}
