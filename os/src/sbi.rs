#[inline(always)]


pub fn shutdown() -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown};
    system_reset(Shutdown, NoReason);
    unreachable!()
}

pub fn set_timer(timer: usize) {
    sbi_rt::set_timer(timer as _);
}