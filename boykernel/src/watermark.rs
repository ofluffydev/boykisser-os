use arrayvec::ArrayVec;

#[allow(dead_code)]
/// A small struct to hold the parsed result.
pub struct PpmImage<'a> {
    /// Width of the image in pixels.
    pub width: usize,
    /// Width of the image in pixels.
    pub height: usize,
    /// Maximum value for pixel data (usually 255).
    pub max_val: u16,
    /// Pointer into the original buffer for pixel data (no copying).
    pub data: &'a [u8],
}

/// Possible errors your parser may emit.
#[derive(Debug)]
pub enum PpmParseError {
    /// The magic number was not "P6".
    InvalidMagicNumber,
    /// The header was incomplete.
    HeaderIncomplete,
    /// The number was invalid (e.g., too large).
    InvalidNumber,
    /// The number was not a valid ASCII number.
    UnexpectedEndOfData,
}

/// Parse a P6 (binary) PPM from raw bytes.
pub fn parse_ppm(bytes: &[u8]) -> Result<PpmImage, PpmParseError> {
    let mut tokens: ArrayVec<&[u8], 32> = ArrayVec::new(); // from a crate like `arrayvec` if needed

    let mut idx = 0;
    while idx < bytes.len() {
        while idx < bytes.len() && is_whitespace(bytes[idx]) {
            idx += 1;
        }

        if idx >= bytes.len() {
            break;
        }

        if bytes[idx] == b'#' {
            while idx < bytes.len() && bytes[idx] != b'\n' {
                idx += 1;
            }
            continue;
        }

        let start = idx;
        while idx < bytes.len() && !is_whitespace(bytes[idx]) {
            idx += 1;
        }
        let token = &bytes[start..idx];
        if tokens.try_push(token).is_err() {
            break;
        }
    }

    if tokens.len() < 4 {
        return Err(PpmParseError::HeaderIncomplete);
    }

    if tokens[0] != b"P6" {
        return Err(PpmParseError::InvalidMagicNumber);
    }

    let width = parse_ascii_number(tokens[1])?;
    let height = parse_ascii_number(tokens[2])?;
    let max_val = parse_ascii_number(tokens[3])?;
    if max_val > u16::MAX as usize {
        return Err(PpmParseError::InvalidNumber);
    }

    let header_end_offset = find_offset_of(tokens[3], bytes) + tokens[3].len();

    let mut data_start = header_end_offset;
    while data_start < bytes.len() && is_whitespace(bytes[data_start]) {
        data_start += 1;
    }

    let total_pixels = width
        .checked_mul(height)
        .ok_or(PpmParseError::InvalidNumber)?;
    let expected_len = total_pixels
        .checked_mul(3)
        .ok_or(PpmParseError::InvalidNumber)?;
    if data_start + expected_len > bytes.len() {
        return Err(PpmParseError::UnexpectedEndOfData);
    }

    let data_slice = &bytes[data_start..data_start + expected_len];

    Ok(PpmImage {
        width,
        height,
        max_val: max_val as u16,
        data: data_slice,
    })
}

/// Minimal check for ASCII whitespace
fn is_whitespace(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r')
}

/// Parse an ASCII number from a &\[u8\] token
fn parse_ascii_number(bytes: &[u8]) -> Result<usize, PpmParseError> {
    let mut val: usize = 0;
    for &b in bytes {
        if !(b'0'..=b'9').contains(&b) {
            return Err(PpmParseError::InvalidNumber);
        }
        val = val.checked_mul(10).ok_or(PpmParseError::InvalidNumber)? + (b - b'0') as usize;
    }
    Ok(val)
}

/// Find the offset in `haystack` where `needle` begins, assuming `needle` is a subslice.
fn find_offset_of(needle: &[u8], haystack: &[u8]) -> usize {
    for i in 0..=haystack.len().saturating_sub(needle.len()) {
        if &haystack[i..(i + needle.len())] == needle {
            return i;
        }
    }
    0
}
