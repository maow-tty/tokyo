use core::fmt;
use bitvec::macros::internal::funty::Fundamental;
use bitvec::prelude::*;
use line_drawing::{Bresenham, Point};
use spin::Mutex;
use unchecked_index::UncheckedIndex;
use x86_64::instructions::interrupts;
use crate::ext::{BlFrameBufferInfo, FrameBufferInfo};

// TODO: Add support for framebuffers other than 24-bit BGR

pub const CHAR_WIDTH  : usize = 8;
pub const CHAR_HEIGHT : usize = 8;

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
            max_columns: info.height / CHAR_HEIGHT,
            max_rows: info.width / CHAR_WIDTH
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
                let mut src = src.iter();
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

        let pixels = &mut [Rgb::BLACK; CHAR_WIDTH * CHAR_HEIGHT];
        chars.iter()
            .flat_map(|char| char.view_bits::<Lsb0>().iter())
            .enumerate()
            .for_each(|(idx, bit)| {
                pixels[idx] = if bit.as_bool() { background } else { foreground };
            });

        self.draw_textured((pos.0 * CHAR_WIDTH, pos.1 * CHAR_HEIGHT), CHAR_WIDTH, pixels);
    }

    pub fn print_char(&mut self, char: char, foreground: Rgb, background: Rgb) {
        self.end_of_row();
        self.end_of_column();

        if char == '\n' {
            self.column += 1;
            self.row = 0;
            return;
        }

        self.draw_char((self.row, self.column), char, foreground, background);
        self.row += 1;
    }

    fn end_of_row(&mut self) {
        if self.row == self.max_rows {
            self.row = 0;
            self.column += 1;
        }
    }

    fn end_of_column(&mut self) -> bool {
        if self.column == self.max_columns {
            // TODO: push line upwards, this will be done when I can implement a text buffer
            self.clear(Rgb::BLACK);
            self.column = 0;
            self.row = 0;
            return true;
        }
        false
    }

    pub fn print_str(&mut self, str: &str, foreground: Rgb, background: Rgb) {
        for char in str.chars() {
            self.print_char(char, foreground, background);
        }
    }

    pub fn new_line(&mut self) {
        if !self.end_of_column() {
            self.column += 1;
            self.row = 0;
        }
    }

    pub fn backspace(&mut self) {
        if self.row == 0 && self.column != 0 {
            self.column -= 1;
            self.row = self.max_rows - 1;
        } else if self.row != 0 {
            self.row -= 1;
            self.draw_char((self.row, self.column), ' ', Rgb::BLACK, Rgb::BLACK);
        }
    }
}

pub struct ErrorWriter<'a>(&'a mut FrameBufferView);

impl<'a> ErrorWriter<'a> {
    pub fn new(view: &'a mut FrameBufferView) -> Self {
        Self(view)
    }
}

impl<'a> fmt::Write for ErrorWriter<'a> {
    fn write_str(&mut self, str: &str) -> fmt::Result {
        self.0.clear(Rgb::RED);
        self.0.print_str(str, Rgb::WHITE, Rgb::RED);
        Ok(())
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