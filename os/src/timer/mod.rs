use riscv::register::time;

use crate::{sbi::set_timer, config::CLOCK_FREQ};

pub mod sleep;

const CLOCK_PER_TICK: usize = CLOCK_FREQ / 50; //10ms per tick
const CLOCK_PER_MILI: usize = CLOCK_FREQ / 1000;

pub fn get_time() -> usize {
    time::read()
}
pub fn get_time_ms() -> usize {
    time::read() / CLOCK_PER_MILI
}

pub fn set_trig() {
    set_timer(get_time() + CLOCK_PER_TICK);
}

