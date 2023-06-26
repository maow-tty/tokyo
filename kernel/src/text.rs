use core::fmt;
use bitvec::macros::internal::funty::Fundamental;
use bitvec::prelude::*;
use crate::framebuffer::Rgb;
use crate::VIEW;

pub fn print_char(char: char, x: usize, y: usize, fore_color: Rgb, back_color: Rgb) {
    let idx = char as usize;
    if idx >= 128 { return }
    let chars = font8x8::legacy::BASIC_LEGACY[idx];

    let pixels = &mut [Rgb::new(0, 0, 0); 64];
    chars.iter()
        .flat_map(|char| char.view_bits::<Lsb0>().iter())
        .enumerate()
        .for_each(|(idx, bit)| {
            pixels[idx] = if bit.as_bool() { back_color } else { fore_color };
        });

    VIEW.lock().get_mut().unwrap().draw_quad(pixels, x * 8, y * 8, 8);
}

pub fn print_str(str: &str, x_offset: usize, y: usize, fore_color: Rgb, back_color: Rgb) {
    for (idx, char) in str.chars().enumerate() {
        print_char(char, idx + x_offset, y, fore_color, back_color);
    }
}

pub struct ErrorWriter {
    pos: usize
}

impl ErrorWriter {
    pub fn new() -> Self {
        Self { pos: 0 }
    }
}

impl fmt::Write for ErrorWriter {
    fn write_str(&mut self, str: &str) -> fmt::Result {
        print_str(str, self.pos, 0, Rgb::WHITE, Rgb::RED);
        self.pos += str.len();
        Ok(())
    }

    fn write_char(&mut self, char: char) -> fmt::Result {
        print_char(char, self.pos, 0, Rgb::WHITE, Rgb::RED);
        self.pos += 1;
        Ok(())
    }
}