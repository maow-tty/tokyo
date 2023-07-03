use core::fmt::{Write, Arguments};
use spin::{Lazy, Mutex};
use uart_16550::SerialPort;

pub static SERIAL1: Lazy<Mutex<SerialPort>> = Lazy::new(|| {
    let mut serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init();
    Mutex::new(serial_port)
});

#[doc(hidden)]
pub fn print(args: Arguments) {
    SERIAL1.lock().write_fmt(args).expect("serial should be printable");
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::print(format_args!($($arg)*))
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    ()                       => { $crate::serial_print!("\n") };
    ($fmt:expr)              => { $crate::serial_print!(concat!($fmt, "\n")) };
    ($fmt:expr, $($arg:tt)*) => { $crate::serial_print!(concat!($fmt, "\n"), $($arg)*) };
}