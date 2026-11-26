//! The warning picture's carried tile words and prepared and published text palettes.

use super::{Fault, warning::Output};
use crate::exact::ridley_warning;
use crate::state::video::VRAM_WORDS;
use types::Colour;
use world_doors::Palette;

const ENGLISH: std::ops::Range<usize> = 81..84;
const HEADING: std::ops::Range<usize> = 97..100;

/// Carried room memory used by the warning drawing consumer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drawing {
    vram: Vec<u16>,
    live: [[Colour; 3]; 2],
    published: [[Colour; 3]; 2],
}

impl Drawing {
    /// A live drawing or an admitted room image.
    ///
    /// Returns `None` when an admitted room picture has the wrong memory size.
    #[must_use]
    pub fn admit(vram: Option<&[u16]>) -> Option<Self> {
        if vram.is_some_and(|vram| vram.len() != VRAM_WORDS) {
            return None;
        }
        Some(Self {
            vram: vram.map_or_else(|| vec![0; VRAM_WORDS], <[u16]>::to_vec),
            live: [[Colour::BLACK; 3]; 2],
            published: [[Colour::BLACK; 3]; 2],
        })
    }

    /// Apply the room's warning effects and commit the palette when the platform publishes.
    ///
    /// # Errors
    ///
    /// A picture update cannot be represented in the admitted frame.
    pub fn advance(
        &mut self,
        warning: Option<&Output>,
        palette: &Palette,
        publishes: bool,
    ) -> Result<(), Fault> {
        if let Some(warning) = warning {
            for patch in &warning.art {
                ridley_warning::sheet(&mut self.vram, patch).ok_or(Fault::RidleyWarning)?;
            }
            for glyph in &warning.text {
                ridley_warning::letter(&mut self.vram, glyph).ok_or(Fault::RidleyWarning)?;
            }
        }
        self.live = [
            row(palette, ENGLISH).ok_or(Fault::RidleyWarning)?,
            row(palette, HEADING).ok_or(Fault::RidleyWarning)?,
        ];
        if publishes {
            self.published = self.live;
        }
        Ok(())
    }

    /// Carried video memory after the warning's art and text writes.
    #[must_use]
    pub fn vram(&self) -> &[u16] {
        &self.vram
    }

    /// The two prepared or physically published text palette ranges.
    #[must_use]
    pub const fn text_colours(&self) -> ([[Colour; 3]; 2], [[Colour; 3]; 2]) {
        (self.live, self.published)
    }
}

fn row(palette: &Palette, span: std::ops::Range<usize>) -> Option<[Colour; 3]> {
    palette.screen.get(span)?.try_into().ok()
}
