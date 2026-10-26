//! Normal locomotion and shinespark crash transitions share the owned player state.
#![forbid(unsafe_code)]
pub(crate) mod content;
pub(crate) mod effects;
pub(crate) mod forced;
mod presentation;
mod selectors;
pub(crate) mod speed;

pub(crate) use presentation::{record_echo, xray_animation_frame};

pub(crate) fn advance_normal(h: &mut impl effects::Host) {
    if h.inputs().frozen == 0 {
        selectors::advance(h);
        presentation::record_echo(h);
        h.probe("player.movement.normal.returned");
    }
}
