use crate::syscall::{sys_write, sys_read};
use core::fmt::{self, Write};

const COUT: usize = 1;
const CIN: usize = 0;
struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s:&str) -> fmt::Result {
        sys_write(COUT, s.as_bytes());
        Ok(())
    }
}

pub fn getchar() -> u8 {
    let mut c = [0u8; 1];
    sys_read(CIN, &mut c);
    c[0]
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    };
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    };
}