#[inline(always)]

pub fn cputchar(c: usize) {
    sbi_rt::legacy::console_putchar(c);
}

pub fn shutdown() -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown};
    system_reset(Shutdown, NoReason);
    unreachable!()
}