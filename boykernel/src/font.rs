#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PSF2Header {
    pub magic: u32,
    pub version: u32,
    pub headersize: u32,
    pub flags: u32,
    pub glyph_count: u32,
    pub bytes_per_glyph: u32,
    pub height: u32,
    pub width: u32,
}

// PSF2 magic number
const PSF2_MAGIC: u32 = 0x864ab572;

pub struct PSF2Font<'a> {
    pub header: PSF2Header,
    pub glyphs: &'a [u8],
}

impl<'a> PSF2Font<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Option<Self> {
        if bytes.len() < core::mem::size_of::<PSF2Header>() {
            return None;
        }

        // SAFETY: PSF2Header is repr(C) and all fields are plain integers.
        let header = unsafe { *(bytes.as_ptr() as *const PSF2Header) };

        if header.magic != PSF2_MAGIC {
            return None;
        }

        let glyph_data_offset = header.headersize as usize;
        let glyph_data_len = (header.glyph_count * header.bytes_per_glyph) as usize;

        if bytes.len() < glyph_data_offset + glyph_data_len {
            return None;
        }

        let glyphs = &bytes[glyph_data_offset..glyph_data_offset + glyph_data_len];
        Some(Self { header, glyphs })
    }

    pub fn glyph(&self, index: u32) -> Option<&'a [u8]> {
        if index >= self.header.glyph_count {
            return None;
        }
        let start = (index * self.header.bytes_per_glyph) as usize;
        let end = start + self.header.bytes_per_glyph as usize;
        self.glyphs.get(start..end)
    }
}

// Example usage
static FONT_DATA: &[u8] = include_bytes!("../spleen-2.1.0/spleen-32x64.psfu");

pub fn load_font() -> Option<PSF2Font<'static>> {
    PSF2Font::from_bytes(FONT_DATA)
}
