//! Shared finite layout calculations; callers retain acquisition and write phases.
#![forbid(unsafe_code)]

pub use sm_data::message::{
    BoxSize, HDMA_DESCRIPTOR, RadiusValues, border, map_offset, scanline_scroll, selection_row,
};

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

pub fn radius_values(radius: u16) -> RadiusValues {
    let upper = 0x7b00u16.wrapping_sub(radius) >> 8;
    let lower = radius.wrapping_add(0x7c00) >> 8;
    RadiusValues {
        top_scroll: 99u16.wrapping_sub(upper),
        bottom_scroll: 148u16.wrapping_sub(lower),
        scratch: [lower.wrapping_add(30), 178, upper.wrapping_sub(30), 69],
    }
}
