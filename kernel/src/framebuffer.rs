use bootloader_api::info::{FrameBufferInfo, PixelFormat};

pub struct FrameBufferView<'a> {
    info: FrameBufferInfo,
    buffer: &'a mut [u8]
}

impl<'a> FrameBufferView<'a> {
    pub fn new(info: FrameBufferInfo, buffer: &'a mut [u8]) -> Self {
        Self { info, buffer }
    }

    pub fn clear(&mut self, pixel: Rgb) {
        let format = self.info.pixel_format;
        match format {
            PixelFormat::Rgb => {
                self.buffer.chunks_exact_mut(3).for_each(|dst| {
                    dst[0] = pixel.red;
                    dst[1] = pixel.green;
                    dst[2] = pixel.blue;
                });
            }
            PixelFormat::Bgr => {
                self.buffer.chunks_exact_mut(3).for_each(|dst| {
                    dst[0] = pixel.blue;
                    dst[1] = pixel.green;
                    dst[2] = pixel.red;
                });
            }
            PixelFormat::U8 => {
                let grayscale = (pixel.red + pixel.green + pixel.blue) / 3;
                self.buffer.fill(grayscale);
            }
            PixelFormat::Unknown { .. } | _ => {
                panic!("unsupported pixel format: {:?}", format);
            }
        }
    }

    pub fn draw_line(&mut self, pixels: &[Rgb], offset: usize) {
        let format = self.info.pixel_format;
        match format {
            PixelFormat::Rgb => {
                self.buffer
                    .chunks_exact_mut(3)
                    .skip(offset)
                    .zip(pixels)
                    .for_each(|(dst, src)| {
                        dst[0] = src.red;
                        dst[1] = src.green;
                        dst[2] = src.blue;
                    });
            }
            PixelFormat::Bgr => {
                self.buffer
                    .chunks_exact_mut(3)
                    .skip(offset)
                    .zip(pixels)
                    .for_each(|(dst, src)| {
                        dst[0] = src.blue;
                        dst[1] = src.green;
                        dst[2] = src.red;
                    });
            }
            PixelFormat::U8 => {
                self.buffer
                    .iter_mut()
                    .skip(offset)
                    .zip(pixels)
                    .for_each(|(dst, src)| {
                        *dst = (src.red + src.green + src.blue) / 3;
                    });
            }
            PixelFormat::Unknown { .. } | _ => {
                panic!("unsupported pixel format: {:?}", format);
            }
        }
    }

    // MASSIVELY TERRIBLE!
    // TODO: FIX EVENTUALLY!!! (probably after the alloc is written)
    pub fn draw_quad(&mut self,
                     pixels: &[Rgb],
                     x_offset: usize, y_offset: usize,
                     line_width: usize) {
        let format = self.info.pixel_format;
        let stride = self.info.stride * 3;
        self.buffer
            .chunks_exact_mut(stride) // chunk into lines
            .skip(y_offset) // skip y offset amount of lines
            .zip(pixels.chunks_exact(line_width)) // chunk src into lines
            .for_each(|(dst_line, src_line)| {
                dst_line
                    .chunks_exact_mut(3) // chunk into pixels
                    .skip(x_offset) // skip x amount of pixels
                    .zip(src_line)
                    .for_each(|(dst, src)| {
                        match format {
                            PixelFormat::Rgb => {
                                dst[0] = src.red;
                                dst[1] = src.green;
                                dst[2] = src.blue;
                            }
                            PixelFormat::Bgr => {
                                dst[0] = src.blue;
                                dst[1] = src.green;
                                dst[2] = src.red;
                            }
                            PixelFormat::U8 | PixelFormat::Unknown { .. } | _ => {
                                panic!("unsupported pixel format: {:?}", format);
                            }
                        }
                    });
            });
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Rgb {
    red: u8,
    green: u8,
    blue: u8
}

impl Rgb {
    pub const BLACK: Rgb = Rgb::new(0, 0, 0);
    pub const WHITE: Rgb = Rgb::new(255, 255, 255);
    pub const RED:   Rgb = Rgb::new(255, 0, 0);
    
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}