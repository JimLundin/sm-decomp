//! Verification guards for the palette and animated-tile runtimes, and the
//! Samus palette observer.
use crate::game_state::Game;

#[path = "../verification/animated_runtime_checks.rs"]
mod animation_guard_checks;
#[path = "../verification/palette_runtime_checks.rs"]
mod palette_guard_checks;
#[path = "../verification/samus_palette_observer.rs"]
pub(crate) mod samus_observer;

pub(crate) fn check_palette_guard(game: &mut Game) -> bool {
    palette_guard_checks::run(game)
}
pub(crate) fn check_animation_guard(game: &mut Game) -> bool {
    animation_guard_checks::run(game)
}
