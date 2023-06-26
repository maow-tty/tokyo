#![no_std]
#![no_main]

mod framebuffer;
mod text;

use core::fmt::Write;
use core::cell::OnceCell;
use core::panic::PanicInfo;
use bootloader_api::BootInfo;
use spin::Mutex;
use crate::framebuffer::{FrameBufferView, Rgb};
use crate::text::{ErrorWriter, print_str};

pub static VIEW: Mutex<OnceCell<FrameBufferView>> = Mutex::new(OnceCell::new());

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let framebuffer = boot_info.framebuffer.as_mut().expect("framebuffer should exist");
    {
        let mut view = FrameBufferView::new(framebuffer.info(), framebuffer.buffer_mut());
        view.clear(Rgb::BLACK);
        VIEW.lock().set(view).ok().unwrap();
    }
    panic!("a test panic to show off the panic screen.");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    VIEW.lock().get_mut().unwrap().clear(Rgb::RED);
    let mut error_writer = ErrorWriter::new();
    if let Err(_) = write!(error_writer, "{}", info) {
        print_str("failed to panic", 0, 0, Rgb::WHITE, Rgb::RED);
    }
    loop {}
}