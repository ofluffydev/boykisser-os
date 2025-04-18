use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::{get_and_lock_renderer, info};

const APIC_BASE_PHYS: usize = 0xFEE00000;
pub static mut APIC_BASE: *mut u32 = APIC_BASE_PHYS as *mut u32;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[32].set_handler_fn(timer_interrupt_handler);
        idt[42].set_handler_fn(test_interrupt_handler);
        idt[255].set_handler_fn(spurious_interrupt_handler); // Register spurious interrupt handler
        idt
    };
}

// Add a spurious interrupt handler
extern "x86-interrupt" fn spurious_interrupt_handler(stack_frame: InterruptStackFrame) {
    info("Spurious interrupt occurred");
    let _ = stack_frame;
}

extern "x86-interrupt" fn test_interrupt_handler(stack_frame: InterruptStackFrame) {
    info("Test interrupt occurred!");
    let _ = stack_frame;
}

pub fn test_interrupts() {
    info("Testing interrupts...");
    unsafe {
        core::arch::asm!("int 42", options(nostack));
    }
}

extern "x86-interrupt" fn interrupt_handler(stack_frame: InterruptStackFrame) {
    info("Interrupt occurred");
    let _ = stack_frame;
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    info("Page fault occurred");
    let _ = stack_frame;
    let _ = error_code;
}

pub fn init_idt() {
    IDT.load();
}

/// Disable the PIC
pub fn disable_pic() {
    info("Disabling PIC...");
    unsafe {
        // Mask all interrupts on both PICs
        core::arch::asm!(
            "mov al, 0xFF",
            "out 0xA1, al", // slave PIC
            "out 0x21, al", // master PIC
            options(nostack, nomem, preserves_flags)
        );
    }
}

// Add a function to register NMI sources for ACPI
pub fn register_nmi_sources() {
    info("Registering NMI sources...");
    unsafe {
        // Example: Configure LINT1 as NMI
        let lint1_reg = APIC_BASE.offset(0x350 / 4);
        core::ptr::write_volatile(lint1_reg, 0x400); // Set delivery mode to NMI
    }
}

pub fn enable_apic() {
    disable_pic(); // Ensure PIC is disabled
    info("Enabling APIC...");

    let spurious_reg = unsafe { APIC_BASE.offset(0xF0 / 4) };
    let value = 0x100 | 0xFF; // enable + vector 255
    unsafe { core::ptr::write_volatile(spurious_reg, value) };

    // Set the timer to periodic mode
    get_and_lock_renderer().println("Setting up APIC timer...");
    unsafe {
        let divide_reg = APIC_BASE.offset(0x3E0 / 4);
        core::ptr::write_volatile(divide_reg, 0b0011); // divide by 16
    }

    // Set the LVT timer to periodic mode
    get_and_lock_renderer().println("Setting up LVT timer...");
    unsafe {
        let lvt_timer = APIC_BASE.offset(0x320 / 4);
        core::ptr::write_volatile(lvt_timer, 0x20 | (1 << 17)); // vector 32 + periodic
    }

    // Set the initial count for the timer
    get_and_lock_renderer().println("Setting up initial count...");
    unsafe {
        let init_count = APIC_BASE.offset(0x380 / 4);
        core::ptr::write_volatile(init_count, 10_000_000); // adjust to taste
    }

    register_nmi_sources(); // Register NMI sources for ACPI
}

// Modify the timer interrupt handler to send EOI
extern "x86-interrupt" fn timer_interrupt_handler(stack_frame: InterruptStackFrame) {
    // PIC can eat it, get with the times and use APIC

    // As for APIC:
    unsafe {
        // EOI to APIC
        let eoi_reg = APIC_BASE.offset(0xB0 / 4);
        core::ptr::write_volatile(eoi_reg, 0);
    }
    let _ = stack_frame;
}
