//! Verification guards for the palette and animated-tile runtimes, and the
//! Samus palette observer.
use crate::game_state::Game;

#[cfg(not(sm_host))]
#[path = "../../../../sm/src/verification/animated_runtime_checks.rs"]
mod animation_guard_checks;
#[cfg(not(sm_host))]
#[path = "../../../../sm/src/verification/palette_runtime_checks.rs"]
mod palette_guard_checks;
#[cfg(not(sm_host))]
#[path = "../../../../sm/src/verification/samus_palette_observer.rs"]
pub(crate) mod samus_observer;

#[cfg(not(sm_host))]
pub(crate) fn check_palette_guard(game: &mut Game) -> bool {
    palette_guard_checks::run(game)
}
#[cfg(not(sm_host))]
pub(crate) fn check_animation_guard(game: &mut Game) -> bool {
    animation_guard_checks::run(game)
}
