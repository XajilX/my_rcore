use super::SerialDevice;
use crate::drivers::Device;
use crate::sync::{CondVar, UThrCell};
use crate::task::processor::schedule;
use alloc::collections::VecDeque;
use bitflags::*;
use log::debug;
use volatile::VolatileFieldAccess;
use volatile::access::{ReadOnly, ReadWrite, WriteOnly};

bitflags! {
    /// InterruptEnableRegister
    pub struct IER: u8 {
        const RX_AVAILABLE = 1 << 0;
        const TX_EMPTY = 1 << 1;
    }

    /// LineStatusRegister
    pub struct LSR: u8 {
        const DATA_AVAILABLE = 1 << 0;
        const THR_EMPTY = 1 << 5;
    }

    /// Model Control Register
    pub struct MCR: u8 {
        const DATA_TERMINAL_READY = 1 << 0;
        const REQUEST_TO_SEND = 1 << 1;
        const AUX_OUTPUT1 = 1 << 2;
        const AUX_OUTPUT2 = 1 << 3;
    }
}

#[repr(C)]
#[allow(dead_code)]
#[derive(VolatileFieldAccess)]
struct ReadRegs {
    /// receiver buffer register
    #[access(ReadOnly)]
    pub rbr: u8,
    /// interrupt enable register
    #[access(ReadWrite)]
    pub ier: IER,
    /// interrupt identification register
    #[access(ReadOnly)]
    pub iir: u8,
    /// line control register
    #[access(ReadWrite)]
    pub lcr: u8,
    /// model control register
    #[access(ReadWrite)]
    pub mcr: MCR,
    /// line status register
    #[access(ReadOnly)]
    pub lsr: LSR,
    /// ignore MSR
    #[access(ReadOnly)]
    _padding1: u8,
    /// ignore SCR
    #[access(ReadOnly)]
    _padding2: u8,
}

#[repr(C)]
#[allow(dead_code)]
#[derive(VolatileFieldAccess)]
struct WriteRegs {
    /// transmitter holding register
    #[access(WriteOnly)]
    pub thr: u8,
    /// interrupt enable register
    #[access(ReadWrite)]
    pub ier: IER,
    /// ignore FCR
    #[access(ReadOnly)]
    _padding0: u8,
    /// line control register
    #[access(ReadWrite)]
    pub lcr: u8,
    /// modem control register
    #[access(ReadWrite)]
    pub mcr: MCR,
    /// line status register
    #[access(ReadOnly)]
    pub lsr: LSR,
    /// ignore other registers
    #[access(ReadOnly)]
    _padding1: u16,
}

pub struct NS16550aRaw {
    base_addr: usize,
}

impl NS16550aRaw {
    fn read_regs(&mut self) -> &mut ReadRegs {
        unsafe { &mut *(self.base_addr as *mut ReadRegs) }
    }

    fn write_regs(&mut self) -> &mut WriteRegs {
        unsafe { &mut *(self.base_addr as *mut WriteRegs) }
    }

    pub fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    pub fn init(&mut self) {
        let read_end = self.read_regs();
        let mcr = MCR::DATA_TERMINAL_READY |
            MCR::REQUEST_TO_SEND |
            MCR::AUX_OUTPUT2;
        read_end.mcr = mcr;
        let ier = IER::RX_AVAILABLE;
        read_end.ier = ier;
    }

    pub fn read(&mut self) -> Option<u8> {
        let read_end = self.read_regs();
        let lsr = read_end.lsr;
        if lsr.contains(LSR::DATA_AVAILABLE) {
            Some(read_end.rbr)
        } else {
            None
        }
    }

    pub fn write(&mut self, ch: u8) {
        let write_end = self.write_regs();
        loop {
            if write_end.lsr.contains(LSR::THR_EMPTY) {
                write_end.thr = ch;
                break;
            }
        }
    }
}

struct NS16550aInner {
    ns16550a: NS16550aRaw,
    read_buffer: VecDeque<u8>,
}

pub struct NS16550a<const BASE_ADDR: usize> {
    inner: UThrCell<NS16550aInner>,
    condvar: CondVar,
}

impl<const BASE_ADDR: usize> NS16550a<BASE_ADDR> {
    pub fn new() -> Self {
        let inner = NS16550aInner {
            ns16550a: NS16550aRaw::new(BASE_ADDR),
            read_buffer: VecDeque::new(),
        };
        //inner.ns16550a.init();
        Self {
            inner: unsafe { UThrCell::new(inner) },
            condvar: CondVar::new(),
        }
    }

    /*
    pub fn is_empty(&self) -> bool {
        self.inner
            .then(|inner| inner.read_buffer.is_empty())
    }
    */
}

impl<const BASE_ADDR: usize> SerialDevice for NS16550a<BASE_ADDR> {
    fn init(&self) {
        let mut inner = self.inner.get_refmut();
        inner.ns16550a.init();
        drop(inner);
    }

    fn read(&self) -> u8 {
        loop {
            let mut inner = self.inner.get_refmut();
            if let Some(ch) = inner.read_buffer.pop_front() {
                return ch;
            } else {
                let task_cx = self.condvar.wait_without_schd();
                drop(inner);
                schedule(task_cx);
            }
        }
    }
    fn write(&self, ch: u8) {
        let mut inner = self.inner.get_refmut();
        inner.ns16550a.write(ch);
    }
}

impl<const BASE_ADDR: usize> Device for NS16550a<BASE_ADDR> {
    fn handle_irq(&self) {
        debug!("Serial Device Handling IRQ");
        let mut count = 0;
        self.inner.then(|inner| {
            while let Some(ch) = inner.ns16550a.read() {
                count += 1;
                inner.read_buffer.push_back(ch);
            }
        });
        if count > 0 {
            self.condvar.signal();
        }
    }
}
