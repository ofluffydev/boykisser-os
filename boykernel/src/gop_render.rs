use core::slice::from_raw_parts_mut;

use crate::{font::PSF2Font, framebuffer::FramebufferInfo};

/// Graphics abstraction for the buffer graphics
pub struct SimplifiedRenderer<'a> {
    buffer: &'a FramebufferInfo,
}

impl<'a> SimplifiedRenderer<'a> {
    pub fn new(buffer: &'a FramebufferInfo) -> SimplifiedRenderer<'a> {
        Self { buffer }
    }

    pub fn clear_screen(&self) {
        unsafe {
            for i in 0..(self.buffer.size / 4) {
                *(self.buffer.address as *mut u32).add(i) = 0x000000; // black
            }
        }
    }

    pub fn render_content(&self) {
        let width = self.buffer.width;
        let stride = self.buffer.stride;

        // White pixels for text
        let _color = 0xFFFFFF;

        const FONT: [[u8; 8]; 5] = [
            [
                0b01111110, 0b10000001, 0b10000001, 0b10000001, 0b10000001, 0b10000001, 0b01111110,
                0b00000000,
            ], // H
            [
                0b00000000, 0b01111110, 0b00010000, 0b00010000, 0b00010000, 0b00010000, 0b01111110,
                0b00000000,
            ], // E
            [
                0b01111110, 0b10000000, 0b10000000, 0b01111110, 0b10000000, 0b10000000, 0b01111110,
                0b00000000,
            ], // L
            [
                0b01111110, 0b10000000, 0b10000000, 0b01111110, 0b10000000, 0b10000000, 0b01111110,
                0b00000000,
            ], // L
            [
                0b01111110, 0b10000001, 0b10000001, 0b10000001, 0b10000001, 0b10000001, 0b01111110,
                0b00000000,
            ], // O
        ];
        // for (i, glyph) in FONT.iter().enumerate() {
        //     draw_char(
        //         self.buffer.address as *mut u32,
        //         width,
        //         stride,
        //         10 + (i * 10),
        //         10,
        //         glyph,
        //         color,
        //     );
        // }

        // Draw random shapes
        draw_rectangle(
            self.buffer.address as *mut u32,
            width,
            stride,
            50,
            50,
            100,
            50,
            0xFF0000,
        ); // Red rectangle
        draw_rectangle(
            self.buffer.address as *mut u32,
            width,
            stride,
            200,
            100,
            80,
            120,
            0x00FF00,
        ); // Green rectangle
        draw_rectangle(
            self.buffer.address as *mut u32,
            width,
            stride,
            300,
            200,
            60,
            60,
            0x0000FF,
        ); // Blue square

        // Corner squares
        draw_rectangle(
            self.buffer.address as *mut u32,
            width,
            stride,
            0,
            0,
            20,
            20,
            0xFF0000,
        );
        draw_rectangle(
            self.buffer.address as *mut u32,
            width,
            stride,
            self.buffer.width - 20,
            0,
            20,
            20,
            0xFF0000,
        );
        draw_rectangle(
            self.buffer.address as *mut u32,
            width,
            stride,
            0,
            self.buffer.height - 20,
            20,
            20,
            0xFF0000,
        );
        draw_rectangle(
            self.buffer.address as *mut u32,
            width,
            stride,
            self.buffer.width - 20,
            self.buffer.height - 20,
            20,
            20,
            0xFF0000,
        );

        // Draw border around the framebuffer
        draw_border(
            self.buffer.address as *mut u32,
            width,
            stride,
            self.buffer.height,
            0xFFFF00,
        ); // Yellow border
    }

    pub fn show_alphabet(&self, _text: &str) {
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
                0xFFFFFFFF, // white text
                0x00000000, // black background
                &font,
                ch,
            );
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
    // Top and bottom borders
    for x in 0..width {
        unsafe {
            *fb.add(x) = color; // Top border
            *fb.add((height - 1) * stride + x) = color; // Bottom border (fixed)
        }
    }

    // Left and right borders
    for y in 0..height {
        unsafe {
            *fb.add(y * stride) = color; // Left border
            *fb.add(y * stride + (width - 1)) = color; // Right border
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
