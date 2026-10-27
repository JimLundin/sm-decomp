//! Shared finite layout calculations; callers retain acquisition and write phases.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxSize {
    Small,
    Large,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextCursor {
    pub word: u16,
    pub source_offset: u16,
    remaining: i32,
}
impl TextCursor {
    pub fn new(length_bytes: u16) -> Self {
        Self {
            word: 32,
            source_offset: 0,
            remaining: i32::from(length_bytes >> 1),
        }
    }
    /// Call only after the tile query, destination check and committed write.
    /// Zero/odd lengths retain the original do-until behavior.
    pub fn advance(&mut self) -> bool {
        self.word = self.word.wrapping_add(1);
        self.source_offset = self.source_offset.wrapping_add(2);
        self.remaining -= 1;
        self.remaining == 0
    }
}

pub fn destination_byte_offset(word: u16) -> usize {
    let offset = 2 * (usize::from(word) + 256);
    let _ = &[(); 0x800][offset..offset + 2];
    offset
}

pub fn dma_length(length_bytes: u16) -> u16 {
    length_bytes.wrapping_add(128)
}

const SPECIAL_OFFSETS: [u16; 27] = [
    0, 0x12a, 0x12a, 0x12c, 0x12c, 0x12c, 0, 0, 0, 0, 0, 0, 0x120, 0, 0, 0, 0, 0, 0x12a, 0, 0, 0,
    0, 0, 0, 0, 0,
];

const LETTERS: [u16; 8] = [
    0x28e0, 0x3ce1, 0x2cf7, 0x38f8, 0x38d0, 0x38eb, 0x38f1, 0x284e,
];

const LARGE_BORDER: [u16; 32] = [
    0xe, 0xe, 0xe, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x280f, 0x280f, 0x280f, 0x280f,
    0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f,
    0x280f, 0x280f, 0x284e, 0x284e, 0xe, 0xe, 0xe,
];

const SMALL_BORDER: [u16; 32] = [
    0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e,
    0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0xe,
    0xe, 0xe, 0xe, 0xe, 0xe, 0xe,
];

const SELECTION: [u16; 96] = [
    0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0x3c4e, 0x3c4e, 0x38cc, 0x38cd, 0x3cf8, 0x3ce4, 0x3cf2, 0x3c4e,
    0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x2ced, 0x2cee, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0xe,
    0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0x3c4e, 0x3c4e, 0x38cc, 0x38cd,
    0x3cf8, 0x3ce4, 0x3cf2, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x2ced, 0x2cee, 0x3c4e,
    0x3c4e, 0x3c4e, 0x3c4e, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe,
    0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x2cf8, 0x2ce4, 0x2cf2, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x38cc,
    0x38cd, 0x3ced, 0x3cee, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe,
];

pub fn border(size: BoxSize) -> &'static [u16; 32] {
    match size {
        BoxSize::Small => &SMALL_BORDER,
        BoxSize::Large => &LARGE_BORDER,
    }
}

pub fn button_letter(buttons: u16) -> u16 {
    let masks = [0x80, 0x8000, 0x40, 0x4000, 0x2000, 0x20, 0x10];
    LETTERS[masks
        .iter()
        .position(|mask| buttons & mask != 0)
        .unwrap_or(7)]
}

pub fn special_button_offset(message_zero_based: isize) -> usize {
    usize::from(SPECIAL_OFFSETS[message_zero_based as usize])
}

pub fn selection_row(selection: u16) -> &'static [u16; 32] {
    let first = if selection == 2 { 64 } else { 32 };
    SELECTION[first..first + 32].try_into().unwrap()
}

pub fn map_offset(size: BoxSize) -> u16 {
    match size {
        BoxSize::Small => 448,
        BoxSize::Large => 416,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RadiusValues {
    pub top_scroll: u16,
    pub bottom_scroll: u16,
    /// Final source scratch words at05a4,05a6,05a8,05aa, in write order.
    pub scratch: [u16; 4],
}

pub fn radius_values(radius: u16) -> RadiusValues {
    let upper = 0x7b00u16.wrapping_sub(radius) >> 8;
    let lower = radius.wrapping_add(0x7c00) >> 8;
    RadiusValues {
        top_scroll: 99u16.wrapping_sub(upper),
        bottom_scroll: 148u16.wrapping_sub(lower),
        scratch: [lower.wrapping_add(30), 178, upper.wrapping_sub(30), 69],
    }
}

pub const HDMA_DESCRIPTOR: [u8; 7] = [0xff, 0x00, 0x30, 0xe1, 0xfe, 0x30, 0x00];

/// Fixed indirect two-byte transfers:127 repeating rows at3000, then97 at30fe.
/// This maps emitted scroll words; it does not replace them with a rectangle.
pub fn scanline_scroll(values: RadiusValues, line_zero_based: usize) -> u16 {
    let _ = &[(); 224][line_zero_based];
    let block = if line_zero_based < usize::from(HDMA_DESCRIPTOR[0] & 0x7f) {
        0
    } else {
        3
    };
    let start = if block == 0 {
        0
    } else {
        usize::from(HDMA_DESCRIPTOR[0] & 0x7f)
    };
    let control = HDMA_DESCRIPTOR[block];
    let pointer = u16::from_le_bytes([HDMA_DESCRIPTOR[block + 1], HDMA_DESCRIPTOR[block + 2]]);
    let word = usize::from(pointer - 0x3000) / 2
        + if control & 0x80 != 0 {
            line_zero_based - start
        } else {
            0
        };
    match word {
        94..124 => values.top_scroll,
        124..154 => values.bottom_scroll,
        _ => 0,
    }
}
