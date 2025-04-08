use core::slice::from_raw_parts_mut;

use crate::{font::PSF2Font, framebuffer::FramebufferInfo, watermark::parse_ppm};

/// Graphics abstraction for the buffer graphics
pub struct SimplifiedRenderer<'a> {
    buffer: &'a FramebufferInfo,
}

/// Enum representing colors
pub enum Color {
    Black = 0x000000,
    White = 0xFFFFFF,
    Red = 0xFF0000,
    Green = 0x00FF00,
    Blue = 0x0000FF,
    Yellow = 0xFFFF00,
}

impl Color {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

impl<'a> SimplifiedRenderer<'a> {
    pub fn new(buffer: &'a FramebufferInfo) -> SimplifiedRenderer<'a> {
        Self { buffer }
    }

    pub fn clear_screen(&self) {
        unsafe {
            for i in 0..(self.buffer.size / 4) {
                *(self.buffer.address as *mut u32).add(i) = Color::Black.as_u32();
            }
        }
    }

    pub fn render_content(&self) {
        let width = self.buffer.width;
        let stride = self.buffer.stride;

        draw_rectangle(
            self.buffer.address as *mut u32,
            width,
            stride,
            50,
            50,
            100,
            50,
            Color::Red.as_u32(),
        );
        draw_rectangle(
            self.buffer.address as *mut u32,
            width,
            stride,
            200,
            100,
            80,
            120,
            Color::Green.as_u32(),
        );
        draw_rectangle(
            self.buffer.address as *mut u32,
            width,
            stride,
            300,
            200,
            60,
            60,
            Color::Blue.as_u32(),
        );

        draw_rectangle(
            self.buffer.address as *mut u32,
            width,
            stride,
            0,
            0,
            20,
            20,
            Color::Red.as_u32(),
        );
        draw_rectangle(
            self.buffer.address as *mut u32,
            width,
            stride,
            self.buffer.width - 20,
            0,
            20,
            20,
            Color::Red.as_u32(),
        );
        draw_rectangle(
            self.buffer.address as *mut u32,
            width,
            stride,
            0,
            self.buffer.height - 20,
            20,
            20,
            Color::Red.as_u32(),
        );
        draw_rectangle(
            self.buffer.address as *mut u32,
            width,
            stride,
            self.buffer.width - 20,
            self.buffer.height - 20,
            20,
            20,
            Color::Red.as_u32(),
        );

        draw_border(
            self.buffer.address as *mut u32,
            width,
            stride,
            self.buffer.height,
            Color::Yellow.as_u32(),
        );
    }

    pub fn show_alphabet(&self) {
        let font = crate::font::load_font().unwrap();
        let letter_width = font.header.width as usize;

        // Convert the buffer into a mutable slice (WILL NOT WORK OTHERWISE)
        let buffer_slice =
            unsafe { from_raw_parts_mut(self.buffer.address as *mut u32, self.buffer.size / 4) };

        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        for (i, &ch) in ALPHABET.iter().enumerate() {
            draw_char(
                buffer_slice,
                self.buffer.width,
                10 + (i * letter_width),
                10,
                Color::White.as_u32(),
                Color::Black.as_u32(),
                &font,
                ch,
            );
        }
    }

    pub fn show_watermark(&self) {
        const WATERMARK_BYTES: &[u8] = include_bytes!("../art/boykisser.ppm");
        let ppm = parse_ppm(WATERMARK_BYTES).unwrap();
        let width = ppm.width;
        let height = ppm.height;
        let pixel_data = ppm.data;

        let fb_width = self.buffer.width;
        let fb_height = self.buffer.height;
        let fb_stride = self.buffer.stride;
        let buffer_slice =
            unsafe { from_raw_parts_mut(self.buffer.address as *mut u32, self.buffer.size / 4) };

        for y in 0..height {
            for x in 0..width {
                let fb_x = fb_width - width + x;
                let fb_y = fb_height - height + y;
                if fb_x < fb_width && fb_y < fb_height {
                    let index = fb_y * fb_stride + fb_x;
                    let pixel_index = (y * width + x) * 3;
                    if pixel_index + 2 < pixel_data.len() {
                        let r = pixel_data[pixel_index] as u32;
                        let g = pixel_data[pixel_index + 1] as u32;
                        let b = pixel_data[pixel_index + 2] as u32;
                        let color = (0xFF << 24) | (r << 16) | (g << 8) | b;
                        buffer_slice[index] = color;
                    }
                }
            }
        }
    }
}

fn draw_rectangle(
    fb: *mut u32,
    _fb_width: usize,
    stride: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: u32,
) {
    for row in 0..height {
        for col in 0..width {
            let px = x + col;
            let py = y + row;
            unsafe {
                *fb.add(py * stride + px) = color;
            }
        }
    }
}

fn draw_border(fb: *mut u32, width: usize, stride: usize, height: usize, color: u32) {
    for x in 0..width {
        unsafe {
            *fb.add(x) = color;
            *fb.add((height - 1) * stride + x) = color;
        }
    }

    for y in 0..height {
        unsafe {
            *fb.add(y * stride) = color;
            *fb.add(y * stride + (width - 1)) = color;
        }
    }
}

pub fn draw_char(
    framebuffer: &mut [u32],
    framebuffer_width: usize,
    x: usize,
    y: usize,
    color: u32,
    bg_color: u32,
    font: &PSF2Font,
    ch: u8,
) {
    if let Some(glyph) = font.glyph(ch as u32) {
        let bytes_per_row = (font.header.width + 7) / 8;
        let height = font.header.height as usize;
        let width = font.header.width as usize;

        for row in 0..height {
            let row_offset = row * bytes_per_row as usize;

            for col in 0..width {
                let byte = glyph.get(row_offset + (col / 8)).copied().unwrap_or(0);
                let bit = 7 - (col % 8);
                let on = (byte >> bit) & 1;

                let fb_x = x + col;
                let fb_y = y + row;

                if fb_x < framebuffer_width && fb_y * framebuffer_width + fb_x < framebuffer.len() {
                    let index = fb_y * framebuffer_width + fb_x;
                    framebuffer[index] = if on != 0 { color } else { bg_color };
                }
            }
        }
    }
}
