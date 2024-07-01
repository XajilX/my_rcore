#![no_std]
#![no_main]

extern crate user;

use embedded_graphics::pixelcolor::Rgb888;
use user::Display;

#[no_mangle]
pub fn main() -> i32 {
    let disp = Display::new();
    let res = disp.resolution();
    disp.render(|x, y| {
        let xf = x * 255 / res.0;
        let yf = y * 255 / res.1;
        Rgb888::new(xf as u8, yf as u8, 0)
    });
    0
}
