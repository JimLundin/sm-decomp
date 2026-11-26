//! The Ceres room's physical display writes.

use crate::state::video::{VRAM_WORDS, Video};
use data::assets::shaft::{OVERLAY_BYTES, Shaft};

const OVERLAY_START: usize = 0x7000;
const OVERLAY_END: usize = 0x7200;

/// The sprite graphics installed when the escape shaft places its overlay.
pub struct ShaftOverlay;

impl ShaftOverlay {
    /// Publish the complete sprite strip at actor placement, without touching the NMI batch.
    ///
    /// Returns `None` for an incomplete VRAM image, leaving it intact.
    #[must_use]
    pub fn publish(video: &mut Video, content: &Shaft) -> Option<()> {
        if video.memory.vram.len() != VRAM_WORDS {
            return None;
        }
        let words = video.memory.vram.get_mut(OVERLAY_START..OVERLAY_END)?;
        let bytes: &[u8; OVERLAY_BYTES] = &content.overlay;
        for (word, &source) in words.iter_mut().zip(bytes.as_chunks::<2>().0) {
            *word = u16::from_le_bytes(source);
        }
        Some(())
    }
}
