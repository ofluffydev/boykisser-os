use crate::font::Font;
use crate::framebuffer::FramebufferInfo;

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
        let color = 0xFFFFFF;

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
        for (i, glyph) in FONT.iter().enumerate() {
            draw_char(
                self.buffer.address as *mut u32,
                width,
                stride,
                10 + (i * 10),
                10,
                glyph,
                color,
            );
        }

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

    pub fn test_text(&self, text: &str) {
        static FONT_DATA: &[u8] = include_bytes!("../spleen-2.1.0/spleen-8x16.psfu");
        let font = Font::from_bytes(FONT_DATA).expect("Invalid PSF2 font");

        let stride = self.buffer.stride;
        let width = self.buffer.width;
        let color = 0xFFFFFF;

        let mut x_offset = 10;
        let mut y_offset = 10;

        for ch in text.bytes() {
            // Prevent text from spilling beyond buffer
            if x_offset + font.header.width as usize >= width {
                x_offset = 10;
                y_offset += font.header.height as usize + 1;
            }
            if let Some(glyph) = font.get_glyph(ch as usize) {
                draw_psf2_char(
                    self.buffer.address as *mut u32,
                    width,
                    stride,
                    x_offset,
                    y_offset,
                    glyph,
                    font.header.width as usize,
                    font.header.height as usize,
                    color,
                );
                x_offset += font.header.width as usize + 1; // Add spacing
            }
        }
    }
}

fn draw_char(
    fb: *mut u32,
    _fb_width: usize,
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

fn draw_psf2_char(
    fb: *mut u32,
    _fb_width: usize,
    stride: usize,
    x_offset: usize,
    y_offset: usize,
    glyph: &[u8],
    _glyph_width: usize,
    glyph_height: usize,
    color: u32,
) {
    for y in 0..glyph_height {
        let row = glyph[y]; // 1 byte per row
        for x in 0..8 {
            let bit = (row >> (7 - x)) & 1;
            if bit != 0 {
                let px = x_offset + x;
                let py = y_offset + y;
                unsafe {
                    *fb.add(py * stride + px) = color; // Use stride directly
                }
            }
        }
    }
}
