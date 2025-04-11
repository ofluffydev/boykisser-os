#![no_main]
#![no_std]

extern crate alloc;

mod elf_garbage;
mod files;
mod framebuffer;

use core::arch::asm;

use elf_garbage::load_kernel;
use framebuffer::{initialize_framebuffer, FramebufferInfo};
use log::info;
use uefi::{
    boot::{get_handle_for_protocol, open_protocol_exclusive},
    prelude::*,
    proto::console::
        text::Output
    ,
    table::cfg::{ConfigTableEntry, ACPI2_GUID, ACPI_GUID},
};

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    let handle = get_handle_for_protocol::<Output>().unwrap();
    let mut output = open_protocol_exclusive::<Output>(handle).unwrap();
    output.clear().expect("Failed to clear screen");
    info!("boyloader online!");
    //info!("Launching in 3 seconds...");
    //boot::stall(3_000_000);
    output.clear().expect("Failed to clear screen");
    boot_system();

    Status::SUCCESS
}

pub fn boot_system() {
    let (entry_point, kernel_entry) = load_kernel("\\EFI\\BOOT\\boykernel");

    info!("Kernel entry point: 0x{:x}", kernel_entry as usize);

    let framebuffer_info = initialize_framebuffer();
    info!("Framebuffer info: {:?}", framebuffer_info);

    let rsdp = locate_rsdp().expect("Failed to locate RSDP");
    info!("RSDP address: 0x{:x}", rsdp as usize);

    info!("Jumping to kernel entry point at 0x{:x}", entry_point);

    unsafe {
        let fb_ptr = &framebuffer_info as *const FramebufferInfo;
        asm!(
            "mov rdi, {0}",
            "mov rsi, {1}",
            in(reg) fb_ptr,
            in(reg) rsdp,
            options(nostack)
        );
        kernel_entry();
    }
}

fn locate_rsdp() -> Option<*const u8> {
    uefi::system::with_config_table(|tables: &[ConfigTableEntry]| {
        for entry in tables {
            if entry.guid == ACPI2_GUID {
                let ptr = entry.address as *const u8;
                // SAFETY: We assume the pointer is valid and points to the RSDP structure.
                return Some(ptr);
            }
        }
        None
    })
}
