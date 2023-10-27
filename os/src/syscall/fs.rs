use core::str::from_utf8;

use crate::{mm::pagetab::PageTab, task::{processor::curr_atp_token, suspend_curr_task}, sbi::cgetchar};

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

const FD_CIN: usize = 0;
pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_CIN => {
            assert_eq!(len, 1, "Only support len = 1 in sys_read");
            let mut c: usize;
            loop {
                c = cgetchar();
                if c == 0 {
                    suspend_curr_task();
                    continue;
                } else {
                    break;
                }
            }
            let mut buffers = PageTab::from_token(curr_atp_token()).trans_bytes_buffer(buf, len);
            unsafe {
                buffers[0].as_mut_ptr().write_volatile(c as u8);
            }
            1
        }
        _ => {
            panic!("Unsupported fd in sys_read")
        }
    }
}