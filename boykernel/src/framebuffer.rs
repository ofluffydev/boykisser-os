#[repr(C)]
pub struct FramebufferInfo {
    pub address: u64,
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub format: u32,
}
