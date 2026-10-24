//! Complete normal locomotion over the one owned player state.
#![forbid(unsafe_code)]
pub(crate) mod content;
pub(crate) mod effects;
mod presentation;
mod selectors;
pub(crate) mod speed;

pub(crate) use presentation::record_echo;

pub(crate) fn advance_normal(h: &mut impl effects::Host) {
    if h.inputs().frozen == 0 {
        selectors::advance(h);
        presentation::record_echo(h);
        h.probe("player.movement.normal.returned");
    }
}
