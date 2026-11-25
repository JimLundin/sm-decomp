//! The Ceres doors: their run with the room's enemies, and the boxes her moves meet.
//!
//! The doors run before she moves, reading where she was at the end of the last frame; her
//! motion meets the boxes of those that were solid as the frame began.

use super::{Fault, station};
use crate::production::world::MovementWorld;
use boss_ceres_ridley::{CeresDoor, ShakeLayers, ShakeStart, Stage};
use types::{Hitbox, Pair, Parts};
use world_quake::{QuakeDirection, QuakeDuration, QuakeKind, QuakeReach, QuakeSize, ShakeOrder};

impl station::State {
    /// The room's `geometry`, with the Ceres doors that block her.
    pub(super) fn world<'a>(&self, geometry: &'a MovementWorld) -> WithDoors<'a> {
        WithDoors::new(geometry, &self.doors)
    }
}

/// One run of each of the room's Ceres doors in `station`, in its order of its enemies,
/// reading where `her` was at the end of the last frame, with the room's quake as `quake` holds
/// it. Door by door: each reads the shake's frames left as the doors before it left them, and
/// the shake it starts, if any, starts in `quake` before the next door runs. In the escape a
/// later piece owns the shaft's overlay.
pub(super) fn run(
    station: &station::State,
    her: &samus_movement::Samus,
    ridley_beaten: bool,
    random: types::RandomSequence,
    rules: &boss_ceres_ridley::Rules,
    quake: &mut world_quake::State,
) -> Result<Vec<boss_ceres_ridley::Run>, Fault> {
    let stage = station.stage;
    if stage == Stage::Escape && station.shaft_overlay {
        return Err(Fault::Escape);
    }
    let her = Pair {
        x: her.position.x.pixel(),
        y: her.position.y.pixel(),
    };
    let mut runs = Vec::with_capacity(station.doors.len());
    for door in &station.doors {
        let inputs = boss_ceres_ridley::Inputs {
            her,
            stage,
            random: random.number.0,
            shake_frames_left: quake.remaining().0,
            // In his room Ridley runs first, and his run in this piece leaves him unbeaten.
            ridley_beaten,
        };
        let run = door.run(&inputs, rules);
        if let Some(shake) = run.shake {
            world_quake::start_or_stop(quake, order(shake)?);
        }
        runs.push(run);
    }
    Ok(runs)
}

/// A door's shake as the room's quake starts it: what it moves, its strength as the size and its
/// length as the frames, diagonally, as every door's shake moves.
///
/// # Errors
/// [`Fault::Quake`] for a strength the quake has no size for.
fn order(start: ShakeStart) -> Result<ShakeOrder, Fault> {
    let size = u8::try_from(start.shake.strength)
        .ok()
        .and_then(QuakeSize::new)
        .ok_or(Fault::Quake)?;
    let reach = match start.layers {
        ShakeLayers::Whole => QuakeReach::Whole,
        ShakeLayers::Back => QuakeReach::Back,
    };
    Ok(ShakeOrder::Start {
        kind: QuakeKind {
            reach,
            size,
            direction: QuakeDirection::Diagonal,
        },
        frames: QuakeDuration(start.shake.length),
    })
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

    /// In the escape the first door finds the room steady and starts a shake; the second finds
    /// it under way and starts none. The frame's update then spends the shake's first frame.
    #[test]
    fn in_the_escape_the_first_door_starts_a_shake_and_the_next_finds_it_under_way() {
        let mut far = door(opening_ends());
        far.position.x = Wrapping(far.position.x.0.wrapping_add(200));
        let mut escaping = state(vec![door(opening_ends()), far]);
        escaping.station.stage = Stage::Escape;
        let outputs = play(&mut escaping, types::Buttons::default(), Stage::Escape).unwrap();
        let shakes: Vec<_> = outputs.ceres_doors.iter().map(|run| run.shake).collect();
        assert!(shakes[0].is_some());
        assert_eq!(shakes[1], None);
        let length = shakes[0].unwrap().shake.length;
        assert_eq!(escaping.quake.0.remaining().0, length - 1);
        assert!(outputs.quake.unwrap().actors_asked);
    }

    #[test]
    fn a_doors_shake_moves_what_its_kind_moves_by_its_strength_diagonally_for_its_length() {
        use boss_ceres_ridley::{Shake, ShakeLayers, ShakeStart};
        use world_quake::{QuakeDirection, QuakeDuration, QuakeReach, ShakeOrder};
        let start = |strength, layers| ShakeStart {
            shake: Shake {
                strength,
                length: 4,
            },
            layers,
        };
        let Ok(ShakeOrder::Start { kind, frames }) = super::order(start(3, ShakeLayers::Back))
        else {
            panic!("a 3 px shake starts");
        };
        assert_eq!(kind.reach, QuakeReach::Back);
        assert_eq!(kind.size.pixels(), 3);
        assert_eq!(kind.direction, QuakeDirection::Diagonal);
        assert_eq!(frames, QuakeDuration(4));
        for strength in [0, 4, 256] {
            assert_eq!(
                super::order(start(strength, ShakeLayers::Whole)),
                Err(Fault::Quake)
            );
        }
    }

    #[test]
    fn the_escape_refuses_the_shafts_overlay_and_leaves_state_as_it_was() {
        let mut escaping = state(vec![door(opening_ends())]);
        escaping.station.stage = Stage::Escape;
        escaping.station.shaft_overlay = true;
        let before = escaping.clone();
        assert_eq!(
            play(&mut escaping, types::Buttons::default(), Stage::Escape),
            Err(Fault::Escape)
        );
        assert_eq!(escaping, before);
    }
}
