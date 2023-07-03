#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod idt;
pub mod gdt;
pub mod mem;
pub mod task;
pub mod render;
pub mod serial;

extern crate alloc; // enable allocation

use core::panic::PanicInfo;
use bootloader_api::{BootInfo, BootloaderConfig};
use bootloader_api::config::{Mapping, Mappings};
use x86_64::{instructions, VirtAddr};
use x86_64::instructions::interrupts;
use crate::idt::PICS;
use crate::mem::heap::KernelFrameAllocator;

const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings = {
        let mut mappings = Mappings::new_default();
        mappings.physical_memory = Some(Mapping::Dynamic);
        mappings
    };
    config.kernel_stack_size = 5_000 * 1024; // 5,000 KiB
    config
};

bootloader_api::entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // TODO: implement acpi

    serial_println!("[tokyo] system booted");

    // memory allocation
    let physical_offset = boot_info.physical_memory_offset.into_option().unwrap();
    let mut offset_table = unsafe { mem::mapper(VirtAddr::new(physical_offset)) };
    let mut frame_allocator = unsafe { KernelFrameAllocator::new(&boot_info.memory_regions).unwrap() };
    mem::heap::init(&mut offset_table, &mut frame_allocator).expect("heap initialization should not fail");

    // framebuffer
    let frame_buffer = boot_info.framebuffer.as_mut().unwrap();
    render::init_global_view(frame_buffer);

    gdt::init(); // global descriptor table
    idt::init(); // interrupt descriptor table

    unsafe { PICS.lock().initialize(); } // programmable interrupt controller

    interrupts::enable(); // set interrupts

    block_indefinitely();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[tokyo] {:?}", info);

    block_indefinitely();
}

fn block_indefinitely() -> ! {
    loop { instructions::hlt(); }
}