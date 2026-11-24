//! The Ceres doors: their run with the room's enemies, and the boxes her moves meet.
//!
//! The doors run before she moves, reading where she was at the end of the last frame; her
//! motion meets the boxes of those that were solid as the frame began.

use super::{Fault, station};
use crate::production::world::MovementWorld;
use boss_ceres_ridley::{CeresDoor, Stage};
use types::{Hitbox, Pair, Parts};

impl station::State {
    /// The room's `geometry`, with the Ceres doors that block her.
    pub(super) fn world<'a>(&self, geometry: &'a MovementWorld) -> WithDoors<'a> {
        WithDoors::new(geometry, &self.doors)
    }
}

/// One run of each of the room's Ceres doors in `station`, in its order of its enemies,
/// reading where `her` was at the end of the last frame. In the escape a later piece owns the
/// shaft's overlay.
pub(super) fn run(
    station: &station::State,
    her: &samus_movement::Samus,
    ridley_beaten: bool,
    random: types::RandomSequence,
    rules: &boss_ceres_ridley::Rules,
) -> Result<Vec<boss_ceres_ridley::Run>, Fault> {
    let stage = station.stage;
    if stage == Stage::Escape && (station.shaft_overlay || !station.doors.is_empty()) {
        return Err(Fault::Escape);
    }
    let inputs = boss_ceres_ridley::Inputs {
        her: Pair {
            x: her.position.x.pixel(),
            y: her.position.y.pixel(),
        },
        stage,
        random: random.number.0,
        // Outside the escape no door reads the screen's shaking, and the escape is refused.
        shake_frames_left: 0,
        // In his room Ridley runs first, and his run in this piece leaves him unbeaten.
        ridley_beaten,
    };
    Ok(station
        .doors
        .iter()
        .map(|door| door.run(&inputs, rules))
        .collect())
}

/// The room's geometry, with the Ceres doors that block her after its other solid actors.
///
/// The original meets the boxes in slot order, and the first that blocks her decides the move.
/// The geometry's boxes keep no slots, so the doors go last: the admission accepts only rooms
/// whose other solid actors sit in slots before the doors'.
pub(super) struct WithDoors<'a> {
    geometry: &'a MovementWorld,
    solid: Vec<Hitbox>,
}

impl<'a> WithDoors<'a> {
    fn new(geometry: &'a MovementWorld, doors: &[CeresDoor]) -> Self {
        let blocking = doors.iter().filter(|door| door.phase.solid());
        Self {
            geometry,
            solid: types::World::solid_enemies(geometry)
                .iter()
                .cloned()
                .chain(blocking.map(|door| door.hitbox()))
                .collect(),
        }
    }
}

impl types::World for WithDoors<'_> {
    fn block(&self, cell: types::Cell) -> types::Block {
        types::World::block(self.geometry, cell)
    }

    fn lets_through(&self, cell: types::Cell, way: types::Way) -> bool {
        types::World::lets_through(self.geometry, cell, way)
    }

    fn solid_enemies(&self) -> &[Hitbox] {
        &self.solid
    }
}

#[cfg(test)]
mod tests {
    use crate::production::played::Fault;
    use crate::production::played::tests::{play, state};
    use crate::production::tests::fixtures;
    use boss_ceres_ridley::{CeresDoor, DoorKind, Phase, Quarter, Stage, Wait};
    use core::num::Wrapping;
    use types::{Pair, Parts, Side};

    /// A Ceres door touching her right side: her box reaches 5 px right of her, the door's 8 px
    /// left of its centre.
    fn door(phase: Phase) -> CeresDoor {
        let her = fixtures::samus().position;
        CeresDoor {
            kind: DoorKind::Door,
            facing: Side::Left,
            position: Pair {
                x: Wrapping(her.x.pixel().0.wrapping_add(13)),
                y: her.y.pixel(),
            },
            phase,
        }
    }

    /// A door on the last frame of its opening, which its run this frame leaves open.
    fn opening_ends() -> Phase {
        Phase::Opening {
            quarter: Quarter::Fourth,
            wait: Wait::new(1).unwrap(),
        }
    }

    #[test]
    fn a_door_that_opens_this_frame_still_blocks_her_moves() {
        let mut blocked = state(vec![door(opening_ends())]);
        play(&mut blocked, types::button::RIGHT, Stage::Calm).unwrap();
        assert!(!blocked.station.doors[0].phase.solid());
        let mut open = state(vec![door(Phase::Open {
            wait: Wait::new(2).unwrap(),
        })]);
        play(&mut open, types::button::RIGHT, Stage::Calm).unwrap();
        let start = fixtures::samus().position.x;
        assert!(open.samus.movement.0.position.x.0 > start.0);
        assert!(blocked.samus.movement.0.position.x.0 < open.samus.movement.0.position.x.0);
    }

    #[test]
    fn the_escape_refuses_a_room_with_ceres_doors_and_leaves_state_as_it_was() {
        let mut escaping = state(vec![door(opening_ends())]);
        escaping.station.stage = Stage::Escape;
        let before = escaping.clone();
        assert_eq!(
            play(&mut escaping, types::Buttons::default(), Stage::Escape),
            Err(Fault::Escape)
        );
        assert_eq!(escaping, before);
    }
}
