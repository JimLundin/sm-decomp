//! Ordered player beta work over the same owned Session.
#![forbid(unsafe_code)]

pub(crate) mod content;
pub(crate) mod effects;
pub(crate) mod minimap;
pub(crate) mod state;

use crate::game::{player_movement, player_transitions as transitions, samus_periodic_damage};
use effects::{GammaAction, Host};
use transitions::state::FrameBetaMode;

/// Chosen before entering Session ownership. Special movement has already run
/// at its legacy boundary; do not redispatch a handler changed by that movement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MovementStep {
    Normal,
    AlreadyAdvanced,
}

/// Contact is cleared by the entry adapter before selecting movement. The
/// caller regains authority before crash, pause, low-health or demo restoration.
pub(crate) fn advance(
    h: &mut impl Host,
    mode: FrameBetaMode,
    movement: MovementStep,
) -> Result<(), samus_periodic_damage::InvalidDamage> {
    if movement == MovementStep::Normal {
        player_movement::advance_normal(h);
    }
    if mode != FrameBetaMode::DemoUnmapped {
        minimap::update(h);
    }
    // Both movement and map projection can change the selected gamma action.
    if mode == FrameBetaMode::Gameplay {
        if let GammaAction::Legacy(selector) = h.gamma_action() {
            h.run_gamma(selector);
        }
    }
    if mode == FrameBetaMode::AnimationOnly {
        transitions::advance_animation(h, transitions::state::AnimationStep::Gameplay);
    } else {
        transitions::finish_movement_and_palette(h);
        if mode == FrameBetaMode::Gameplay {
            // Gamma and settlement effects refresh this observation on return.
            let frozen = h.inputs().frozen;
            return samus_periodic_damage::apply(h.session_mut(), frozen);
        }
    }
    Ok(())
}
