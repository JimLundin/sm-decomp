//! Saved current external movement premises, separate from the actual model state.

pub use crate::production::movement::{FigurePin, MovementInputs, MovementInputsPin};
use canonical::{Canonical, model::Pinned};
use types::{Parity, Sprite};

/// Current stable player modes and requested-NMI frame parity.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct InsidePremises {
    cinematic_active: bool,
    elevator_moving: bool,
    odd: bool,
}

impl InsidePremises {
    /// Retain the three independently current origin quantities.
    #[must_use]
    pub const fn new(inputs: &MovementInputs) -> Self {
        Self {
            cinematic_active: inputs.cinematic_active,
            elevator_moving: inputs.elevator_moving,
            odd: matches!(inputs.parity, Parity::Odd),
        }
    }

    pub(super) const fn complete_requested_nmi(&mut self) {
        self.odd = !self.odd;
    }

    pub(super) const fn at_inside(
        self,
        pose: &super::borrowed_player_preparation::PoseInputCompletion,
        environment: &super::borrowed_player_preparation::EnvironmentCompletion,
        playing_sprite: Sprite,
        ahead: samus_appearance::Ahead,
    ) -> MovementInputs {
        MovementInputs {
            sprite_before_movement: playing_sprite,
            cinematic_active: self.cinematic_active,
            pad: pose.terminal.held_actions,
            pressed: pose.terminal.pressed_actions,
            items: environment.dependency.equipment,
            spin_turned_far: ahead.turned_far,
            elevator_moving: self.elevator_moving,
            parity: if self.odd { Parity::Odd } else { Parity::Even },
            animation_ends: ahead.end,
        }
    }
}

/// Saved checked geometry, including eligible enemies in their current order.
pub struct WorldPin;
impl Pinned for WorldPin {
    type State = crate::production::world::MovementWorld;
    const NAME: &'static str = "production-frame-movement-world";
    const SHAPE: u64 = 0xC7D7_878F_E209_7FBE;
}

/// Actual action captured when movement preparation runs, before later pose changes.
pub struct PreparedActionPin;
impl Pinned for PreparedActionPin {
    type State = types::Action;
    const NAME: &'static str = "production-frame-prepared-action";
    const SHAPE: u64 = 0x9b77_4411_29f3_700e;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_movement_input_shape_remains_pinned() {
        assert_eq!(canonical::model::pin_holds::<MovementInputsPin>(), Ok(()));
    }

    #[test]
    fn current_geometry_shape_remains_pinned() {
        assert_eq!(canonical::model::pin_holds::<WorldPin>(), Ok(()));
    }

    #[test]
    fn prepared_action_shape_remains_pinned() {
        assert_eq!(canonical::model::pin_holds::<PreparedActionPin>(), Ok(()));
    }

    #[test]
    fn playing_figure_shape_remains_pinned() {
        assert_eq!(canonical::model::pin_holds::<FigurePin>(), Ok(()));
    }
}
