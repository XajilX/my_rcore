use core::{convert::Infallible, mem::size_of, slice::from_raw_parts};

use embedded_graphics::{draw_target::DrawTarget, geometry::{OriginDimensions, Size}, pixelcolor::{Rgb888, RgbColor}};

use crate::{get_fbfd, get_resolution, seek, write};

pub struct Display {
    fd: isize,
    res: Size
}

impl Display {
    pub fn new() -> Self {
        let (xres, yres) = get_resolution();
        let fd = get_fbfd();
        Self {
            fd,
            res: Size::new(xres, yres)
        }
    }
    pub fn render(&self, mut f: impl FnMut(u32, u32) -> Rgb888) {
        let mut buf = [Rgb888::new(0, 0, 0); 512];
        let mut idx = 0;
        for y in 0..self.res.height {
            for x in 0..self.res.width {
                buf[idx] = f(x, y);
                idx += 1;
                if idx >= 512 {
                    idx = 0;
                    let ptr = buf.as_ptr() as *const u8;
                    let len = buf.len() * size_of::<Rgb888>();
                    let raw_buf = unsafe { from_raw_parts(ptr, len) };
                    write(self.fd as usize, raw_buf);
                }
            }
        }
        let ptr = buf.as_ptr() as *const u8;
        let len = idx * size_of::<Rgb888>();
        let raw_buf = unsafe { from_raw_parts(ptr, len) };
        write(self.fd as usize, raw_buf);
    }
    pub fn resolution(&self) -> (u32, u32) {
        (self.res.width, self.res.height)
    }
    fn flush(&self) {
        let len = self.res.width * self.res.width * 4;
        seek(self.fd as usize, len as isize, 0);
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> embedded_graphics::prelude::Size {
        self.res
    }
}

impl DrawTarget for Display {
    type Color = Rgb888;

    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::prelude::Pixel<Self::Color>>
    {
        pixels.into_iter().for_each(|px| {
            let idx = (px.0.y * self.res.width as i32 + px.0.x) as isize;
            let buf = [px.1.b(), px.1.g(), px.1.r()];
            seek(self.fd as usize, idx * 4, 0);
            write(self.fd as usize, &buf);
        });
        self.flush();
        Ok(())
    }
}