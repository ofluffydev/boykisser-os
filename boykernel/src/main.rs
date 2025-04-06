#![no_std]
#![no_main]

use core::{arch::asm, ptr};

#[repr(C)]
pub struct FramebufferInfo {
    pub address: u64,
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub format: u32,
}

pub const FONT: [[u8; 8]; 9] = [
    // H
    [0x42, 0x42, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x00],
    // E
    [0x7E, 0x40, 0x40, 0x7C, 0x40, 0x40, 0x7E, 0x00],
    // L
    [0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x7E, 0x00],
    // L
    [0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x7E, 0x00],
    // O
    [0x3C, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00],
    // (space)
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    // A
    [0x18, 0x24, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x00],
    // R
    [0x7C, 0x42, 0x42, 0x7C, 0x48, 0x44, 0x42, 0x00],
    // Y
    [0x42, 0x42, 0x24, 0x18, 0x18, 0x18, 0x18, 0x00],
];

#[unsafe(no_mangle)]
pub extern "C" fn _start(fb: &FramebufferInfo) -> ! {
    let fb_ptr = fb.address as *mut u32;
    let width = fb.width;
    let stride = fb.stride;

    // Black background
    unsafe {
        for i in 0..(fb.size / 4) {
            *fb_ptr.add(i) = 0x000000; // black
        }
    }

    // White pixels for text
    let color = 0xFFFFFF;

    for (i, glyph) in FONT.iter().enumerate() {
        draw_char(fb_ptr, width, stride, 10 + (i * 10), 10, glyph, color);
    }

    loop {
        unsafe { asm!("hlt") }
    }
}

fn draw_char(
    fb: *mut u32,
    _fb_width: usize, // Prefix unused variable with an underscore
    stride: usize,
    x_offset: usize,
    y_offset: usize,
    bitmap: &[u8; 8],
    color: u32,
) {
    for (y, row) in bitmap.iter().enumerate() {
        for x in 0..8 {
            if (row >> (7 - x)) & 1 != 0 {
                let px = x_offset + x;
                let py = y_offset + y;
                unsafe {
                    *fb.add(py * stride + px) = color;
                }
            }
        }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
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
