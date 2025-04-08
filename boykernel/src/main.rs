#![no_std]
#![no_main]

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::{arch::asm, ptr};

use framebuffer::FramebufferInfo;
use gop_render::SimplifiedRenderer;
mod font;
mod framebuffer;
mod gop_render;
mod watermark;

extern crate alloc;

const HEAP_SIZE: usize = 4096;
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
static mut OFFSET: usize = 0;

struct GayAllocator;

#[allow(static_mut_refs)]
unsafe impl GlobalAlloc for GayAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        let mut current = unsafe { OFFSET };
        if current % align != 0 {
            current += align - (current % align);
        }
        if current + size > HEAP_SIZE {
            return null_mut();
        }
        let allocated_ptr;
        unsafe {
            allocated_ptr = HEAP.as_mut_ptr().add(current);
            OFFSET = current + size;
        }
        allocated_ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        let offset_ptr = unsafe { HEAP.as_mut_ptr().add(OFFSET) };
        if unsafe { ptr.add(size) } == offset_ptr {
            unsafe { OFFSET -= size };
        }
    }
}

#[global_allocator]
static GLOBAL: GayAllocator = GayAllocator;

#[unsafe(no_mangle)] // THIS HAS TO BE &FrameBufferInfo or it WILL NOT WORK
pub extern "C" fn _start(fb: &FramebufferInfo) -> ! {
    let renderer = SimplifiedRenderer::new(fb); // Pass fb directly
    renderer.clear_screen();
    renderer.render_content();
    renderer.show_alphabet();
    renderer.show_watermark();
    // TODO: add some kind of "sleep" function
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

#[unsafe(no_mangle)]
pub unsafe fn memcmp(a: *const u8, b: *const u8, count: usize) -> i32 {
    for i in 0..count {
        unsafe {
            let va = *a.add(i);
            let vb = *b.add(i);
            if va < vb {
                return -1;
            } else if va > vb {
                return 1;
            }
        }
    }
    0
}
