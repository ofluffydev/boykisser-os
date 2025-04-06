use uefi::{
    boot::{get_handle_for_protocol, open_protocol_exclusive},
    proto::console::gop::{self, GraphicsOutput},
};
use log::info;

#[repr(C)]
#[derive(Debug)]
pub struct FramebufferInfo {
    pub address: u64,
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub format: u32, // You can define enum values for known formats
}

pub fn initialize_framebuffer() -> FramebufferInfo {
    let gop_handle = get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop_protocol = open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
    let gop = gop_protocol.get_mut().unwrap();
    let mode_info = gop.current_mode_info();
    let resolution = mode_info.resolution();
    let stride = mode_info.stride();
    let pixel_format = mode_info.pixel_format();

    let mut gop_buffer = gop.frame_buffer();
    let gop_buffer_first_byte = gop_buffer.as_mut_ptr() as usize;

    info!("Framebuffer address: 0x{:x}", gop_buffer_first_byte);
    info!("Framebuffer size: {} bytes", gop_buffer.size());

    FramebufferInfo {
        address: gop_buffer.as_mut_ptr() as u64,
        size: gop_buffer.size(),
        width: resolution.0,
        height: resolution.1,
        stride,
        format: match pixel_format {
            gop::PixelFormat::Rgb => 0,
            gop::PixelFormat::Bgr => 1,
            gop::PixelFormat::Bitmask => 2,
            gop::PixelFormat::BltOnly => 3,
        },
    }
}
