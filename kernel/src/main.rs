#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod idt;
pub mod gdt;
pub mod acpi;
pub mod graphics;
pub mod mem;
pub mod ext;

extern crate alloc;

use core::fmt::Write;
use core::panic::PanicInfo;
use bootloader_api::{BootInfo, BootloaderConfig};
use bootloader_api::config::{Mapping, Mappings};
use x86_64::{instructions, VirtAddr};
use x86_64::instructions::interrupts;
use crate::graphics::{ErrorWriter, Rgb};
use crate::idt::PICS;
use crate::mem::StandardFrameAllocator;

const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    let mut mappings = Mappings::new_default();
    mappings.physical_memory = Some(Mapping::Dynamic);
    config.mappings = mappings;
    config
};

bootloader_api::entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // TODO: implement acpi after alloc implementation
    // if let Some(rsdp) = boot_info.rsdp_addr.as_ref() {
    //     acpi::init(*rsdp);
    // }

    // offset page table
    let physical_offset = boot_info.physical_memory_offset.into_option().unwrap();
    let mut offset_table = unsafe { mem::init(VirtAddr::new(physical_offset)) };
    let mut frame_allocator = unsafe { StandardFrameAllocator::new(&boot_info.memory_regions) };
    mem::init_heap(&mut offset_table, &mut frame_allocator).expect("heap initialization should not fail");

    // framebuffer
    let framebuffer = boot_info.framebuffer.as_mut().expect("framebuffer should exist");
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
        view.clear(Rgb::RED);
        let mut error_writer = ErrorWriter::new(view);
        write!(&mut error_writer, "{}", info).unwrap();
    });
    block_indefinitely();
}

fn block_indefinitely() -> ! {
    loop { instructions::hlt(); }
}