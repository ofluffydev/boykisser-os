#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use core::{alloc::{GlobalAlloc, Layout}, arch::asm, cell::UnsafeCell, mem::MaybeUninit, ptr, sync::atomic::{AtomicUsize, Ordering}};
use bk_interrupts::enable_apic;
use framebuffer::FramebufferInfo;
use gop_render::SimplifiedRenderer;
use heapless::String;
use spin::{Mutex, Once};
use x86_64::instructions::interrupts::enable;

mod font;
mod framebuffer;
mod gop_render;
mod watermark;
mod strings;
mod bk_interrupts;

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
            // spin loop to allocate
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
                    // successfully wrote the new offset, the allocation succeeded
                    return alloc;
                } else {
                    // something else modified the offset inbetween the start of the loop and here, just redo everything
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

            // this is just `if self.offset == end_of_alloc { self.offset = start_of_alloc; }` but done atomically
            _ = self.offset.compare_exchange(
                end_of_alloc,
                start_of_alloc,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
        }
    }
}

unsafe impl Sync for GayAllocator {} // this is so it can be put in a `static` below

#[global_allocator]
static GLOBAL: GayAllocator = GayAllocator {
    heap: UnsafeCell::new([const { MaybeUninit::uninit() }; HEAP_SIZE]),
    offset: AtomicUsize::new(0),
};


// Global Once to hold the Mutex for the renderer
pub static RENDERER: Once<Mutex<SimplifiedRenderer>> = Once::new();

/// Helper function to get and lock the global renderer
fn get_and_lock_renderer() -> spin::MutexGuard<'static, SimplifiedRenderer<'static>> {
    RENDERER.get().expect("Renderer is not initialized").lock()
}

#[unsafe(no_mangle)]
pub extern "C" fn _start(fb: &'static FramebufferInfo) -> ! {
    let renderer = SimplifiedRenderer::new(fb);
    // Initialize the global renderer
    RENDERER.call_once(|| Mutex::new(renderer));

    enable(); // Enable interrupts
    enable_apic(); // Enable the APIC
    bk_interrupts::init_idt(); // Initialize the IDT

    loop {
        // Test the global one
        get_and_lock_renderer().clear_screen();
        get_and_lock_renderer().print("Meow");

        get_and_lock_renderer().clear_screen();
        get_and_lock_renderer().show_alphabet();
        get_and_lock_renderer().render_content();
        get_and_lock_renderer().show_watermark();
        get_and_lock_renderer().print("meow");

        let mut message: String<32> = String::new();
        let range = 0..=80;
        for i in range.into_iter() {
            message.clear();
            message.push_str("Text :P ").unwrap();
            append_number_to_string(&mut message, i);

            // Access the renderer through the helper function
            get_and_lock_renderer().print(&message);
        }

        let words = "Hello, world! This is a test of the text rendering system. Let's see how it handles this long string of text.";
        for word in words.split_whitespace() {
            get_and_lock_renderer().print(word);
            sleep(100);
        }
    }

    loop {
        unsafe { asm!("hlt") }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(panic: &core::panic::PanicInfo) -> ! {
    use gop_render::CURSOR_STATE;
    unsafe {
        gop_render::CURSOR_STATE.force_unlock(); // rare case the item is locked
    }
    CURSOR_STATE.lock().x = 40;
    CURSOR_STATE.lock().y = 40;

    let mut message: String<128> = String::new();
    message
        .push_str(panic.message().as_str().unwrap_or("unknown error"))
        .unwrap();

    if let Some(location) = panic.location() {
        message.push_str(" at ").unwrap();

        // Ensure the file name is valid UTF-8
        if let Ok(file_name) = core::str::from_utf8(location.file().as_bytes()) {
            message.push_str(file_name).unwrap();
        } else {
            message.push_str("<invalid file>").unwrap();
        }

        message.push_str(": Line ").unwrap(); // Add "Line" label

        let mut line_str: String<16> = String::new();
        append_number_to_string(&mut line_str, location.line() as usize);
        message.push_str(&line_str).unwrap();
    }

    get_and_lock_renderer().clear_screen();
    get_and_lock_renderer().println(&message);

    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}

#[unsafe(no_mangle)]
pub unsafe fn memset(dest: *mut u8, value: u8, count: usize) {
    let mut ptr = dest;
    unsafe {
        for _ in 0..count {
            ptr::write(ptr, value);
            ptr = ptr.add(1);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let a = unsafe { ptr::read(s1.add(i)) };
        let b = unsafe { ptr::read(s2.add(i)) };
        if a != b {
            return a as i32 - b as i32;
        }
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe fn memcpy(dest: *mut u8, src: *const u8, count: usize) {
    for i in 0..count {
        unsafe { ptr::write(dest.add(i), ptr::read(src.add(i))) };
    }
}

#[unsafe(no_mangle)]
pub unsafe fn memmove(dest: *mut u8, src: *const u8, count: usize) {
    if dest as usize <= src as usize || dest as usize >= src as usize + count {
        // Non-overlapping regions, can copy forward
        for i in 0..count {
            unsafe { ptr::write(dest.add(i), ptr::read(src.add(i))) };
        }
    } else {
        // Overlapping regions, copy backward
        for i in (0..count).rev() {
            unsafe { ptr::write(dest.add(i), ptr::read(src.add(i))) };
        }
    }
}

/// Reads the CPU's timestamp counter
fn read_timestamp_counter() -> u64 {
    let mut low: u32;
    let mut high: u32;
    unsafe {
        asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// Busy-wait loop to sleep for the specified number of milliseconds
pub fn sleep(milliseconds: u64) {
    // Assuming a CPU frequency of 3 GHz (adjust as needed for your system)
    const CPU_FREQUENCY_HZ: u64 = 2_200_000_000;
    let cycles_per_ms = CPU_FREQUENCY_HZ / 1_000;

    let start = read_timestamp_counter();
    let target = start + (milliseconds * cycles_per_ms);

    while read_timestamp_counter() < target {
        // Busy-wait
    }
}

pub fn append_number_to_string<const N: usize>(s: &mut String<N>, num: usize) {
    let mut buffer = [0u8; 20]; // Enough to hold any usize
    let mut i = 0;
    let mut n = num;

    // Convert the number to a string in reverse order
    loop {
        buffer[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
        if n == 0 {
            break;
        }
    }

    // Append the digits in the correct order
    for j in (0..i).rev() {
        let _ = s.push(buffer[j] as char);
    }
}
