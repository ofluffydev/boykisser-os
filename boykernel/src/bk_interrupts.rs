use acpi::madt::{Madt, MadtEntry};
use alloc::format;
use core::pin::Pin;
use core::ptr;
use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::structures::tss::TaskStateSegment;

use crate::{get_and_lock_renderer, info};

const APIC_BASE_PHYS: usize = 0xFEE00000;
pub static mut APIC_BASE: *mut u32 = APIC_BASE_PHYS as *mut u32;

const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// Allocate stacks for IST entries
#[repr(align(16))]
struct AlignedStack([u8; 4096]);
static mut DOUBLE_FAULT_STACK: AlignedStack = AlignedStack([0; 4096]);

// Define the TSS
static mut TSS: TaskStateSegment = TaskStateSegment::new();

// Define the GDT using lazy_static
lazy_static! {
    static ref GDT: (GlobalDescriptorTable, SegmentSelector) = {
        // Set up the TSS
        unsafe {
            TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] =
                VirtAddr::new(&raw const DOUBLE_FAULT_STACK as *const _ as u64 + 4096);
        }

        // Set up the GDT
        let mut gdt = GlobalDescriptorTable::new();
        let entry = Descriptor::tss_segment(unsafe{&TSS});
        let tss_selector = gdt.append(entry);
        (gdt, tss_selector)
    };
}

pub fn init_gdt_tss() {
    // Load the GDT
    GDT.0.load();

    // Load the TSS
    unsafe {
        load_tss(GDT.1);
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[32].set_handler_fn(timer_interrupt_handler);
        idt[42].set_handler_fn(test_interrupt_handler);
        idt[255].set_handler_fn(spurious_interrupt_handler);
        // Register new handlers
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.divide_error.set_handler_fn(divide_by_zero_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
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

// Handler for Double Fault
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    info("Double Fault occurred!");
    panic!("Double Fault: {:#?}", stack_frame);
}

// Handler for General Protection Fault
extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    info("General Protection Fault occurred!");
    panic!(
        "General Protection Fault: {:#?}, Error Code: {:#x}",
        stack_frame, error_code
    );
}

// Handler for Invalid Opcode
extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    info("Invalid Opcode occurred!");
    panic!("Invalid Opcode: {:#?}", stack_frame);
}

// Handler for Divide-by-Zero
extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: InterruptStackFrame) {
    info("Divide-by-Zero occurred!");
    panic!("Divide-by-Zero: {:#?}", stack_frame);
}

// Handler for Stack Segment Fault
extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    info("Stack Segment Fault occurred!");
    panic!(
        "Stack Segment Fault: {:#?}, Error Code: {:#x}",
        stack_frame, error_code
    );
}

pub fn init_idt() {
    init_gdt_tss(); // Initialize GDT and TSS
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

/// Calibrate the APIC timer using the PIT
fn calibrate_apic_timer() -> u32 {
    info("Calibrating APIC timer...");

    // Set up the PIT for a known duration (e.g., 50 ms)
    let pit_channel = 0x40; // PIT channel 0
    let pit_command = 0x43; // PIT command register
    let pit_reload_value: u16 = 59659; // ~50 ms at 1193182 Hz clock

    unsafe {
        // Configure PIT in mode 2 (rate generator)
        core::arch::asm!(
            "out dx, al",
            in("dx") pit_command,
            in("al") 0x34u8, // Binary mode, mode 2, access mode: lobyte/hibyte
        );

        // Set reload value
        core::arch::asm!(
            "out dx, al",
            in("dx") pit_channel,
            in("al") (pit_reload_value & 0xFF) as u8, // Low byte
        );
        core::arch::asm!(
            "out dx, al",
            in("dx") pit_channel,
            in("al") (pit_reload_value >> 8) as u8, // High byte
        );
    }

    // Start the APIC timer with a large initial count
    unsafe {
        let divide_reg = APIC_BASE.offset(0x3E0 / 4);
        core::ptr::write_volatile(divide_reg, 0b0011); // Divide by 16

        let init_count = APIC_BASE.offset(0x380 / 4);
        core::ptr::write_volatile(init_count, 0xFFFFFFFF); // Max count
    }

    // Wait for the PIT to finish
    let mut pit_finished = false;
    while !pit_finished {
        let pit_status: u8;
        unsafe {
            core::arch::asm!(
                "in al, dx",
                out("al") pit_status,
                in("dx") pit_channel,
            );
        }
        pit_finished = pit_status & 0x80 == 0; // Check if PIT output is low
    }

    // Read the APIC timer's current count
    let current_count: u32;
    unsafe {
        let current_count_reg = APIC_BASE.offset(0x390 / 4);
        current_count = core::ptr::read_volatile(current_count_reg);
    }

    // Calculate the calibrated count for 1 ms (assuming 50 ms PIT duration)
    let calibrated_count = (0xFFFFFFFF - current_count) / 50;
    info(&format!(
        "Calibrated APIC timer count: {}",
        calibrated_count
    ));
    calibrated_count
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

    // Calibrate and set the initial count for the timer
    let calibrated_count = calibrate_apic_timer();
    get_and_lock_renderer().println("Setting up initial count...");
    unsafe {
        let init_count = APIC_BASE.offset(0x380 / 4);
        core::ptr::write_volatile(init_count, calibrated_count);
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

/// Parse the ACPI MADT table to retrieve APIC IDs and mappings
pub fn parse_madt(madt: &Madt) {
    info("Parsing MADT...");
    let madt = unsafe { Pin::new_unchecked(madt) }; // Pin the MADT instance
    for entry in madt.entries() {
        match entry {
            MadtEntry::LocalApic(apic) => {
                info(&format!(
                    "Found Local APIC: Processor ID = {}, APIC ID = {}, Flags = {:#x}",
                    apic.processor_id,
                    apic.apic_id,
                    {
                        let flags = apic.flags;
                        flags
                    }
                ));
            }
            MadtEntry::IoApic(io_apic) => {
                info(&format!(
                    "Found I/O APIC: ID = {}, Address = {:#x}, Global System Interrupt Base = {}",
                    io_apic.io_apic_id,
                    {
                        let address = io_apic.io_apic_address;
                        address
                    },
                    {
                        let gsi_base = io_apic.global_system_interrupt_base;
                        gsi_base
                    }
                ));
            }
            MadtEntry::InterruptSourceOverride(iso) => {
                info(&format!(
                    "Interrupt Source Override: Bus = {}, Source = {}, GSI = {}, Flags = {:#x}",
                    iso.bus,
                    iso.irq,
                    {
                        let gsi = iso.global_system_interrupt;
                        gsi
                    },
                    {
                        let flags = iso.flags;
                        flags
                    }
                ));
            }
            _ => {
                info("Unknown MADT entry");
            }
        }
    }
}

/// Reprogram the I/O APIC for interrupt routing
pub fn configure_io_apic(io_apic_base: usize, gsi: u32, vector: u8, flags: u16) {
    info(&format!(
        "Configuring I/O APIC: Base = {:#x}, GSI = {}, Vector = {}, Flags = {:#x}",
        io_apic_base, gsi, vector, flags
    ));

    let io_apic = io_apic_base as *mut u32;

    unsafe {
        // Select the redirection table entry
        ptr::write_volatile(io_apic.offset(0), 0x10 + (gsi * 2) as u32);

        // Write the lower 32 bits of the redirection entry
        let mut redirection_entry = vector as u32;
        if flags & 0x2 != 0 {
            redirection_entry |= 1 << 13; // Active low
        }
        if flags & 0x8 != 0 {
            redirection_entry |= 1 << 15; // Level-triggered
        }
        ptr::write_volatile(io_apic.offset(4), redirection_entry);

        // Write the upper 32 bits of the redirection entry (destination APIC ID)
        ptr::write_volatile(io_apic.offset(0), 0x10 + (gsi * 2 + 1) as u32);
        ptr::write_volatile(io_apic.offset(4), 0); // Destination APIC ID = 0 for now
    }
}

pub fn init_interrupts(madt: &Madt) {
    parse_madt(madt); // Parse the MADT table

    // Example: Configure an I/O APIC entry for a specific GSI
    configure_io_apic(0xFEC00000, 1, 32, 0x8); // Example values

    init_gdt_tss(); // Initialize GDT and TSS
    IDT.load(); // Load the IDT
}
