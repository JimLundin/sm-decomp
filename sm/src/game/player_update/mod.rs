//! Ordered player beta work over the same owned Session.
#![forbid(unsafe_code)]

pub(crate) mod content;
pub(crate) mod effects;
pub(crate) mod minimap;
pub(crate) mod state;

use crate::game::{player_movement, player_transitions as transitions, samus_periodic_damage};
use effects::{GammaAction, Host};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BetaMode {
    Gameplay,
    DemoMapped,
    DemoUnmapped,
    AnimationOnly,
}

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
    mode: BetaMode,
    movement: MovementStep,
) -> Result<(), samus_periodic_damage::InvalidDamage> {
    if movement == MovementStep::Normal {
        player_movement::advance_normal(h);
    }
    if mode != BetaMode::DemoUnmapped {
        minimap::update(h);
    }
    // Both movement and map projection can change the selected gamma action.
    if mode == BetaMode::Gameplay {
        if let GammaAction::Legacy(selector) = h.gamma_action() {
            h.run_gamma(selector);
        }
    }
    if mode == BetaMode::AnimationOnly {
        transitions::advance_animation(h, transitions::state::AnimationStep::Gameplay);
    } else {
        transitions::finish_movement_and_palette(h);
        if mode == BetaMode::Gameplay {
            // Gamma and settlement effects refresh this observation on return.
            let frozen = h.inputs().frozen;
            let result = samus_periodic_damage::apply(h.session_mut(), frozen);
            if result.is_ok() {
                h.probe("player.update.returned");
            }
            return result;
        }
    }
    h.probe("player.update.returned");
    Ok(())
}
