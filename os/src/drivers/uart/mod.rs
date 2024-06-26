use core::any::Any;

use alloc::sync::Arc;
use lazy_static::lazy_static;
use ns16550a::NS16550a;

use crate::config::VIRT_UART;

mod ns16550a;


pub trait SerialDevice: Send + Sync + Any {
    fn init(&self);
    fn read(&self) -> u8;
    fn write(&self, ch: u8);
    fn handle_irq(&self);
}

type SerialDeviceImpl = NS16550a<VIRT_UART>;

lazy_static! {
    pub static ref SERIAL_DEV: Arc<dyn SerialDevice> = {
        Arc::new(SerialDeviceImpl::new())
    };
}