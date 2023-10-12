use crate::batch::run_app;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93usize;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id)
    }
}

const FD_COUT: usize = 1;
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_COUT => {
            write_mem_check(buf, len);
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        },
        _ => {
            panic!("Unsupported fd in sys_write")
        }
    }
}

use crate::batch::mem_range_curr_app;
fn write_mem_check(buf: *const u8, len: usize) {
    let (buf_s, buf_e) = (buf as usize, buf as usize + len);
    let (prog_s, prog_e) = mem_range_curr_app();
    if  buf_s < prog_s || buf_s >= prog_e ||
        buf_e <= prog_s || buf_e > prog_e {
            println!("[kernel] Memory violation in sys_write, kernel execution");
            run_app()
        }
}

pub fn sys_exit(xstate: i32) -> ! {
    println!("[kernel] Application exited with code {}", xstate);
    run_app()
}