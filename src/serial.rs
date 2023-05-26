use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;
use core::fmt;

lazy_static! {
    pub static ref SERIAL1 : Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3f8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

pub struct Green(pub &'static str);

impl fmt::Display for Green {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
        write!(f, "\x1B[32m")?;
        write!(f, "{}", self.0)?;
        write!(f, "\x1B[0m")?;
        Ok(())
    }
}

pub struct Red(pub &'static str);

impl fmt::Display for Red {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
        write!(f, "\x1B[31m")?;
        write!(f, "{}", self.0)?;
        write!(f, "\x1B[0m")?;
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}