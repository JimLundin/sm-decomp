//! Apply the warning's drawing outputs to carried room memory.

use super::{Fault, warning::Output};
use crate::exact::ridley_warning;
use types::Colour;
use world_doors::Palette;

pub(super) fn publish(vram: &mut Vec<u16>, warning: Option<&Output>) -> Result<(), Fault> {
    let Some(warning) =
        warning.filter(|warning| !warning.art.is_empty() || !warning.text.is_empty())
    else {
        return Ok(());
    };
    let mut updated = vram.clone();
    for patch in &warning.art {
        ridley_warning::sheet(&mut updated, patch).ok_or(Fault::RidleyWarning)?;
    }
    for glyph in &warning.text {
        ridley_warning::letter(&mut updated, glyph).ok_or(Fault::RidleyWarning)?;
    }
    *vram = updated;
    Ok(())
}

const ENGLISH: std::ops::Range<usize> = 81..84;
const HEADING: std::ops::Range<usize> = 97..100;

/// The warning text's last NMI palette publication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextColours {
    live: [[Colour; 3]; 2],
    published: [[Colour; 3]; 2],
}

impl Default for TextColours {
    fn default() -> Self {
        Self {
            live: [[Colour::BLACK; 3]; 2],
            published: [[Colour::BLACK; 3]; 2],
        }
    }
}

impl TextColours {
    /// Read the room's current warning palette and its last published values.
    ///
    /// # Errors
    /// The room must provide both warning palette ranges.
    pub fn advance(&mut self, palette: &Palette, publishes: bool) -> Result<(), Fault> {
        self.live = [
            row(palette, ENGLISH).ok_or(Fault::RidleyWarning)?,
            row(palette, HEADING).ok_or(Fault::RidleyWarning)?,
        ];
        if publishes {
            self.published = self.live;
        }
        Ok(())
    }

    /// The two prepared or physically published text palette ranges.
    #[must_use]
    pub const fn values(&self) -> ([[Colour; 3]; 2], [[Colour; 3]; 2]) {
        (self.live, self.published)
    }
}

fn row(palette: &Palette, span: std::ops::Range<usize>) -> Option<[Colour; 3]> {
    palette.screen.get(span)?.try_into().ok()
}
