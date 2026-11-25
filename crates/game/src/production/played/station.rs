//! The Ceres station's actors in her room, and the order in which the room's enemies run:
//! Ridley (`ridley`), then the Ceres doors (`ceres_doors`), then the shaft's overlay with its
//! lights (`shaft`).
//!
//! The room's enemies run before her moves. The Ceres doors read where she was at the end of the
//! last frame, and Ridley's tail reads her as the frame starts. Fetching the next room's contents lets the last room's actors go and places its own.

use super::passage::Passing;
use super::{Content, Fault, Rules, ceres_doors, ridley};
use crate::production::room_loading::{self, CeresActors};
use boss_ceres_ridley::{CeresDoor, CeresRidley, Her, Lights, PadAndCover, RidleyRun, Stage};
use world_doors::Palette;

/// The station's actors in her room, carried frame to frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// The room's Ceres doors, in its order of its enemies.
    pub doors: Vec<CeresDoor>,
    /// The Ceres shaft's elevator pad and the cover below it.
    pub pad_and_cover: PadAndCover,
    /// The room places the Ceres shaft's overlay, which shows the lights.
    pub shaft_overlay: bool,
    /// Ceres Ridley and the baby, in his room.
    pub ridley: Option<CeresRidley>,
    /// The Ceres station's stage, which his placing sets.
    pub stage: Stage,
}

/// When the room's enemies run: in a frame of play, Ridley's tail reading her as her moves
/// leave her, or while a passage is under way, when nothing of hers is there to reach.
#[derive(Clone, Copy)]
pub(super) enum Running<'a> {
    /// A frame of play, with her as his tail reads her in his room.
    Play(Option<&'a Her>),
    /// A passage is under way.
    Passage,
}

/// One run of the room's enemies: Ridley's, each Ceres door's and the shaft's lights, and the
/// room's quake as Ridley's shake and the doors' leave it.
pub(super) struct Enemies {
    /// Ridley's run; none outside his room.
    pub(super) ridley: Option<RidleyRun>,
    /// Each Ceres door's run, in the room's order of its enemies.
    pub(super) doors: Vec<boss_ceres_ridley::Run>,
    /// What the shaft's lights show; none without its overlay.
    pub(super) lights: Option<Lights>,
    /// The room's quake after the doors' starts.
    pub(super) quake: world_quake::State,
}

impl Enemies {
    /// No run of the room's enemies: the quake as it was.
    pub(super) const fn none(quake: world_quake::State) -> Self {
        Self {
            ridley: None,
            doors: Vec::new(),
            lights: None,
            quake,
        }
    }
}

impl State {
    /// The station after a frame of play's run of its enemies.
    pub(super) fn ran(&mut self, enemies: &Enemies) {
        self.doors = enemies.doors.iter().map(|run| run.door).collect();
        self.ridley = enemies.ridley.map(|run| run.ridley);
    }

    /// The station after a passage's frame: what its enemies' run left, then the actors the
    /// next room's contents place, if fetched, and the pad as the passage left it.
    pub(super) fn passed(
        &mut self,
        enemies: &Enemies,
        placed: Option<CeresActors>,
        pad_and_cover: PadAndCover,
    ) {
        if !enemies.doors.is_empty() {
            self.doors = enemies.doors.iter().map(|run| run.door).collect();
        }
        if let Some(run) = enemies.ridley {
            self.ridley = Some(run.ridley);
        }
        if let Some(placed) = placed {
            self.doors = placed.doors;
            self.shaft_overlay = placed.shaft_overlay;
            self.ridley = placed.ridley.map(|placing| placing.ridley);
            if let Some(placing) = placed.ridley {
                self.stage = placing.stage;
            }
        }
        self.pad_and_cover = pad_and_cover;
    }
}

impl super::State {
    /// One run of the room's enemies, in its order: Ridley, then the Ceres doors, then the
    /// shaft's overlay. The baby's colours and the beacon's go to `palette`, and Ridley's shake
    /// and then the doors' to the room's quake as the frame began.
    pub(super) fn enemies(
        &self,
        running: Running<'_>,
        random: types::RandomSequence,
        passing: &Passing<'_>,
        content: &Content,
        rules: &Rules,
        palette: &mut Palette,
    ) -> Result<Enemies, Fault> {
        let station = &self.station;
        let her_energy = self.health.0.energy.whole.get();
        let ridley = station
            .ridley
            .map(|ridley| ridley::run(ridley, running, random, her_energy, content, rules, palette))
            .transpose()?;
        let ridley_beaten = room_loading::Run {
            progress: passing.progress,
            ceres: station.stage,
        }
        .ridley_beaten();
        let mut quake = self.quake.0;
        // Ridley runs first, so the shake he starts is under way as the doors run: each door
        // reads its frames left and starts a shake only on a steady room.
        if let Some(shake) = ridley.and_then(|run| run.shake) {
            world_quake::start_or_stop(&mut quake, ceres_doors::order(shake)?);
        }
        let doors = ceres_doors::run(
            station,
            &self.samus.movement.0,
            ridley_beaten,
            random,
            &rules.ceres,
            &mut quake,
        )?;
        let lights = self.lights(content, rules, palette)?;
        Ok(Enemies {
            ridley,
            doors,
            lights,
            quake,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::production::played::State;
    use crate::production::played::tests::{play, state};
    use crate::production::tests::fixtures;
    use boss_ceres_ridley::{
        CeresDoor, CeresRidley, DoorKind, FlightPhase, GiveUp, Phase, RidleyPhase, Stage, Wait,
    };
    use core::num::Wrapping;
    use types::{Pair, Parts as _, Side};
    use world_quake::{QuakeDirection, QuakeReach};

    /// Ceres Ridley in flight, `phase`, at `position`, moving by `speed`.
    fn flying(phase: FlightPhase, position: Pair<i16>, speed: Pair<i16>) -> CeresRidley {
        CeresRidley {
            phase: RidleyPhase::InFlight(phase),
            position: Pair {
                x: Wrapping(position.x.cast_unsigned()),
                y: Wrapping(position.y.cast_unsigned()),
            },
            fraction: Pair { x: 0, y: 0 },
            speed: Pair {
                x: Wrapping(speed.x),
                y: Wrapping(speed.y),
            },
            ..CeresRidley::place(false).unwrap().ridley
        }
    }

    /// Her room with Ridley in it, her energy `energy` as the frame starts.
    fn his_room(ridley: CeresRidley, energy: u16, doors: Vec<CeresDoor>) -> State {
        let mut room = state(doors);
        room.station.ridley = Some(ridley);
        room.health.0.energy.whole = samus_health::WholeEnergy::new(energy).unwrap();
        room
    }

    /// Slowing at x 41 with 1,000/256 px a run leftward: his move meets the room's left side
    /// fast.
    fn into_the_left_side() -> CeresRidley {
        flying(
            FlightPhase::Slowing,
            Pair { x: 41, y: 100 },
            Pair { x: -1000, y: 0 },
        )
    }

    /// A Ceres door open, far right of her.
    fn far_door() -> CeresDoor {
        let her = fixtures::samus().position;
        CeresDoor {
            kind: DoorKind::Door,
            facing: Side::Left,
            position: Pair {
                x: Wrapping(her.x.pixel().0.wrapping_add(200)),
                y: her.y.pixel(),
            },
            phase: Phase::Open {
                wait: Wait::new(2).unwrap(),
            },
        }
    }

    /// Hovering far from his point, he reads her energy as the frame starts it: at 29 he gives
    /// up and retreats, at 30 he hovers on.
    #[test]
    fn her_energy_as_the_frame_starts_reaches_his_hover() {
        let hovering = flying(
            FlightPhase::Hovering,
            Pair { x: 100, y: 40 },
            Pair { x: 0, y: 0 },
        );
        for (energy, phase) in [
            (29, RidleyPhase::GivingUp(GiveUp::Retreating)),
            (30, RidleyPhase::InFlight(FlightPhase::Hovering)),
        ] {
            let mut room = his_room(hovering, energy, Vec::new());
            play(&mut room, types::Buttons::default(), Stage::Calm).unwrap();
            assert_eq!(room.station.ridley.map(|ridley| ridley.phase), Some(phase));
        }
    }

    /// Meeting the room's left side fast, his shake reaches the room's quake: the back layers,
    /// sideways, 3 px, for 12 frames, of which the frame's update spends the first.
    #[test]
    fn his_sideways_shake_reaches_the_rooms_quake() {
        let mut room = his_room(into_the_left_side(), 99, Vec::new());
        play(&mut room, types::Buttons::default(), Stage::Calm).unwrap();
        let ridley = room.station.ridley.unwrap();
        assert_eq!(ridley.position.x, Wrapping(40));
        assert_eq!(ridley.speed.x, Wrapping(0));
        let shake = room.quake.0.shake.expect("his shake runs");
        assert_eq!(shake.kind.reach, QuakeReach::Back);
        assert_eq!(shake.kind.direction, QuakeDirection::Sideways);
        assert_eq!(shake.kind.size.pixels(), 3);
        assert_eq!(shake.remaining.get(), 11);
    }

    /// Ridley runs before the doors: a door that would start the escape's shake on a steady
    /// room finds his under way and starts none, so his shake is the one that runs. The game
    /// never has both on one run (a door shakes only in the escape, he only in the fight), so
    /// this is the order of the room's enemies, unproved by any record.
    #[test]
    fn a_door_finds_his_shake_under_way_and_starts_none() {
        let mut steady = state(vec![far_door()]);
        let outputs = play(&mut steady, types::Buttons::default(), Stage::Escape).unwrap();
        assert!(outputs.ceres_doors[0].shake.is_some());

        let mut room = his_room(into_the_left_side(), 99, vec![far_door()]);
        let outputs = play(&mut room, types::Buttons::default(), Stage::Escape).unwrap();
        assert_eq!(outputs.ceres_doors[0].shake, None);
        let shake = room.quake.0.shake.expect("his shake runs");
        assert_eq!(shake.kind.direction, QuakeDirection::Sideways);
        assert_eq!(shake.remaining.get(), 11);
    }
}
