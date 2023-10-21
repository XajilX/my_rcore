use core::str::from_utf8;

use crate::{mm::pagetab::PageTab, task::curr_atp_token};

const FD_COUT: usize = 1;
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_COUT => {
            //write_mem_check(buf, len);
            let buffer = PageTab::from_token(curr_atp_token())
                .trans_bytes_buffer(buf, len);
            for slice in buffer {
                print!("{}", from_utf8(slice)
                    .expect("[kernel] Unparsable slice"))
            } 
            len as isize
        },
        _ => {
            panic!("Unsupported fd in sys_write")
        }
    }
}
