use core::panic::PanicInfo;

use crate::syscall::sys_exit;
use crate::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(loc) = info.location() {
        println!("Panicked at {}:{} {}", loc.file(), loc.line(), info.message().unwrap());
    } else {
        println!("Panicked: {}", info.message().unwrap());
    }
    sys_exit(1);
    loop{}
}