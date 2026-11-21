//! Saved current external movement premises, separate from the actual model state.

use canonical::{Canonical, model::Pinned};
use serde::{Deserialize, Serialize};
use types::{Buttons, ClipEnd, FiringDirection, Item, Parity, Pose, Radius, Set, Sprite};

/// Complete owned movement inputs at the next actual opportunity.
/// The caller updates inputs only from their admitted current writers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovementInputs {
    /// Appearance figure captured before movement preparation.
    pub sprite_before_movement: Sprite,
    /// Current mode's cinematic activity.
    pub cinematic_active: bool,
    /// Current committed held controls.
    pub pad: Buttons,
    /// Current committed newly pressed controls.
    pub pressed: Buttons,
    /// Current progress equipment output.
    pub items: Set<Item>,
    /// Current completed spin-animation query.
    pub spin_turned_far: bool,
    /// Current elevator producer output.
    pub elevator_moving: bool,
    /// Current frame parity.
    pub parity: Parity,
    /// Actual completed frame-start animation query.
    pub animation_ends: Option<ClipEnd>,
}

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

/// Owned current movement input schema, including independently captured appearance facts.
pub struct MovementInputsPin;
impl Pinned for MovementInputsPin {
    type State = MovementInputs;
    const NAME: &'static str = "production-frame-movement-inputs";
    const SHAPE: u64 = 0x1073_CEEA_7B2B_8D9B;
}

impl MovementInputs {
    /// Borrow the current inputs with actual imported content adapters.
    /// No gameplay decision or query is performed by borrowing them.
    #[must_use]
    pub const fn frame<'a>(
        &self,
        body: &'a dyn Fn(Pose) -> Radius,
        cannon_heading: &'a dyn Fn(Pose) -> Option<FiringDirection>,
    ) -> samus_movement::Frame<'a> {
        samus_movement::Frame {
            sprite_before_movement: self.sprite_before_movement,
            cinematic_active: self.cinematic_active,
            pad: self.pad,
            pressed: self.pressed,
            items: self.items,
            spin_turned_far: self.spin_turned_far,
            elevator_moving: self.elevator_moving,
            parity: self.parity,
            animation_ends: self.animation_ends,
            body,
            cannon_heading,
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

/// Actual playing pose and sprite, independently retained before later pose selection.
pub struct FigurePin;
impl Pinned for FigurePin {
    type State = samus_appearance::Figure;
    const NAME: &'static str = "production-frame-playing-figure";
    const SHAPE: u64 = 0x4301_5F68_4F3E_58BA;
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
