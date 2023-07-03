pub mod pixel;
pub mod view;

use core::cell::OnceCell;
use bootloader_api::info::FrameBuffer;
use spin::Mutex;
use x86_64::instructions::interrupts;
use crate::render::view::ImmediateView;

/// A global frame buffer view that is initialized by the kernel entrypoint.
///
/// This is used by various places around the kernel and provides concurrency safety,
/// which is necessary when rendering from hardware interrupts.
pub static GLOBAL_VIEW: Mutex<OnceCell<ImmediateView>> = Mutex::new(OnceCell::new());

pub fn init_global_view(frame_buffer: &'static mut FrameBuffer) {
    GLOBAL_VIEW.lock()
        .set(ImmediateView::new(frame_buffer))
        .ok().expect("frame buffer view should not be initialized twice");
}

/// Access the global frame buffer view.
///
/// This creates a temporary block where hardware interrupts are disabled, and
/// the view is locked by the caller.
pub fn use_global_view<F>(mut func: F)
where
    F: FnMut(&mut ImmediateView)
{
    interrupts::without_interrupts(|| {
        let mut lock = GLOBAL_VIEW.lock();
        if let Some(view) = lock.get_mut() {
            func(view);
        }
    });
}

/// An RGB color type.
#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8)) -> Self {
        Color::new(value.0, value.1, value.2)
    }
}