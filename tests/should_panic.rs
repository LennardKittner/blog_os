#![no_std]
#![no_main]

use core::panic::PanicInfo;
use my_os::{QemuExitCode, exit_qemu, serial_println, serial_print, serial::Green, serial::Red};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_panic();
    serial_println!("{}", Red("[test did not panic]"));
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("{}", Green("[ok]"));
    exit_qemu(QemuExitCode::Success);
    loop {}
}

fn should_panic() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}