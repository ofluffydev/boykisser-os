#![no_std]
#![no_main]

use core::{arch::asm, ptr};
use framebuffer::FramebufferInfo;
use gop_render::SimplifiedRenderer;
use heapless::{String, Vec};
use spin::{Mutex, Once};

mod font;
mod framebuffer;
mod gop_render;
mod watermark;

// Global Once to hold the Mutex for the renderer
pub static RENDERER: Once<Mutex<SimplifiedRenderer>> = Once::new();

#[unsafe(no_mangle)] // THIS HAS TO BE &FrameBufferInfo or it WILL NOT WORK
pub extern "C" fn _start(fb: &'static FramebufferInfo) -> ! {
    let renderer = SimplifiedRenderer::new(fb);
    // Initialize the global renderer
    RENDERER.call_once(|| Mutex::new(renderer));

    loop {
        // Test the globabl one
        RENDERER.get().unwrap().lock().clear_screen();
        RENDERER.get().unwrap().lock().println("Meow");

        RENDERER.get().unwrap().lock().clear_screen();
        RENDERER.get().unwrap().lock().show_alphabet();
        RENDERER.get().unwrap().lock().render_content();
        RENDERER.get().unwrap().lock().show_watermark();
        RENDERER.get().unwrap().lock().println("meow");

        let mut message: String<32> = String::new();
        let range = 0..=80;
        for i in range.into_iter() {
            message.clear();
            message.push_str("Text :P ").unwrap();
            append_number_to_string(&mut message, i);

            // Access the renderer through the global Mutex
            if let Some(renderer) = RENDERER.get() {
                renderer.lock().println(&message);
            }
        }

        let words = "Hello, world! This is a test of the text rendering system. Let's see how it handles this long string of text.";
        for word in words.split_whitespace() {
            RENDERER.get().unwrap().lock().println(word);
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

    RENDERER.get().unwrap().lock().clear_screen();
    RENDERER.get().unwrap().lock().println(&message);

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
        s.push(buffer[j] as char);
    }
}
