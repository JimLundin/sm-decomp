mod legacy_inflate;

pub struct TextureImage {
    pub width: usize,
    pub height: usize,
    pub channels: usize,
    pub pixels: Vec<u8>,
}

fn decode_pixels(
    header: &[u8; 13],
    raw: &[u8],
    palette: &[[u8; 4]; 256],
    palette_alpha: bool,
) -> Option<Vec<u8>> {
    let width = u32::from_be_bytes(header[..4].try_into().ok()?) as usize;
    let height = u32::from_be_bytes(header[4..8].try_into().ok()?) as usize;
    let depth = usize::from(header[8]);
    let color = header[9];
    let channels = if color == 3 {
        1
    } else {
        (if color & 2 != 0 { 3 } else { 1 }) + usize::from(color & 4 != 0)
    };
    let sample_bytes = if depth == 16 { 2 } else { 1 };
    let pixel_bytes = if color == 3 {
        3 + usize::from(palette_alpha)
    } else {
        channels * sample_bytes
    };
    let mut pixels = vec![0; width * height * pixel_bytes];
    let adam7 = [
        (0, 0, 8, 8),
        (4, 0, 8, 8),
        (0, 4, 4, 8),
        (2, 0, 4, 4),
        (0, 2, 2, 4),
        (1, 0, 2, 2),
        (0, 1, 1, 2),
    ];
    let plain = [(0, 0, 1, 1)];
    let passes = if header[12] == 0 {
        &plain[..]
    } else {
        &adam7[..]
    };
    let mut position = 0;
    for &(x0, y0, dx, dy) in passes {
        let w = width.saturating_sub(x0).div_ceil(dx);
        let h = height.saturating_sub(y0).div_ceil(dy);
        if w == 0 || h == 0 {
            continue;
        }
        let row_bytes = (w * channels * depth).div_ceil(8);
        // The legacy decoder accepts low-bit multi-channel images only when
        // their packed row fits within the pass width.
        if depth < 8 && row_bytes > w {
            return None;
        }
        let filter_bytes = if depth < 8 {
            1
        } else {
            channels * sample_bytes
        };
        let mut row = vec![0u8; row_bytes];
        let mut previous = vec![0u8; row_bytes];
        for y in 0..h {
            let filter = *raw.get(position)?;
            if filter > 4 {
                return None;
            }
            position += 1;
            row.copy_from_slice(raw.get(position..position + row_bytes)?);
            position += row_bytes;
            for i in 0..row_bytes {
                let left = if i < filter_bytes {
                    0
                } else {
                    row[i - filter_bytes]
                };
                let above = previous[i];
                let upper_left = if i < filter_bytes {
                    0
                } else {
                    previous[i - filter_bytes]
                };
                let prediction = match filter {
                    0 => 0,
                    1 => left,
                    2 => above,
                    3 => ((u16::from(left) + u16::from(above)) / 2) as u8,
                    4 => {
                        let p = i32::from(left) + i32::from(above) - i32::from(upper_left);
                        let a = (p - i32::from(left)).abs();
                        let b = (p - i32::from(above)).abs();
                        let c = (p - i32::from(upper_left)).abs();
                        if a <= b && a <= c {
                            left
                        } else if b <= c {
                            above
                        } else {
                            upper_left
                        }
                    }
                    _ => unreachable!(),
                };
                row[i] = row[i].wrapping_add(prediction);
            }
            for x in 0..w {
                let start = ((y0 + y * dy) * width + x0 + x * dx) * pixel_bytes;
                let pixel = &mut pixels[start..start + pixel_bytes];
                if depth < 8 {
                    for channel in 0..channels {
                        let bit = (x * channels + channel) * depth;
                        let value = (row[bit / 8] >> (8 - depth - bit % 8)) & ((1 << depth) - 1);
                        if color == 3 {
                            pixel.copy_from_slice(&palette[usize::from(value)][..pixel_bytes]);
                        } else {
                            pixel[channel] = if color == 0 {
                                value * (255 / ((1 << depth) - 1))
                            } else {
                                value
                            };
                        }
                    }
                } else if color == 3 {
                    pixel.copy_from_slice(&palette[usize::from(row[x])][..pixel_bytes]);
                } else {
                    pixel.copy_from_slice(&row[x * pixel_bytes..(x + 1) * pixel_bytes]);
                }
            }
            std::mem::swap(&mut row, &mut previous);
        }
    }
    Some(pixels)
}

enum PngSource<'a> {
    Bytes(&'a [u8]),
    File(*mut crate::types::FILE),
}

impl PngSource<'_> {
    fn read(&mut self, output: &mut [u8]) -> usize {
        match self {
            Self::Bytes(bytes) => {
                let count = output.len().min(bytes.len());
                output[..count].copy_from_slice(&bytes[..count]);
                *bytes = &bytes[count..];
                count
            }
            Self::File(file) => unsafe {
                crate::c_library::fread(output.as_mut_ptr().cast(), 1, output.len(), *file) as usize
            },
        }
    }

    fn skip(&mut self, length: usize) {
        match self {
            Self::Bytes(bytes) => *bytes = bytes.get(length..).unwrap_or(&[]),
            Self::File(file) => unsafe {
                use crate::c_library::{fgetc, fseek, ungetc};
                fseek(*file, length as std::ffi::c_long, 1);
                let ch = fgetc(*file);
                if ch != -1 {
                    ungetc(ch, *file);
                }
            },
        }
    }
}

// Match the legacy 128-byte refills, direct IDAT reads and signed chunk skips.
struct ChunkInput<'a> {
    source: PngSource<'a>,
    buffer: [u8; 128],
    position: usize,
    end: usize,
    eof: bool,
    scratch: Vec<u8>,
}

impl<'a> ChunkInput<'a> {
    fn new(source: PngSource<'a>) -> Self {
        Self {
            source,
            buffer: [0; 128],
            position: 0,
            end: 0,
            eof: false,
            scratch: Vec::new(),
        }
    }

    fn read(&mut self, length: usize) -> Option<&[u8]> {
        self.scratch.clear();
        while self.scratch.len() < length {
            if self.position == self.end {
                if self.eof {
                    return None;
                }
                self.end = self.source.read(&mut self.buffer);
                self.position = 0;
                self.eof = self.end == 0;
                if self.eof {
                    return None;
                }
            }
            let count = (length - self.scratch.len()).min(self.end - self.position);
            self.scratch
                .extend_from_slice(&self.buffer[self.position..self.position + count]);
            self.position += count;
        }
        Some(&self.scratch)
    }

    fn data(&mut self, length: usize) -> Option<&[u8]> {
        self.scratch.resize(length, 0);
        let buffered = length.min(self.end - self.position);
        self.scratch[..buffered]
            .copy_from_slice(&self.buffer[self.position..self.position + buffered]);
        self.position += buffered;
        if buffered < length && self.source.read(&mut self.scratch[buffered..]) != length - buffered
        {
            return None;
        }
        Some(&self.scratch)
    }

    fn skip(&mut self, length: u32) {
        if (length as i32) < 0 {
            self.position = self.end;
        } else {
            let buffered = (length as usize).min(self.end - self.position);
            self.position += buffered;
            if buffered < length as usize {
                self.source.skip(length as usize - buffered);
            }
        }
    }
}

pub fn decode_texture_png(input: &[u8]) -> Option<TextureImage> {
    decode(&mut ChunkInput::new(PngSource::Bytes(input)))
}

fn decode(input: &mut ChunkInput<'_>) -> Option<TextureImage> {
    for &byte in b"\x89PNG\r\n\x1a\n" {
        if input.read(1)?[0] != byte {
            return None;
        }
    }
    let mut header = None;
    let mut palette = [[0u8; 4]; 256];
    let mut palette_length = 0;
    let mut palette_alpha = false;
    let mut transparency: Option<Vec<u8>> = None;
    let mut compressed = Vec::new();
    let mut iphone = false;
    loop {
        let length = u32::from_be_bytes(input.read(4)?.try_into().ok()?) as usize;
        let kind: [u8; 4] = input.read(4)?.try_into().ok()?;
        if &kind == b"IEND" {
            if header.is_none() || compressed.is_empty() {
                return None;
            }
            break;
        }
        if &kind == b"CgBI" {
            iphone = true;
            input.skip(length as u32);
        } else if &kind == b"IHDR" {
            if header.is_some() || length != 13 {
                return None;
            }
            let mut h = [0; 13];
            h[..8].copy_from_slice(input.read(8)?);
            let w = u32::from_be_bytes(h[..4].try_into().ok()?);
            let y = u32::from_be_bytes(h[4..8].try_into().ok()?);
            if w > 4096 || y > 4096 {
                return None;
            }
            // Reject each field before reading the next, including at refill boundaries.
            for i in 8..13 {
                h[i] = input.read(1)?[0];
                let valid = match i {
                    8 => [1, 2, 4, 8, 16].contains(&h[i]),
                    9 => [0, 2, 3, 4, 6].contains(&h[i]) && !(h[i] == 3 && h[8] == 16),
                    10 | 11 => h[i] == 0,
                    _ => h[i] <= 1,
                };
                if !valid {
                    return None;
                }
            }
            if w == 0 || y == 0 {
                return None;
            }
            header = Some(h);
        } else {
            let h = header.as_ref()?;
            match &kind {
                b"PLTE" => {
                    if length > 768 || length % 3 != 0 {
                        return None;
                    }
                    let data = input.read(length)?;
                    palette_length = length / 3;
                    for (index, color) in data.chunks_exact(3).enumerate() {
                        palette[index][..3].copy_from_slice(color);
                        palette[index][3] = 255;
                    }
                }
                b"tRNS" => {
                    if !compressed.is_empty() {
                        return None;
                    }
                    if h[9] == 3 {
                        if palette_length == 0 || length > palette_length {
                            return None;
                        }
                        palette_alpha = true;
                        for (p, alpha) in palette.iter_mut().zip(input.read(length)?) {
                            p[3] = *alpha;
                        }
                    } else {
                        let channels = if h[9] == 0 {
                            1
                        } else if h[9] == 2 {
                            3
                        } else {
                            return None;
                        };
                        if length != 2 * channels {
                            return None;
                        }
                        transparency = Some(input.read(length)?.to_vec());
                    }
                }
                b"IDAT" => {
                    if h[9] == 3 && palette_length == 0 {
                        return None;
                    }
                    if compressed.len().checked_add(length)? > i32::MAX as usize {
                        return None;
                    }
                    compressed.extend_from_slice(input.data(length)?);
                }
                _ => {
                    if kind[0] & 0x20 == 0 {
                        return None;
                    }
                    input.skip(length as u32);
                }
            }
        }
        input.read(4)?;
    }
    let header = header?;
    let width = u32::from_be_bytes(header[..4].try_into().ok()?) as usize;
    let height = u32::from_be_bytes(header[4..8].try_into().ok()?) as usize;
    let depth = header[8];
    let color = header[9];
    let input_channels = if color == 3 {
        1
    } else {
        (if color & 2 != 0 { 3 } else { 1 }) + usize::from(color & 4 != 0)
    };
    let initial_size = (width * usize::from(depth) + 7) / 8 * height * input_channels + height;
    let raw = legacy_inflate::inflate(&compressed, initial_size, !iphone)?;
    let channels = match color {
        0 => 1,
        2 => 3,
        3 => {
            if palette_alpha {
                4
            } else {
                3
            }
        }
        4 => 2,
        6 => 4,
        _ => return None,
    };
    let pixels = decode_pixels(&header, &raw, &palette, palette_alpha)?;
    // IEND's length is ignored; its CRC is read only after successful decoding.
    // Legacy get8 supplies zeroes at EOF, so a missing CRC is accepted.
    let _ = input.read(4);
    let sample_bytes = if depth == 16 { 2 } else { 1 };
    let mut out =
        Vec::with_capacity(width * height * (channels + usize::from(transparency.is_some())));
    for pixel in pixels.chunks_exact(channels * sample_bytes) {
        out.extend(pixel.iter().step_by(sample_bytes));
        if let Some(transparency) = &transparency {
            let matches = if depth == 16 {
                pixel == transparency
            } else {
                let scale = if depth < 8 {
                    255 / ((1u16 << depth) - 1)
                } else {
                    1
                };
                pixel
                    .iter()
                    .zip(transparency.chunks_exact(2))
                    .all(|(v, t)| *v == t[1].wrapping_mul(scale as u8))
            };
            out.push(if matches { 0 } else { 255 });
        }
    }
    Some(TextureImage {
        width,
        height,
        channels: channels + usize::from(transparency.is_some()),
        pixels: out,
    })
}

/// Read texture paths through the same C runtime as preset loading.
pub(super) unsafe fn load(filename: *const std::ffi::c_char) -> Option<TextureImage> {
    use crate::c_library::{fclose, fopen, fseek};
    let file = fopen(filename, b"rb\0".as_ptr().cast());
    if file.is_null() {
        return None;
    }
    let mut input = ChunkInput::new(PngSource::File(file));
    let image = decode(&mut input);
    if image.is_some() {
        fseek(file, -((input.end - input.position) as std::ffi::c_long), 1);
    }
    fclose(file);
    image
}
