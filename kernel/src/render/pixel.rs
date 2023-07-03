//! A collection of simple pixel drawing functions used by the frame buffer view.

use crate::render::Color;

pub fn no_op(_pos: usize, _buffer: &mut [u8], _color: Color) {}

pub fn bgr_24(pos: usize, buffer: &mut [u8], color: Color) {
    buffer[pos]     = color.blue;
    buffer[pos + 1] = color.green;
    buffer[pos + 2] = color.red;
}

pub fn bgr_32(pos: usize, buffer: &mut [u8], color: Color) {
    buffer[pos]     = 0; // reserved byte on GOP buffers, leave this empty
    buffer[pos + 1] = color.blue;
    buffer[pos + 2] = color.green;
    buffer[pos + 3] = color.red;
}