use crate::{fs::inode::{OSInode, OpenFlag}, mm::pagetab::{PageTab, UserBuffer},  task::processor::{curr_atp_token, curr_proc}};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = curr_atp_token();
    let proc = curr_proc().unwrap();
    let inner = proc.get_mutpart();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        drop(inner);
        file.write(UserBuffer::from(
            PageTab::from_token(token).trans_bytes_buffer(buf, len)
        )) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = curr_atp_token();
    let proc = curr_proc().unwrap();
    let inner = proc.get_mutpart();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        drop(inner);
        file.read(UserBuffer::from(
            PageTab::from_token(token).trans_bytes_buffer(buf, len)
        )) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = curr_proc().unwrap();
    let token = curr_atp_token();
    let path = PageTab::from_token(token).trans_cstr(path);
    if let Some(inode) = OSInode::open(path.as_str(),
        OpenFlag::from_bits(flags).unwrap()
    ) {
        let mut inner = task.get_mutpart();
        let pos = inner.alloc_newfd();
        inner.fd_table[pos] = Some(inode);
        pos as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = curr_proc().unwrap();
    let mut inner = task.get_mutpart();
    if  fd >= inner.fd_table.len() ||
        inner.fd_table[fd].is_none() {
        -1
    } else {
        inner.fd_table[fd].take();
        0
    }
}