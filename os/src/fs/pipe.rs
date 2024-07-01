use alloc::sync::{Arc, Weak};
use spin::Mutex;

use crate::task::suspend_curr_task;

use super::File;

pub struct Pipe {
    readable: bool,
    writable: bool,
    buffer: Arc<Mutex<RingBuffer>>
}

const BUFFER_SIZE: usize = 32;

#[derive(Clone, Copy, PartialEq)]
enum RingBufferStat {
    Full,
    Empty,
    Normal
}
struct RingBuffer {
    arr: [u8; BUFFER_SIZE],
    head: usize,
    tail: usize,
    stat: RingBufferStat,
    writer: Option<Weak<Pipe>>
}



impl Pipe {
    fn reader_with_buffer(buffer: &Arc<Mutex<RingBuffer>>) -> Self {
        Self {
            readable: true,
            writable: false,
            buffer: buffer.clone()
        }
    }

    fn writer_with_buffer(buffer: &Arc<Mutex<RingBuffer>>) -> Self {
        Self {
            readable: false,
            writable: true,
            buffer: buffer.clone()
        }
    }

    // Return (reader, writer)
    pub fn make_pipe() -> (Arc<Pipe>, Arc<Pipe>) {
        let buffer = Arc::new(Mutex::new(
            RingBuffer::new()
        ));
        let reader = Arc::new(
            Pipe::reader_with_buffer(&buffer)
        );
        let writer = Arc::new(
            Pipe::writer_with_buffer(&buffer)
        );
        buffer.lock().set_writer(&writer);
        (reader, writer)
    }
}

impl File for Pipe {
    fn readable(&self) -> bool { self.readable }

    fn writable(&self) -> bool { self.writable }

    fn seekable(&self) -> bool { false }

    fn read(&self, buf: crate::mm::pagetab::UserBuffer) -> usize {
        assert!(self.readable);
        let len_buf = buf.len();
        let mut buf_iter = buf.into_iter();
        let mut cnt_read = 0usize;
        loop {
            let mut ringbuf = self.buffer.lock();
            let len_ringbuf = ringbuf.bytes_contain();
            if len_ringbuf == 0 {
                if ringbuf.all_writer_closed() {
                    return cnt_read;
                }
                drop(ringbuf);
                suspend_curr_task();
                continue;
            }
            for _ in 0..len_ringbuf {
                if let Some(mem_ref) = buf_iter.next() {
                    unsafe {
                        *mem_ref = ringbuf.read_byte();
                    }
                    cnt_read += 1;
                    if cnt_read == len_buf {
                        return len_buf;
                    }
                } else {
                    return cnt_read;
                }
            }
        }
    }

    fn write(&self, buf: crate::mm::pagetab::UserBuffer) -> usize {
        assert!(self.writable);
        let len_buf = buf.len();
        let mut buf_iter = buf.into_iter();
        let mut cnt_writ = 0usize;
        loop {
            let mut ringbuf = self.buffer.lock();
            let len_ringbuf = ringbuf.bytes_contain();
            if len_ringbuf == BUFFER_SIZE {
                drop(ringbuf);
                suspend_curr_task();
                continue;
            }
            for _ in len_ringbuf..BUFFER_SIZE {
                if let Some(mem_ref) = buf_iter.next() {
                    unsafe {
                        ringbuf.write_byte(*mem_ref);
                    }
                    cnt_writ += 1;
                    if cnt_writ == len_buf {
                        return len_buf;
                    }
                } else {
                    return cnt_writ;
                }
            }
        }
    }

    fn seek(&self, _offset: isize, _whence: usize) {
        panic!("Unable to seek");
    }
}

impl RingBuffer {
    pub fn new() -> Self {
        Self {
            arr: [0; BUFFER_SIZE],
            head: 0,
            tail: 0,
            stat: RingBufferStat::Empty,
            writer: None
        }
    }

    pub fn set_writer(&mut self, writer: &Arc<Pipe>) {
        self.writer = Some(Arc::downgrade(writer));
    }

    pub fn read_byte(&mut self) -> u8 {
        assert!(self.stat != RingBufferStat::Empty);
        self.stat = RingBufferStat::Normal;
        let b = self.arr[self.head];
        self.head = (self.head + 1) & (BUFFER_SIZE - 1);
        if self.head == self.tail {
            self.stat = RingBufferStat::Empty;
        }
        b
    }

    pub fn write_byte(&mut self, b: u8) {
        assert!(self.stat != RingBufferStat::Full);
        self.stat = RingBufferStat::Normal;
        self.arr[self.tail] = b;
        self.tail = (self.tail + 1) & (BUFFER_SIZE - 1);
        if self.head == self.tail {
            self.stat = RingBufferStat::Full;
        }
    }

    pub fn bytes_contain(&self) -> usize {
        if self.stat == RingBufferStat::Empty {
            0
        } else {
            ((BUFFER_SIZE - 1 - self.head + self.tail) & (BUFFER_SIZE - 1)) + 1
        }
    }

    pub fn all_writer_closed(&self) -> bool {
        self.writer.as_ref().unwrap().upgrade().is_none()
    }
}
