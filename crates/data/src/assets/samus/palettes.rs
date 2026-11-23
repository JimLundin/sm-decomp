//! Her palettes (docs/architecture/assets.md §3.2, `SamusPalettes`): her suit's by suit, and
//! her shots' by the beams switched on.

pub use types::{Beam, Set};

use crate::assets::art::Palette;

/// Her suit's palette for each suit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Suits {
    /// The power suit, her first.
    pub power: Palette,
    /// The varia suit.
    pub varia: Palette,
    /// The gravity suit.
    pub gravity: Palette,
}

/// Her shots' palette while exactly these beams are switched on, the charge beam aside: it
/// never changes their colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeamPalette {
    /// The beams switched on, without the charge beam.
    pub beams: Set<Beam>,
    /// The palette.
    pub palette: Palette,
}

/// Her palettes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SamusPalettes {
    /// Her suit's, by suit.
    pub suits: Suits,
    /// Her shots', one for each combination of beams she can have switched on.
    pub beams: Vec<BeamPalette>,
}

impl SamusPalettes {
    /// Her shots' palette while `on` are switched on; `None` for a combination none is for.
    #[must_use]
    pub fn beam(&self, on: Set<Beam>) -> Option<&Palette> {
        let mut on = on;
        on[Beam::Charge] = false;
        self.beams
            .iter()
            .find(|entry| entry.beams == on)
            .map(|entry| &entry.palette)
    }
}
