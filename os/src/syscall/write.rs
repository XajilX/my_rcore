use log::debug;

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
