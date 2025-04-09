use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

const APIC_BASE_PHYS: usize = 0xFEE00000;
pub static mut APIC_BASE: *mut u32 = APIC_BASE_PHYS as *mut u32;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[32].set_handler_fn(timer_interrupt_handler); // 32 is the IRQ number for the timer
        idt
    };
}

extern "x86-interrupt" fn interrupt_handler(stack_frame: InterruptStackFrame) {
    let _ = stack_frame;
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let _ = stack_frame;
    let _ = error_code;
}

pub fn init_idt() {
    IDT.load();
}

pub fn enable_apic() {
    let spurious_reg = unsafe { APIC_BASE.offset(0xF0 / 4) };
    let value = 0x100 | 0xFF; // enable + vector 255
    unsafe { core::ptr::write_volatile(spurious_reg, value) };

    // Set the timer to periodic mode
    unsafe {
        let divide_reg = APIC_BASE.offset(0x3E0 / 4);
        core::ptr::write_volatile(divide_reg, 0b0011); // divide by 16
    }

    // Set the LVT timer to periodic mode
    unsafe {
        let lvt_timer = APIC_BASE.offset(0x320 / 4);
        core::ptr::write_volatile(lvt_timer, 0x20 | (1 << 17)); // vector 32 + periodic
    }

    // Set the initial count for the timer
    unsafe {
        let init_count = APIC_BASE.offset(0x380 / 4);
        core::ptr::write_volatile(init_count, 10_000_000); // adjust to taste
    }
}

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
