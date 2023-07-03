use pic8259::ChainedPics;
use spin::{Lazy, Mutex};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::{block_indefinitely, gdt, serial_println};

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

    idt.page_fault.set_handler_fn(page_fault);

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

macro_rules! eoi {
    ($name:ident) => {{
        unsafe {
            $crate::idt::PICS.lock().notify_end_of_interrupt($crate::idt::InterruptIndex::$name as u8);
        }
    }};
}

extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, _code: u64) -> ! {
    serial_println!("[tokyo] double fault: {:?}", frame);
    block_indefinitely();
}

extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, code: PageFaultErrorCode) {
    serial_println!("[tokyo] page fault ({:?}): {:?}", code, frame);
    block_indefinitely();
}

extern "x86-interrupt" fn timer(_frame: InterruptStackFrame) {
    eoi!(Timer);
}

extern "x86-interrupt" fn keyboard(_frame: InterruptStackFrame) {
    eoi!(Keyboard);
}

