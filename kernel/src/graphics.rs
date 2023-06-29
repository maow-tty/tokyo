use core::slice;
use bitvec::macros::internal::funty::Fundamental;
use bitvec::prelude::*;
use bootloader_api::info::FrameBufferInfo;
use line_drawing::{Bresenham, Point};
use spin::Mutex;
use x86_64::instructions::interrupts;

// TODO: Add support for framebuffers other than 24-bit BGR

static VIEW: Mutex<Option<FrameBufferView>> = Mutex::new(None);

pub struct FrameBufferView<'a> {
    info: FrameBufferInfo,
    buffer: &'a mut [u8]
}

impl<'a> FrameBufferView<'a> {
    pub fn new(info: FrameBufferInfo, buffer: &'a mut [u8]) -> Self {
        Self { info, buffer }
    }

    pub fn clear(&mut self, color: Rgb) {
        // chunk by stride (width + padding), split at width, discard padding
        self.buffer
            .chunks_exact_mut(self.info.stride * 3)
            .map(|dst| dst.split_at_mut(self.info.width * 3))
            .for_each(|(dst, _)| {
                for pixel in dst.chunks_exact_mut(3) {
                    pixel[0] = color.blue;
                    pixel[1] = color.green;
                    pixel[2] = color.red;
                }
            });
    }

    pub fn draw_line<P>(&mut self, start: P, end: P, color: Rgb)
    where
        P: Into<Point<isize>>
    {
        let row_length = self.info.stride; // row length in pixels
        for (x, y) in Bresenham::new(start.into(), end.into()) {
            let idx = 3 * ((y * row_length as isize) + x) as usize;
            self.buffer[idx] = color.blue;
            self.buffer[idx + 1] = color.green;
            self.buffer[idx + 2] = color.red;
        }
    }

    pub fn draw_rect(&mut self, pos: (usize, usize), width: usize, height: usize, color: Rgb) {
        let end = (pos.0 + width, pos.1 + height);
        self.draw_rect_points(pos, end, width, color);
    }

    fn draw_rect_points(&mut self, start: (usize, usize), end: (usize, usize), width: usize, color: Rgb) {
        (start.1..end.1)
            .map(|y| (self.info.width * 3) * y + (start.0 * 3))
            .map(|offset| unsafe { self.buffer.as_mut_ptr().offset(offset as isize) })
            .map(|ptr| unsafe { slice::from_raw_parts_mut(ptr, width * 3) })
            .flat_map(|dst| dst.chunks_exact_mut(3))
            .for_each(|dst| {
                dst[0] = color.blue;
                dst[1] = color.green;
                dst[2] = color.red;
            });
    }
}

pub(crate) fn init(info: FrameBufferInfo, buffer: &'static mut[u8]) {
    let view = FrameBufferView::new(info, buffer);
    *VIEW.lock() = Some(view);
}

pub fn use_view<F>(mut func: F)
where
    F: FnMut(&mut FrameBufferView)
{
    interrupts::without_interrupts(|| {
        let mut lock = VIEW.lock();
        if let Some(view) = &mut *lock {
            func(view);
        }
    });
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Rgb {
    pub red: u8,
    pub blue: u8,
    pub green: u8
}

impl Rgb {
    pub const BLACK:  Rgb = Rgb::new(0, 0, 0);
    pub const WHITE:  Rgb = Rgb::new(255, 255, 255);
    pub const RED:    Rgb = Rgb::new(255, 0, 0);
    pub const ORANGE: Rgb = Rgb::new(255, 127, 0);
    pub const YELLOW: Rgb = Rgb::new(255, 255, 0);
    pub const GREEN:  Rgb = Rgb::new(0, 255, 0);
    pub const BLUE:   Rgb = Rgb::new(0, 0, 255);
    pub const PURPLE: Rgb = Rgb::new(255, 0, 255);

    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

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

    //draw_quad(pixels, x * 8, y * 8, 8);
}

pub fn print_str(str: &str, x_offset: usize, y: usize, fore_color: Rgb, back_color: Rgb) {
    for (idx, char) in str.chars().enumerate() {
        print_char(char, idx + x_offset, y, fore_color, back_color);
    }
}