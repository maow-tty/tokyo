use alloc::boxed::Box;
use alloc::vec;
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use crate::render::{Color, pixel};

pub type PixelConverter = fn(usize, &mut [u8], Color);

/// Provides a high-level abstraction for the frame buffer.
pub trait FrameBufferView {
    /// Provides the frame buffer info.
    fn info(&self) -> FrameBufferInfo;

    /// Provides width in pixels. Equivalent to `width` on [`FrameBufferInfo`](FrameBufferInfo).
    fn width(&self) -> usize;

    /// Provides height in pixels. Equivalent to `height` on [`FrameBufferInfo`](FrameBufferInfo).
    fn height(&self) -> usize;

    /// Provides a pixel converter function.
    ///
    /// This is used to convert layout-agnostic pixel values to
    /// buffer-specific bytes. Not all views are guaranteed to need this, so this
    /// getter defaults to returning a no-op implementation.
    fn pixel_converter(&self) -> PixelConverter {
        pixel::no_op
    }

    /// Clears the screen by setting every pixel to a single color.
    fn clear<C: Copy + Into<Color>>(&mut self, color: C) {
        let color = color.into();
        let info = self.info();

        for y in 0..self.height() {
            for x in 0..self.width() {
                let i = (y * info.stride) + x;
                unsafe {
                    self.set_pixel_unchecked(i * info.bytes_per_pixel, color);
                }
            }
        }
    }

    /// Sets a pixel on the underlying buffer.
    ///
    /// `pos` represents an (x, y) coordinate pair, measured in pixels.
    fn set_pixel<C: Copy + Into<Color>>(&mut self, pos: (usize, usize), color: C);

    /// Sets a pixel on the underlying buffer, avoiding bounds checking.
    ///
    /// Additionally, `index` is measured in bytes rather than pixels.
    ///
    /// ## Safety
    ///
    /// This has the possibility of writing out of the bounds of the buffer,
    /// or, in the case of the frame buffer, writing into the padding bytes.
    ///
    /// The caller must ensure that the passed `index` is within bounds, i.e. does not enter the
    /// padding area (`stride - width`) and doesn't exceed the byte length of the buffer.
    unsafe fn set_pixel_unchecked(&mut self, index: usize, color: Color);

    /// Provides a mutable reference to the underlying buffer.
    ///
    /// ## Safety
    ///
    /// It is possible to invalidate the invariants of the view through this reference.
    /// Care should be taken to make sure all mutation accounts for pixel layout differences if necessary.
    unsafe fn buffer(&mut self) -> &mut [u8];
}

pub struct ImmediateView {
    info: FrameBufferInfo,
    frame_buffer: &'static mut [u8],
    pixel_converter: PixelConverter
}

impl ImmediateView {
    /// Create a view upon this framebuffer.
    pub fn new(frame_buffer: &'static mut FrameBuffer) -> Self {
        let info = frame_buffer.info();
        let format = info.pixel_format;
        let depth = info.bytes_per_pixel;

        // figure out the buffer-specific pixel converter function
        let pixel_converter = match format {
            PixelFormat::Rgb                => unimplemented!(),
            PixelFormat::Bgr if depth == 3  => pixel::bgr_24,
            PixelFormat::Bgr if depth == 4  => pixel::bgr_32,
            PixelFormat::U8                 => unimplemented!(),
            _ => panic!("pixel format not supported: {:?}", format)
        };

        Self { info, frame_buffer: frame_buffer.buffer_mut(), pixel_converter }
    }
}

impl FrameBufferView for ImmediateView {
    fn info(&self) -> FrameBufferInfo {
        self.info
    }

    fn width(&self) -> usize {
        self.info.width
    }

    fn height(&self) -> usize {
        self.info.height
    }

    fn pixel_converter(&self) -> PixelConverter {
        self.pixel_converter
    }

    fn set_pixel<C: Copy + Into<Color>>(&mut self, pos: (usize, usize), color: C) {
        let (x, y) = pos; // position in pixels

        // make sure position is within bounds
        assert!(x < self.info.width);  // 0..width
        assert!(y < self.info.height); // 0..height

        let i = (y * self.info.stride) + x; // total rows + column offset

        (self.pixel_converter)(
            i * self.info.bytes_per_pixel, // convert from pixels to bytes
            self.frame_buffer,
            color.into()
        );
    }

    unsafe fn set_pixel_unchecked(&mut self, index: usize, color: Color) {
        (self.pixel_converter)(index, self.frame_buffer, color);
    }

    unsafe fn buffer(&mut self) -> &mut [u8] {
        self.frame_buffer
    }
}

pub type BufImmediateView = BufView<ImmediateView>;

pub struct BufView<I: FrameBufferView> {
    view: I,
    buffer: Box<[u8]>
}

impl BufView<ImmediateView> {
    pub fn from_immediate(frame_buffer: &'static mut FrameBuffer) -> Self {
        Self::from(ImmediateView::new(frame_buffer))
    }
}

impl<I: FrameBufferView> BufView<I> {
    /// Create a buffered view upon an existing view,
    /// dynamically allocating a back buffer of equivalent byte length.
    pub fn from(view: I) -> Self {
        let buffer = vec![0; view.info().byte_len].into_boxed_slice();
        Self { view, buffer }
    }

    /// Copies the contents of the back buffer to the frame buffer.
    pub fn swap(&mut self) {
        unsafe { self.view.buffer() }.copy_from_slice(&self.buffer);
    }
}

impl<I: FrameBufferView> FrameBufferView for BufView<I> {
    fn info(&self) -> FrameBufferInfo {
        self.view.info()
    }

    fn width(&self) -> usize {
        self.view.width()
    }

    fn height(&self) -> usize {
        self.view.height()
    }

    fn pixel_converter(&self) -> PixelConverter {
        self.view.pixel_converter()
    }

    fn set_pixel<C: Copy + Into<Color>>(&mut self, pos: (usize, usize), color: C) {
        let (x, y) = pos;

        assert!(x < self.info().width);
        assert!(y < self.info().height);

        let i = (y * self.info().stride) + x;

        self.pixel_converter()(
            i * self.info().bytes_per_pixel,
            &mut self.buffer,
            color.into()
        );
    }

    unsafe fn set_pixel_unchecked(&mut self, index: usize, color: Color) {
        self.pixel_converter()(index, &mut self.buffer, color);
    }

    /// Returns a mutable reference to the back buffer.
    unsafe fn buffer(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}