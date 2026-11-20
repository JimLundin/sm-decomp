//! Current external player premises and post-actor geometry at their actual opportunity.

use super::movement;
use crate::source;
use game::production::frame::inputs::MovementInputs;
use types::{Buttons, Item, Parity, Set, Sprite};

/// Complete external current quantities, excluding appearance feedback and future pose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExternalInputs {
    /// Current cinematic owner activity.
    pub cinematic_active: bool,
    /// Current committed held controls.
    pub pad: Buttons,
    /// Current committed newly pressed controls.
    pub pressed: Buttons,
    /// Current progress owner equipment.
    pub items: Set<Item>,
    /// Current elevator owner movement.
    pub elevator_moving: bool,
    /// Current completed-frame parity.
    pub parity: Parity,
}

/// Invalid independent current player input memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerInputFault {
    /// A complete current physical WRAM image is required at the source edge.
    Length,
    /// A required current source field is absent.
    Field,
    /// Complete independently accepted source provenance is absent or incompatible.
    Provenance,
}

#[path = "frame_player_inputs/source.rs"]
mod provenance;

/// Import the independently qualified current scalar and room dependency seed.
/// The loader pins the proof and origin and requires independent source acceptance.
/// Actual controls, world dependencies, appearance feedback and completed actor geometry
/// must still be joined at the selected production opportunity.
/// # Errors
/// Rejects incomplete descriptors, source mismatches or prohibited actor/appearance substitution.
pub fn qualified(bytes: &[u8], proof: &str) -> Result<ExternalInputs, PlayerInputFault> {
    provenance::validate(bytes, proof)?;
    current(bytes)
}

fn word(bytes: &[u8], field: source::WramWord) -> Result<u16, PlayerInputFault> {
    field.read(bytes).map_err(|_| PlayerInputFault::Field)
}

/// Decode current external player owners without reading pose, figure or playhead.
///
/// The caller admits each producer's currentness after S2 actors. Borrowing from
/// the origin separately requires a complete preceding-writer source proof.
/// # Errors
/// Rejects incomplete memory or missing current source fields.
pub fn current(bytes: &[u8]) -> Result<ExternalInputs, PlayerInputFault> {
    if bytes.len() != 131_072 {
        return Err(PlayerInputFault::Length);
    }
    Ok(ExternalInputs {
        cinematic_active: word(bytes, source::CINEMATIC_ACTIVE)? != 0,
        pad: movement::pad(word(bytes, source::JOYPAD1_LASTKEYS)?),
        pressed: movement::pad(word(bytes, source::JOYPAD1_NEWKEYS)?),
        items: movement::items(word(bytes, source::EQUIPPED_ITEMS)?),
        elevator_moving: word(bytes, source::ELEVATOR_STATUS)? != 0,
        parity: if word(bytes, source::NMI_FRAME_COUNTER_WORD)? & 1 == 0 {
            Parity::Even
        } else {
            Parity::Odd
        },
    })
}

impl ExternalInputs {
    /// Join actual current-playing appearance and its completed frame-start query.
    /// No observed or selected future figure is substituted for these real owners.
    #[must_use]
    pub const fn movement(
        self,
        playing_sprite: Sprite,
        ahead: samus_appearance::Ahead,
    ) -> MovementInputs {
        MovementInputs {
            sprite_before_movement: playing_sprite,
            cinematic_active: self.cinematic_active,
            pad: self.pad,
            pressed: self.pressed,
            items: self.items,
            spin_turned_far: ahead.turned_far,
            elevator_moving: self.elevator_moving,
            parity: self.parity,
            animation_ends: ahead.end,
        }
    }
}

#[cfg(test)]
#[path = "frame_player_inputs/tests.rs"]
mod tests;
