use core::fmt::Write;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, KeyCode, layouts, ScancodeSet1};
use pic8259::ChainedPics;
use spin::{Lazy, Mutex};
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{gdt, graphics};
use crate::graphics::{Rgb, use_view};

pub(crate) const PIC_OFFSET: u8 = 32;

pub(crate) static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new_contiguous(PIC_OFFSET) });

pub(crate) static mut IDT: Lazy<InterruptDescriptorTable> = Lazy::new(||  {
    let mut idt = InterruptDescriptorTable::new();

    // hardware timer handler
    idt[InterruptIndex::Timer as usize].set_handler_fn(timer);

    // keyboard handler
    idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard);

    unsafe {
        // double fault handler
        idt.double_fault
            .set_handler_fn(double_fault)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }

    idt
});

pub(crate) fn init() {
    unsafe {
        IDT.load();
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(u8)]
pub(crate) enum InterruptIndex {
    Timer = PIC_OFFSET,
    Keyboard
}

extern "x86-interrupt" fn double_fault(_frame: InterruptStackFrame, _code: u64) -> ! {
    graphics::use_view(|view| {
        view.clear(Rgb::RED);
    });
    loop {}
}

macro_rules! eoi {
    ($name:ident) => {{
        unsafe {
            $crate::idt::PICS.lock().notify_end_of_interrupt($crate::idt::InterruptIndex::$name as u8);
        }
    }};
}

extern "x86-interrupt" fn timer(_frame: InterruptStackFrame) {
    eoi!(Timer);
}

static KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(
    ScancodeSet1::new(),
    layouts::Us104Key,
    HandleControl::Ignore
));

extern "x86-interrupt" fn keyboard(_frame: InterruptStackFrame) {
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    // TODO: un-ugly the code
    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(char) => {
                    use_view(|view| {
                        if char == '\x08' {
                            view.backspace()
                        } else {
                            view.print_char(char, Rgb::WHITE, Rgb::BLACK);
                        }
                    })
                }
                DecodedKey::RawKey(code) => {
                    match code {
                        KeyCode::Return => {
                            use_view(|view| { view.new_line() });
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    eoi!(Keyboard);
}

