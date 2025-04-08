#![no_std]
#![no_main]

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::{arch::asm, ptr};

use framebuffer::FramebufferInfo;
use gop_render::SimplifiedRenderer;
mod font;
mod framebuffer;
mod gop_render;
mod watermark;

extern crate alloc;

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

#[cfg(test)]
mod tests {
    use super::*;
    use core::alloc::Layout;

    #[test]
    fn test_memset() {
        let mut buffer = [0u8; 10];
        unsafe { memset(buffer.as_mut_ptr(), 0xFF, buffer.len()) };
        assert_eq!(buffer, [0xFF; 10]);
    }

    #[test]
    fn test_memcmp() {
        let a = [1, 2, 3];
        let b = [1, 2, 3];
        assert_eq!(unsafe { memcmp(a.as_ptr(), b.as_ptr(), a.len()) }, 0);
    }

    #[test]
    fn test_allocator_alloc() {
        let layout = Layout::from_size_align(16, 8).unwrap();
        unsafe {
            let ptr = GLOBAL.alloc(layout);
            assert!(!ptr.is_null(), "Allocation failed");
            GLOBAL.dealloc(ptr, layout);
        }
    }

    #[test]
    fn test_allocator_multiple_allocations() {
        let layout1 = Layout::from_size_align(16, 8).unwrap();
        let layout2 = Layout::from_size_align(32, 8).unwrap();
        unsafe {
            let ptr1 = GLOBAL.alloc(layout1);
            assert!(!ptr1.is_null(), "First allocation failed");

            let ptr2 = GLOBAL.alloc(layout2);
            assert!(!ptr2.is_null(), "Second allocation failed");

            GLOBAL.dealloc(ptr1, layout1);
            GLOBAL.dealloc(ptr2, layout2);
        }
    }

    #[test]
    fn test_allocator_out_of_memory() {
        let layout = Layout::from_size_align(HEAP_SIZE + 1, 8).unwrap();
        unsafe {
            let ptr = GLOBAL.alloc(layout);
            assert!(ptr.is_null(), "Allocation should fail when out of memory");
        }
    }

    #[test]
    fn test_allocator_alignment() {
        let layout = Layout::from_size_align(16, 16).unwrap();
        unsafe {
            let ptr = GLOBAL.alloc(layout);
            assert!(!ptr.is_null(), "Allocation failed");
            assert_eq!(ptr as usize % 16, 0, "Pointer is not properly aligned");
            GLOBAL.dealloc(ptr, layout);
        }
    }
}
