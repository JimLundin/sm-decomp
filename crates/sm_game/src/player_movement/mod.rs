//! Normal locomotion and shinespark crash transitions share the owned player state.
pub mod content;
pub mod effects;
pub mod forced;
pub mod presentation;
pub mod selectors;
pub mod speed;

pub use presentation::{record_echo, xray_animation_frame};

pub fn advance_normal(h: &mut impl effects::Host) {
    if h.inputs().frozen == 0 {
        selectors::advance(h);
        presentation::record_echo(h);
        h.probe("player.movement.normal.returned");
    }
}
