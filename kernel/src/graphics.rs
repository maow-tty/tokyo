use bitvec::macros::internal::funty::Fundamental;
use bitvec::prelude::*;
use line_drawing::{Bresenham, Point};
use spin::Mutex;
use unchecked_index::UncheckedIndex;
use x86_64::instructions::interrupts;
use crate::ext::{BlFrameBufferInfo, FrameBufferInfo};

// TODO: Add support for framebuffers other than 24-bit BGR

static VIEW: Mutex<Option<FrameBufferView>> = Mutex::new(None);

pub struct FrameBufferView {
    info: FrameBufferInfo,
    buffer: UncheckedIndex<&'static mut [u8]>,
    column: usize,
    row: usize,
    max_columns: usize,
    max_rows: usize
}

impl FrameBufferView {
    pub fn new(info: BlFrameBufferInfo, buffer: &'static mut [u8]) -> Self {
        Self {
            info: FrameBufferInfo::from(info),
            buffer: unsafe { unchecked_index::unchecked_index(buffer) },
            column: 0,
            row: 0,
            max_columns: info.height / 8,
            max_rows: info.width / 8
        }
    }

    pub fn clear(&mut self, color: Rgb) {
        let width = self.info.width;
        self.buffer
            .chunks_exact_mut(self.info.stride)
            .map(|line| &mut line[..width])
            .for_each(|dst| {
                for i in (0..width).step_by(3) {
                    dst[i]     = color.blue;
                    dst[i + 1] = color.green;
                    dst[i + 2] = color.red;
                }
            });
    }

    pub fn draw_line<P>(&mut self, start: P, end: P, color: Rgb)
    where
        P: Into<Point<isize>>
    {
        let width = self.info.width;
        for (x, y) in Bresenham::new(start.into(), end.into()) {
            assert!((x as usize) < width);

            let i = (y as usize * width) + (x as usize * 3);
            self.buffer[i]     = color.blue;
            self.buffer[i + 1] = color.green;
            self.buffer[i + 2] = color.red;
        }
    }

    pub fn draw_rect(&mut self, pos: (usize, usize), width: usize, height: usize, color: Rgb) {
        let stride = self.info.stride;

        let mut start = pos.1 * stride;
        for _ in 0..height {
            let dst = &mut self.buffer[start + pos.0 * 3..];
            for i in (0..width * 3).step_by(3) {
                dst[i]     = color.blue;
                dst[i + 1] = color.green;
                dst[i + 2] = color.red;
            }
            start += stride;
        }
    }

    pub fn draw_textured(&mut self, pos: (usize, usize), width: usize, pixels: &[Rgb]) {
        let stride = self.info.stride;
        let start = pos.0 * 3;

        (self.buffer[pos.1 * stride..])
            .chunks_exact_mut(stride)
            .zip(pixels.chunks_exact(width))
            .for_each(|(dst, src)| {
                let mut src = src.into_iter();
                for dst in dst[start..start + width * 3].chunks_exact_mut(3) {
                    let color = src.next().expect("pixel should exist");
                    dst[0] = color.blue;
                    dst[1] = color.green;
                    dst[2] = color.red;
                }
            });
    }

    pub fn draw_char(&mut self, pos: (usize, usize), char: char, foreground: Rgb, background: Rgb) {
        let chars = font8x8::legacy::BASIC_LEGACY[char as usize];

        let pixels = &mut [Rgb::new(0, 0, 0); 64];
        chars.iter()
            .flat_map(|char| char.view_bits::<Lsb0>().iter())
            .enumerate()
            .for_each(|(idx, bit)| {
                pixels[idx] = if bit.as_bool() { background } else { foreground };
            });

        self.draw_textured((pos.0 * 8, pos.1 * 8), 8, pixels);
    }

    pub fn draw_str(&mut self, str: &str, foreground: Rgb, background: Rgb) {
        for char in str.chars() {
            self.draw_char((self.row, self.column), char, foreground, background);
            self.row += 1;
            if self.row == self.max_rows {
                self.row = 0;
                if self.column == self.max_columns {
                    self.column = 0;
                    self.clear(Rgb::BLACK);
                }
                self.column += 1;
            }
        }
    }
}

pub(crate) fn init(info: BlFrameBufferInfo, buffer: &'static mut[u8]) {
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