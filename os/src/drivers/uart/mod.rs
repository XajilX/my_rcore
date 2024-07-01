use alloc::sync::Arc;
use lazy_static::lazy_static;
use ns16550a::NS16550a;

use crate::config::VIRT_UART;

use super::Device;

mod ns16550a;


pub trait SerialDevice: Device {
    fn init(&self);
    fn read(&self) -> u8;
    fn write(&self, ch: u8);
}

type SerialDeviceImpl = NS16550a<VIRT_UART>;

lazy_static! {
    pub static ref SERIAL_DEV: Arc<dyn SerialDevice> = {
        Arc::new(SerialDeviceImpl::new())
    };
}