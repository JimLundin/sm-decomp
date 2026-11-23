//! Her clip's read-ahead and the touchdown's graphics work.

/// The read-ahead: the end her clip reaches and whether her spin has turned far, asked of her
/// whole appearance State and her clip as the frame starts, without changing either.
#[must_use]
pub fn ahead(
    state: &samus_appearance::AppearanceState,
    clip: &samus_appearance::Clip,
) -> samus_appearance::Ahead {
    samus_appearance::Ahead::of(state, clip)
}

/// Independently admitted area selection at the touchdown graphics entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchdownArea {
    /// The dry room arm that clears the two landing-atmosphere slots.
    DryClear,
}

/// Graphics work published by Appearance at the touchdown opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchdownGraphics {
    /// Clear both landing-atmosphere type slots.
    ClearLandingSlots {
        /// Figure selected by the completed pose fit.
        figure: samus_appearance::Figure,
    },
}

pub(super) const fn touchdown_graphics(
    figure: samus_appearance::Figure,
    area: TouchdownArea,
) -> TouchdownGraphics {
    match area {
        TouchdownArea::DryClear => TouchdownGraphics::ClearLandingSlots { figure },
    }
}
