use core::panic::PanicInfo;
use crate::sbi::shutdown;
use log::error;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(loc) = info.location() {
        error!("Panicked at {}:{} {}", loc.file(), loc.line(), info.message().unwrap());
    } else {
        error!("Panicked: {}", info.message().unwrap());
    }
    shutdown()
}
