#![no_std]
#![no_main]

extern crate user;

use embedded_graphics::pixelcolor::Rgb888;
use user::{get_inputevent, println, Display};
use virtio_input_decoder::{DecodeType, Key, KeyType};

#[no_mangle]
pub fn main() -> i32 {
    let disp = Display::new();
    let res = disp.resolution();
    loop {
        match get_inputevent().decode() {
            Some(DecodeType::Key(Key::A, KeyType::Press)) => {
                println!("Key pressed A");
                disp.render(|x, y| {
                    let xf = x * 255 / res.0;
                    let yf = y * 255 / res.1;
                    Rgb888::new(xf as u8, yf as u8, 0)
                });
            }
            Some(DecodeType::Key(Key::B, KeyType::Press)) => {
                println!("Key pressed B");
                disp.render(|x, y| {
                    let xf = x * 255 / res.0;
                    let yf = y * 255 / res.1;
                    Rgb888::new(xf as u8, 0, yf as u8)
                });
            }
            Some(DecodeType::Key(Key::C, KeyType::Press)) => {
                println!("Key pressed C");
                disp.render(|x, y| {
                    let xf = x * 255 / res.0;
                    let yf = y * 255 / res.1;
                    Rgb888::new(0, xf as u8, yf as u8)
                });
            }
            Some(DecodeType::Key(Key::D, KeyType::Press)) => {
                println!("Key pressed D");
                disp.render(|x, y| {
                    let xf = x * 255 / res.0;
                    let yf = y * 255 / res.1;
                    Rgb888::new(xf as u8, yf as u8, (xf + yf + 128) as u8)
                });
            }
            Some(DecodeType::Key(Key::E, KeyType::Press)) => {
                break;
            }
            _ => { continue; }
        }
    }
    0
}
