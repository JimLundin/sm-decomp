//! Live foreign adapters over the shared Session projection.
#![forbid(unsafe_code)]
use super::host::Graphics;
use crate::game::session_runtime::Runtime;
use crate::ram_map::{word_arrays as A, words as W};
impl Graphics for Runtime<'_> {
    fn vram_write_queue_tail(&self) -> u16 {
        self.word(W::vram_write_queue_tail)
    }
    fn set_vram_write_queue_tail(&mut self, value: u16) {
        self.write_slice_le(W::vram_write_queue_tail, value);
    }
    fn set_vram_queue_size(&mut self, cursor: u16, value: u16) {
        self.write_slice_le(0xd0 + usize::from(cursor) + 0, value);
    }
    fn set_vram_queue_source(&mut self, cursor: u16, value: u16) {
        self.write_slice_le(0xd0 + usize::from(cursor) + 2, value);
    }
    fn set_vram_queue_bank_word(&mut self, cursor: u16, value: u16) {
        self.write_slice_le(0xd0 + usize::from(cursor) + 4, value);
    }
    fn set_vram_queue_destination(&mut self, cursor: u16, value: u16) {
        self.write_slice_le(0xd0 + usize::from(cursor) + 5, value);
    }
    fn set_bg2_y_scroll(&mut self, value: u16) {
        self.write_slice_le(W::bg2_y_scroll, value);
    }
    fn set_palette_buffer(&mut self, index: isize, value: u16) {
        if let Ok(slot) = usize::try_from(index) {
            if let Some(color) = self.session_mut().visual.colors.get_mut(slot) {
                *color = value;
                return;
            }
        }
        self.write_slice_le(
            A::palette_buffer
                .checked_add_signed(index.checked_mul(2).unwrap())
                .unwrap(),
            value,
        );
    }
}
