#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use core::{
    alloc::{GlobalAlloc, Layout},
    arch::asm,
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};
use spin::{Mutex, Once};
use x86_64::instructions::interrupts::enable;

#[allow(unused_imports)] // Falsely reports as unused for some reason
use alloc::string::ToString;

extern crate alloc;

use crate::{
    beep::beep,
    bk_interrupts::{enable_apic, init_idt, test_interrupts},
    framebuffer::FramebufferInfo,
    gop_render::SimplifiedRenderer,
    serial::info,
};

mod beep;
mod bk_interrupts;
mod font;
mod framebuffer;
mod gop_render;
pub mod memory;
mod serial;
mod strings;
mod utils;
mod watermark;

const HEAP_SIZE: usize = 4096;

struct GayAllocator {
    heap: UnsafeCell<[MaybeUninit<u8>; HEAP_SIZE]>,
    offset: AtomicUsize,
}

unsafe impl GlobalAlloc for GayAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let size = layout.size();
            let align = layout.align();

            let heap_start = self.heap.get().cast::<u8>();
            loop {
                let orig_offset = self.offset.load(Ordering::Relaxed);
                let ptr = heap_start.add(orig_offset);

                let offset = ptr.align_offset(align);
                if offset == usize::MAX {
                    return core::ptr::null_mut();
                }

                let alloc = ptr.add(offset);
                if alloc.offset_from(heap_start) as usize > HEAP_SIZE {
                    return core::ptr::null_mut();
                }

                if self
                    .offset
                    .compare_exchange_weak(
                        orig_offset,
                        orig_offset + offset + size,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    return alloc;
                } else {
                    continue;
                }
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            let size = layout.size();
            let heap_start = self.heap.get().cast::<u8>();
            let start_of_alloc = ptr.offset_from(heap_start) as usize;
            let end_of_alloc = ptr.add(size).offset_from(heap_start) as usize;

            _ = self.offset.compare_exchange(
                end_of_alloc,
                start_of_alloc,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
        }
    }
}

unsafe impl Sync for GayAllocator {}

#[global_allocator]
static GLOBAL: GayAllocator = GayAllocator {
    heap: UnsafeCell::new([const { MaybeUninit::uninit() }; HEAP_SIZE]),
    offset: AtomicUsize::new(0),
};

// Global Once to hold the Mutex for the renderer
pub static RENDERER: Once<Mutex<SimplifiedRenderer>> = Once::new();

/// Helper function to get and lock the global renderer
pub fn get_and_lock_renderer() -> spin::MutexGuard<'static, SimplifiedRenderer<'static>> {
    RENDERER.get().expect("Renderer is not initialized").lock()
}

#[unsafe(no_mangle)]
pub extern "C" fn _start(fb: &'static FramebufferInfo) -> ! {
    info("Kernel successfully jumped to!");

    let renderer = SimplifiedRenderer::new(fb);
    info("Initializing global renderer");
    RENDERER.call_once(|| Mutex::new(renderer));

    info("Initializing interrupts");
    let madt = get_madt_table();
    bk_interrupts::init_interrupts(&madt); // Pass the MADT table here

    let renderer = get_and_lock_renderer();
    renderer.clear_screen();
    renderer.show_alphabet();
    renderer.show_watermark();

    info("Running interrupts test");
    test_interrupts();

    loop {
        unsafe { asm!("hlt") }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(panic: &core::panic::PanicInfo) -> ! {
    use serial::{error, serial_write_str};

    error("Panic occurred: ");
    serial_write_str("=== PANIC ===\n");

    #[cfg(not(debug_assertions))]
    serial_write_str("[WARNING] This is a release build, panic information may be limited.\n");

    if let Some(location) = panic.location() {
        serial_write_str("Location: ");
        serial_write_str(location.file());
        serial_write_str(":");
        serial_write_str(location.line().to_string().as_str());
        serial_write_str(":");
        serial_write_str(location.column().to_string().as_str());
        serial_write_str("\n");
    } else {
        serial_write_str("Location: <unknown>\n");
    }

    if let Some(message) = panic.message().as_str() {
        serial_write_str("Message: ");
        serial_write_str(message);
        serial_write_str("\n");
    } else {
        serial_write_str("Message: <none>\n");
    }

    serial_write_str("=============\n");
    serial_write_str("\n\n");

    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}
