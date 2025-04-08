#![no_std]
#![no_main]
#![allow(dead_code)]

use core::{arch::asm, ptr};

use framebuffer::FramebufferInfo;
use gop_render::SimplifiedRenderer;
mod font;
mod framebuffer;
mod gop_render;

#[unsafe(no_mangle)] // THIS HAS TO BE &FrameBufferInfo or it WILL NOT WORK
pub extern "C" fn _start(fb: &FramebufferInfo) -> ! {
    // B background
    let renderer = SimplifiedRenderer::new(fb); // Pass fb directly
    renderer.clear_screen();

    // Render some text (Fixed to the top left)
    renderer.show_alphabet("Short test");

    // Render some extra garbage to see that it worked.
    // renderer.render_content();

    loop {
        unsafe { asm!("hlt") }
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
