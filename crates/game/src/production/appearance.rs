//! Read the existing clean animation state at its preceding movement opportunity.

/// The complete causal animation feedback, without advancing the playhead.
#[must_use]
pub fn ahead(
    state: &samus_appearance::State,
    clip: &samus_appearance::Clip,
) -> samus_appearance::Ahead {
    samus_appearance::Ahead::of(state.playhead, clip)
}
