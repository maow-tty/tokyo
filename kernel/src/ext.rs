use bootloader_api::info::PixelFormat;

pub type BlFrameBufferInfo = bootloader_api::info::FrameBufferInfo;

/// An improved version of the FrameBufferInfo struct that is more convenient for low-level graphics programming.
/// Every field, aside from depth, is a measurement of bytes rather than pixels, as this allows for better manipulation of the framebuffer.
///
/// The `padding` field is equivalent to `(info.stride - info.width) * info.bytes_per_pixel`.
///
/// It is also important to note that the layout of a pixel should **always** be in little-endian byte order,
/// as specified by the Graphics Output Protocol specification, and that the reserved byte in UEFI 32-bit output is
/// always placed as the first byte in a pixel. (RGB then reserved/BGR then reserved)
pub struct FrameBufferInfo {
    pub len     : usize,        // total length
    pub width   : usize,        // row length
    pub padding : usize,        // row padding length
    pub stride  : usize,        // total row length (width + padding)
    pub height  : usize,        // column length
    pub depth   : usize,        // bytes per pixel AKA bit depth
    pub format  : PixelFormat   // pixel layout format
}

impl FrameBufferInfo {
    pub fn from(info: BlFrameBufferInfo) -> Self {
        let depth = info.bytes_per_pixel;
        Self {
            len     : info.byte_len,
            width   : info.width * depth,
            padding : (info.stride - info.width) * depth,
            stride  : info.stride * depth,
            height  : info.height * depth,
            depth,
            format  : info.pixel_format
        }
    }
}