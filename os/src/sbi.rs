#[inline(always)]

pub fn cputchar(c: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

pub fn cgetchar() -> usize {
    #[allow(deprecated)]
    sbi_rt::legacy::console_getchar()
}

pub fn shutdown() -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown};
    system_reset(Shutdown, NoReason);
    unreachable!()
}

pub fn set_timer(timer: usize) {
    sbi_rt::set_timer(timer as _);
}