#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod idt;
mod gdt;
mod acpi;
pub mod graphics;
pub mod alloc;
pub mod ext;

use core::fmt::Write;
use core::panic::PanicInfo;
use bootloader_api::BootInfo;
use x86_64::instructions;
use x86_64::instructions::interrupts;
use crate::graphics::{ErrorWriter, Rgb};
use crate::idt::PICS;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // TODO: implement acpi after alloc implementation
    // if let Some(rsdp) = boot_info.rsdp_addr.as_ref() {
    //     acpi::init(*rsdp);
    // }

    // framebuffer
    let mut framebuffer = boot_info.framebuffer.as_mut().expect("framebuffer should exist");
    graphics::init(framebuffer.info(), framebuffer.buffer_mut());

    gdt::init(); // global descriptor table
    idt::init(); // interrupt descriptor table

    unsafe { PICS.lock().initialize(); } // programmable interrupt controller
    interrupts::enable(); // set interrupts

    graphics::use_view(|view| {
        view.clear(Rgb::BLACK);
        view.print_str("tokyo 0.1.0\n---\nthe bees!\n", Rgb::PURPLE, Rgb::BLACK);
    });

    block_indefinitely();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    graphics::use_view(|view| {
        let mut error_writer = ErrorWriter::new(view);
        write!(&mut error_writer, "{}", info).unwrap();
    });
    block_indefinitely();
}

fn block_indefinitely() -> ! {
    loop { instructions::hlt(); }
}