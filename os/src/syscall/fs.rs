use crate::{fs::{inode::{OSInode, OpenFlag}, pipe::Pipe}, mm::pagetab::{PageTab, UserBuffer},  task::{proc::PCBMut, processor::{curr_atp_token, curr_proc}}};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = curr_atp_token();
    let proc = curr_proc();
    let inner = proc.get_mutpart();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
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
    let proc = curr_proc();
    let inner = proc.get_mutpart();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        drop(inner);
        file.read(UserBuffer::from(
            PageTab::from_token(token).trans_bytes_buffer(buf, len)
        )) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = curr_proc();
    let token = curr_atp_token();
    let path = PageTab::from_token(token).trans_cstr(path);
    if let Some(inode) = OSInode::open(path.as_str(),
        OpenFlag::from_bits(flags).unwrap()
    ) {
        let mut inner = task.get_mutpart();
        let fd = PCBMut::alloc_new_id(&mut inner.fd_table);
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = curr_proc();
    let token = curr_atp_token();
    let mut inner = task.get_mutpart();
    let (pipe_read, pipe_writ) = Pipe::make_pipe();
    let read_fd = PCBMut::alloc_new_id(&mut inner.fd_table);
    inner.fd_table[read_fd] = Some(pipe_read);
    let writ_fd = PCBMut::alloc_new_id(&mut inner.fd_table);
    inner.fd_table[writ_fd] = Some(pipe_writ);
    unsafe {
        *(PageTab::from_token(token).trans_mut(pipe)) = read_fd;
        *(PageTab::from_token(token).trans_mut(pipe.add(1))) = writ_fd;
    }
    0
}

pub fn sys_close(fd: usize) -> isize {
    let task = curr_proc();
    let mut inner = task.get_mutpart();
    if  fd >= inner.fd_table.len() ||
        inner.fd_table[fd].is_none() {
        -1
    } else {
        inner.fd_table[fd].take();
        0
    }
}

pub fn sys_seek(fd: usize, offset: isize, whence: usize) -> isize {
    let task = curr_proc();
    let inner = task.get_mutpart();
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.seekable() {
            return -1;
        }
        drop(inner);
        file.seek(offset, whence);
        0
    } else {
        -1
    }
}

pub fn sys_dup(fd: usize) -> isize {
    let proc = curr_proc();
    let mut inner = proc.get_mutpart();
    if  fd >= inner.fd_table.len() ||
        inner.fd_table[fd].is_none() {
        -1
    } else {
        let newfd = PCBMut::alloc_new_id(&mut inner.fd_table);
        inner.fd_table[newfd] = Some(inner.fd_table[fd].as_ref().unwrap().clone());
        newfd as isize
    }
}