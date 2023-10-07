use core::arch::asm;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe { asm!(
        "ecall",
        inlateout("x10") arg[0] => ret,
        in("x11") arg[1],
        in("x12") arg[2],
        in("x17") id
    )};
    ret
}

const SYSCALL_WRITE = 64usize;
const SYSCALL_EXIT = 93usize;

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(xstate: i32) -> isize {
    syscall(SYSCALL_EXIT, [xstate as usize, 0, 0])
}