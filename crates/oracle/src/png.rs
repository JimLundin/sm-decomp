//! The lsnes picture's transport: lsnes's `bitmap:save_png()` gives the
//! screenshot as base64 PNG text, decoded here to 8-bit RGB. Only what that
//! encoder writes is read: 8-bit RGB or RGBA, not interlaced, one zlib stream
//! over the IDAT chunks. Written from RFC 1950, RFC 1951, RFC 4648 and the PNG
//! specification; no emulator code.

/// Base64 text (RFC 4648, with or without padding) to bytes.
pub fn base64(text: &[u8]) -> Option<Vec<u8>> {
    let value = |symbol: u8| -> Option<u32> {
        let index = match symbol {
            b'A'..=b'Z' => symbol - b'A',
            b'a'..=b'z' => symbol - b'a' + 26,
            b'0'..=b'9' => symbol - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        Some(u32::from(index))
    };
    let text = text.trim_ascii_end();
    let body = text
        .strip_suffix(b"==")
        .or_else(|| text.strip_suffix(b"="))
        .unwrap_or(text);
    let mut out = Vec::with_capacity(body.len() / 4 * 3 + 2);
    for group in body.chunks(4) {
        let mut bits = 0u32;
        for &symbol in group {
            bits = (bits << 6) | value(symbol)?;
        }
        let bytes = (bits << (6 * (4 - group.len()))).to_be_bytes();
        let count = match group.len() {
            4 => 3,
            3 => 2,
            2 => 1,
            _ => return None,
        };
        out.extend_from_slice(&bytes[1..=count]);
    }
    Some(out)
}

/// A PNG's size and dots as 8-bit R, G, B, row by row.
pub fn rgb(png: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    const SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    let mut rest = png.strip_prefix(SIGNATURE)?;
    let (mut width, mut height, mut channels) = (0usize, 0usize, 0usize);
    let mut stream = Vec::new();
    while let Some((length, after)) = rest.split_first_chunk::<4>() {
        let length = usize::try_from(u32::from_be_bytes(*length)).ok()?;
        let (kind, after) = after.split_first_chunk::<4>()?;
        let (data, after) = after.split_at_checked(length)?;
        rest = after.get(4..)?;
        match kind {
            b"IHDR" => {
                let header: &[u8; 13] = data.try_into().ok()?;
                let size = |at: usize| {
                    let field: [u8; 4] = header[at..at + 4].try_into().ok()?;
                    usize::try_from(u32::from_be_bytes(field)).ok()
                };
                (width, height) = (size(0)?, size(4)?);
                channels = match (header[8], header[9], header[12]) {
                    (8, 2, 0) => 3,
                    (8, 6, 0) => 4,
                    _ => return None,
                };
            }
            b"IDAT" => stream.extend_from_slice(data),
            b"IEND" => break,
            _ => {}
        }
    }
    let rows = inflate(stream.get(2..)?)?;
    let stride = width.checked_mul(channels)?;
    if rows.len() != height.checked_mul(stride + 1)? {
        return None;
    }
    let mut dots = vec![0u8; height * stride];
    for (line, row) in rows.chunks_exact(stride + 1).enumerate() {
        let (&filter, row) = row.split_first()?;
        let (done, current) = dots.split_at_mut(line * stride);
        let above = done
            .get(done.len().saturating_sub(stride)..)
            .filter(|_| line > 0);
        let current = &mut current[..stride];
        for (x, &raw) in row.iter().enumerate() {
            let left = x.checked_sub(channels).map_or(0, |at| current[at]);
            let up = above.map_or(0, |above| above[x]);
            let corner = above
                .zip(x.checked_sub(channels))
                .map_or(0, |(above, at)| above[at]);
            let predictor = match filter {
                0 => 0,
                1 => left,
                2 => up,
                3 => u8::try_from((u16::from(left) + u16::from(up)) / 2).ok()?,
                4 => paeth(left, up, corner),
                _ => return None,
            };
            current[x] = raw.wrapping_add(predictor);
        }
    }
    if channels == 4 {
        dots = dots
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|&[r, g, b, _]| [r, g, b])
            .collect();
    }
    Some((width, height, dots))
}

fn paeth(left: u8, up: u8, corner: u8) -> u8 {
    let (a, b, c) = (i16::from(left), i16::from(up), i16::from(corner));
    let estimate = a + b - c;
    let (pa, pb, pc) = (
        (estimate - a).abs(),
        (estimate - b).abs(),
        (estimate - c).abs(),
    );
    if pa <= pb && pa <= pc {
        left
    } else if pb <= pc {
        up
    } else {
        corner
    }
}

/// The bits of a deflate stream, least significant first.
struct Bits<'a> {
    bytes: &'a [u8],
    at: usize,
    bit: u32,
    count: u32,
}

impl Bits<'_> {
    fn take(&mut self, need: u32) -> Option<u32> {
        while self.count < need {
            let byte = *self.bytes.get(self.at)?;
            self.at += 1;
            self.bit |= u32::from(byte) << self.count;
            self.count += 8;
        }
        let value = self.bit & ((1u32 << need) - 1);
        self.bit = self.bit.checked_shr(need).unwrap_or(0);
        self.count -= need;
        Some(value)
    }

    fn align(&mut self) {
        self.bit = 0;
        self.count = 0;
    }
}

/// A canonical Huffman code: the count of codes of each length and the
/// symbols in code order.
struct Huffman {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl Huffman {
    fn new(lengths: &[u8]) -> Self {
        let mut counts = [0u16; 16];
        for &length in lengths {
            counts[usize::from(length)] += 1;
        }
        counts[0] = 0;
        let mut offsets = [0u16; 16];
        for length in 1..16 {
            offsets[length] = offsets[length - 1] + counts[length - 1];
        }
        let mut symbols = vec![0u16; lengths.len()];
        for (symbol, &length) in (0u16..).zip(lengths) {
            if length != 0 {
                let slot = &mut offsets[usize::from(length)];
                symbols[usize::from(*slot)] = symbol;
                *slot += 1;
            }
        }
        Self { counts, symbols }
    }

    fn decode(&self, bits: &mut Bits) -> Option<u16> {
        let (mut code, mut first, mut index) = (0i32, 0i32, 0i32);
        for &count in &self.counts[1..] {
            code |= i32::try_from(bits.take(1)?).ok()?;
            let count = i32::from(count);
            if code - first < count {
                return self
                    .symbols
                    .get(usize::try_from(index + code - first).ok()?)
                    .copied();
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }
        None
    }
}

const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DISTANCE_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
/// The order the code-length code's lengths are sent in.
const LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

/// A raw deflate stream (RFC 1951) to its bytes.
fn inflate(stream: &[u8]) -> Option<Vec<u8>> {
    let mut bits = Bits {
        bytes: stream,
        at: 0,
        bit: 0,
        count: 0,
    };
    let mut out = Vec::new();
    loop {
        let last = bits.take(1)? == 1;
        match bits.take(2)? {
            0 => {
                bits.align();
                let header = stream.get(bits.at..bits.at + 4)?;
                let length = usize::from(u16::from_le_bytes([header[0], header[1]]));
                bits.at += 4;
                out.extend_from_slice(stream.get(bits.at..bits.at + length)?);
                bits.at += length;
            }
            1 => {
                let mut lengths = [0u8; 288];
                lengths[..144].fill(8);
                lengths[144..256].fill(9);
                lengths[256..280].fill(7);
                lengths[280..].fill(8);
                let literals = Huffman::new(&lengths);
                let distances = Huffman::new(&[5u8; 30]);
                block(&mut bits, &mut out, &literals, &distances)?;
            }
            2 => {
                let (literals, distances) = dynamic_codes(&mut bits)?;
                block(&mut bits, &mut out, &literals, &distances)?;
            }
            _ => return None,
        }
        if last {
            return Some(out);
        }
    }
}

fn dynamic_codes(bits: &mut Bits) -> Option<(Huffman, Huffman)> {
    let literal_count = usize::try_from(bits.take(5)?).ok()? + 257;
    let distance_count = usize::try_from(bits.take(5)?).ok()? + 1;
    let code_count = usize::try_from(bits.take(4)?).ok()? + 4;
    let mut code_lengths = [0u8; 19];
    for &slot in LENGTH_ORDER.iter().take(code_count) {
        code_lengths[slot] = u8::try_from(bits.take(3)?).ok()?;
    }
    let code = Huffman::new(&code_lengths);
    let mut lengths = Vec::with_capacity(literal_count + distance_count);
    while lengths.len() < literal_count + distance_count {
        let symbol = code.decode(bits)?;
        let (value, repeat) = match symbol {
            0..=15 => (u8::try_from(symbol).ok()?, 1),
            16 => (*lengths.last()?, 3 + bits.take(2)?),
            17 => (0, 3 + bits.take(3)?),
            18 => (0, 11 + bits.take(7)?),
            _ => return None,
        };
        for _ in 0..repeat {
            lengths.push(value);
        }
    }
    if lengths.len() != literal_count + distance_count {
        return None;
    }
    let (literals, distances) = lengths.split_at(literal_count);
    Some((Huffman::new(literals), Huffman::new(distances)))
}

fn block(
    bits: &mut Bits,
    out: &mut Vec<u8>,
    literals: &Huffman,
    distances: &Huffman,
) -> Option<()> {
    loop {
        let symbol = literals.decode(bits)?;
        match symbol {
            0..=255 => out.push(u8::try_from(symbol).ok()?),
            256 => return Some(()),
            _ => {
                let index = usize::from(symbol - 257);
                let length = usize::from(*LENGTH_BASE.get(index)?)
                    + usize::try_from(bits.take(u32::from(LENGTH_EXTRA[index]))?).ok()?;
                let index = usize::from(distances.decode(bits)?);
                let distance = usize::from(*DISTANCE_BASE.get(index)?)
                    + usize::try_from(bits.take(u32::from(DISTANCE_EXTRA[index]))?).ok()?;
                let from = out.len().checked_sub(distance)?;
                for at in from..from + length {
                    out.push(out[at]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trips_rfc_4648_vectors() {
        assert_eq!(base64(b"Zm9vYmFy").as_deref(), Some(&b"foobar"[..]));
        assert_eq!(base64(b"Zm9vYg==").as_deref(), Some(&b"foob"[..]));
        assert_eq!(base64(b"Zm9vYmE=").as_deref(), Some(&b"fooba"[..]));
        assert_eq!(base64(b"Zg==").as_deref(), Some(&b"f"[..]));
    }

    #[test]
    fn inflate_reads_stored_and_fixed_blocks() {
        // A stored block "abc"; "abcabcabc" as zlib level 9 deflates it.
        assert_eq!(
            inflate(&[1, 3, 0, 0xFC, 0xFF, b'a', b'b', b'c']).as_deref(),
            Some(&b"abc"[..])
        );
        assert_eq!(
            inflate(&[0x4B, 0x4C, 0x4A, 0x4E, 0x04, 0x23, 0x00]).as_deref(),
            Some(&b"abcabcabc"[..])
        );
    }
}
