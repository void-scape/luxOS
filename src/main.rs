#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(clippy::missing_safety_doc)]

use core::{arch::global_asm, panic::PanicInfo};
use exit::{exit_qemu, QemuExitCode};
use multiboot::MultibootHeader;
use port::PortManager;

extern crate alloc;

pub mod channel;
pub mod circular_buffer;
pub mod cpuuid;
pub mod exit;
pub mod framebuffer;
pub mod gdt;
pub mod idt;
pub mod interrupt;
pub mod kernel;
pub mod lock;
pub mod log;
pub mod memory;
pub mod multiboot;
pub mod pic;
pub mod port;
pub mod ps2;
pub mod serial;
pub mod test;
pub mod time;
pub mod vga;

global_asm!(include_str!("boot.s"));

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn kernel_main(magic: u32, multiboot_header: *const MultibootHeader) {
    multiboot::verify_mutliboot_magic(magic);
    let multiboot_header = unsafe { &*multiboot_header };
    memory::ALLOCATOR.init(multiboot_header);

    let mut port_manager = PortManager::default();
    // WARN: Tests require the `log` feature for no discernable reason. Will hang here otherwise.
    log::init(log::LogLevel::Info, &mut port_manager);

    #[cfg(test)]
    test_main();

    let mut kernel = kernel::Kernel::new(multiboot_header, port_manager);
    // kernel.run();
    kernel.square_demo();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    struct PanicWriter;

    impl core::fmt::Write for PanicWriter {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for c in s.chars() {
                while unsafe { port::Port::new(0x3FD).read() & 0x20 } == 0 {}
                unsafe { port::Port::new(0x3F8).write(c as u8) };
            }
            Ok(())
        }
    }

    use core::fmt::Write;
    let _ = write!(PanicWriter, "\nPANIC: {}", info.message());

    exit_qemu(QemuExitCode::Failed);
    loop {}
}
