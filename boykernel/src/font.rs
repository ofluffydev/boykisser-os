pub struct Font<'a> {
    pub header: &'a Psf2Header,
    pub glyphs: &'a [u8],
}

#[repr(C)]
#[derive(Debug)]
pub struct Psf2Header {
    pub magic: [u8; 4],
    pub version: u32,
    pub headersize: u32,
    pub flags: u32,
    pub glyph_count: u32,
    pub bytes_per_glyph: u32,
    pub height: u32,
    pub width: u32,
}

impl<'a> Font<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Option<Self> {
        if data.len() < core::mem::size_of::<Psf2Header>() {
            return None;
        }

        let header = unsafe { &*(data.as_ptr() as *const Psf2Header) };

        if header.magic != [0x72, 0xb5, 0x4a, 0x86] {
            return None; // Invalid PSF2 magic number
        }

        let glyphs = &data[header.headersize as usize..];
        Some(Self { header, glyphs })
    }

    pub fn get_glyph(&self, index: usize) -> Option<&[u8]> {
        if index >= self.header.glyph_count as usize {
            return None;
        }

        let start = index * self.header.bytes_per_glyph as usize;
        let end = start + self.header.bytes_per_glyph as usize;
        Some(&self.glyphs[start..end])
    }
}
