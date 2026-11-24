//! The touchdown's graphics work.

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
