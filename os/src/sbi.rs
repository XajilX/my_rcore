#[inline(always)]

#[allow(unused)]
pub fn cputchar(ch: char) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(ch as usize);
}

pub fn shutdown() -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown};
    system_reset(Shutdown, NoReason);
    unreachable!()
}

pub fn set_timer(timer: usize) {
    sbi_rt::set_timer(timer as _);
}