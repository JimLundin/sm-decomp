//! The Ceres station's actors and their frame order.
//!
//! Ridley runs first, followed by the Ceres doors, the shaft's overlay and the steam vents.
//!
//! The room's enemies run before her moves. The Ceres doors read where she was at the end of the
//! last frame. The tail and the vents read her from before she moves. Loading the next room
//! replaces the station's actors.

use super::passage::Passing;
use super::{Content, Fault, Rules, ceres_doors, ridley, warning};
use crate::production::room_loading::{self, CeresActors};
use boss_ceres_ridley::{CeresDoor, CeresRidley, Her, Lights, PadAndCover, RidleyRun, Stage, Vent};
use types::ceres_warning::WarningState;
use types::{Hit, Pair, Parts as _};
use world_doors::Palette;

/// The station's actors in her room, carried frame to frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// The room's Ceres doors, in its order of its enemies.
    pub doors: Vec<CeresDoor>,
    /// Population indices of each active door and vent.
    pub slots: room_loading::ActorSlots,
    /// The room's steam vents, in enemy slot order.
    pub vents: Vec<Vent>,
    /// The Ceres shaft's elevator pad and the cover below it.
    pub pad_and_cover: PadAndCover,
    /// The room places the Ceres shaft's overlay, which shows the lights.
    pub shaft_overlay: bool,
    /// Ceres Ridley and the baby, in his room.
    pub ridley: Option<CeresRidley>,
    /// The warning phase and glow while Ridley's room carries them.
    pub warning: Option<WarningState>,
    /// The Ceres station's stage, which his placing sets.
    pub stage: Stage,
}

/// When the room's enemies run: in play they can reach her as the frame starts. During a
/// passage she is unavailable for contact.
#[derive(Clone, Copy)]
pub(super) enum Running<'a> {
    /// A frame of play, with her frame-start position, body and protection.
    Play(Option<&'a Her>),
    /// A passage is under way.
    Passage,
}

/// Which actor's contact won the frame.
#[derive(Clone, Copy)]
pub(super) enum Contact {
    /// Ridley's tail.
    Tail(Hit),
    /// A steam puff.
    Steam(Hit),
}

impl Contact {
    /// The side and push duration read by movement.
    pub(super) const fn push(self) -> Hit {
        match self {
            Self::Tail(hit) | Self::Steam(hit) => hit,
        }
    }
}

/// One run of the room's enemies: Ridley's, each Ceres door's and the shaft's lights, and the
/// room's quake as Ridley's shake and the doors' leave it.
pub(super) struct Enemies {
    /// Ridley's run; none outside his room.
    pub(super) ridley: Option<RidleyRun>,
    pub(super) warning: Option<warning::Output>,
    /// The vents after their run, if the room ran its enemies.
    pub(super) vents: Option<Vec<Vent>>,
    /// The first puff hit in slot order.
    pub(super) steam_hit: Option<Hit>,
    /// Each Ceres door's run, in the room's order of its enemies.
    pub(super) doors: Vec<boss_ceres_ridley::Run>,
    /// What the shaft's lights show; none without its overlay.
    pub(super) lights: Option<Lights>,
    /// The room's quake after the doors' starts.
    pub(super) quake: world_quake::State,
}

impl Enemies {
    /// The contact shared by her reaction and health. Ridley precedes vents in enemy order.
    pub(super) fn contact(&self) -> Option<Contact> {
        self.ridley
            .and_then(|run| run.hit)
            .map(Contact::Tail)
            .or_else(|| self.steam_hit.map(Contact::Steam))
    }

    /// No run of the room's enemies: the quake as it was.
    pub(super) const fn none(quake: world_quake::State) -> Self {
        Self {
            ridley: None,
            warning: None,
            vents: None,
            steam_hit: None,
            doors: Vec::new(),
            lights: None,
            quake,
        }
    }
}

impl State {
    /// Newly placed walls first run on the following frame.
    fn place_backdrop(&mut self, enemies: &Enemies) {
        if let Some(backdrop) = enemies.ridley.and_then(|run| run.backdrop) {
            self.doors
                .extend([backdrop.left_wall.door, backdrop.right_wall.door]);
            self.slots.doors.extend([2, 3]);
        }
    }

    /// Retire the old hall population in slot order as the clearing work progresses.
    pub(super) fn retire_before(&mut self, first_live: usize) {
        fn retire<T>(actors: &mut Vec<T>, slots: &mut Vec<usize>, first_live: usize) {
            let count = slots.partition_point(|slot| *slot < first_live);
            slots.drain(..count);
            actors.drain(..count);
        }
        retire(&mut self.vents, &mut self.slots.vents, first_live);
        retire(&mut self.doors, &mut self.slots.doors, first_live);
        // The Ceres room places Ridley first. The walls, door and other
        // actors have their own slot indices.
        if first_live > 0 {
            self.ridley = None;
        }
    }

    /// The station after a frame of play's run of its enemies.
    pub(super) fn ran(&mut self, enemies: &Enemies) {
        self.doors = enemies.doors.iter().map(|run| run.door).collect();
        if let Some(output) = &enemies.warning {
            self.warning = Some(output.state);
            if let Some(activation) = output.activation {
                self.stage = activation.stage;
            }
        } else {
            self.ridley = enemies.ridley.map(|run| run.ridley);
        }
        if let Some(stage) = enemies
            .ridley
            .and_then(|run| run.getaway)
            .and_then(|getaway| getaway.stage)
        {
            self.stage = stage;
        }
        self.place_backdrop(enemies);
    }

    /// The station after a passage's frame: what its enemies' run left, then the actors the
    /// next room's contents place, if fetched, and the pad as the passage left it.
    pub(super) fn passed(
        &mut self,
        enemies: &mut Enemies,
        placed: Option<CeresActors>,
        pad_and_cover: PadAndCover,
    ) {
        if !enemies.doors.is_empty() {
            self.doors = enemies.doors.iter().map(|run| run.door).collect();
        }
        if let Some(run) = enemies.ridley {
            self.ridley = Some(run.ridley);
        }
        self.place_backdrop(enemies);
        if let Some(vents) = enemies.vents.take() {
            self.vents = vents;
        }
        if let Some(placed) = placed {
            self.slots = placed.slots;
            self.vents = placed.vents;
            self.doors = placed.doors;
            self.shaft_overlay = placed.shaft_overlay;
            self.ridley = placed.ridley.map(|placing| placing.ridley);
            self.warning = None;
            if let Some(placing) = placed.ridley {
                self.stage = placing.stage;
            }
        }
        self.pad_and_cover = pad_and_cover;
    }
}

impl super::State {
    /// The shared contact input before movement or the health countdown changes it.
    pub(super) fn enemy_contact(&self, content: &Content) -> Result<Option<Her>, Fault> {
        if self.station.ridley.is_none() && self.station.vents.is_empty() {
            return Ok(None);
        }
        let samus = &self.samus.movement.0;
        let body = content
            .movement
            .radius(samus.pose)
            .ok_or(Fault::Body(samus.pose))?;
        Ok(Some(Her {
            position: Pair {
                x: samus.position.x.pixel(),
                y: samus.position.y.pixel(),
            },
            body,
            protection: u16::from(self.health.0.protection.get()),
        }))
    }

    /// One run of the room's enemies, in its order: Ridley, then the Ceres doors, then the
    /// shaft's overlay and the vents. The baby's colours and the beacon's go to `palette`, and Ridley's shake
    /// and then the doors' to the room's quake as the frame began.
    pub(super) fn enemies(
        &self,
        running: Running<'_>,
        random: types::RandomSequence,
        passing: &Passing,
        content: &Content,
        rules: &Rules,
        palette: &mut Palette,
    ) -> Result<Enemies, Fault> {
        let station = &self.station;
        let her_energy = self.health.0.energy.whole.get();
        let warning = station
            .warning
            .or_else(|| {
                matches!(
                    station.ridley?.phase,
                    boss_ceres_ridley::RidleyPhase::Getaway(
                        types::ceres_getaway::GetawayState::WarningReady
                    )
                )
                .then_some(WarningState::ready())
            })
            .map(|state| {
                warning::run(
                    state,
                    content.warning.as_ref().ok_or(Fault::RidleyWarning)?,
                    self.frame,
                    matches!(running, Running::Play(_)) && passing.work_begun,
                    palette,
                )
            })
            .transpose()?;
        let ridley = station
            .ridley
            .filter(|_| warning.is_none())
            .map(|ridley| {
                ridley::run(
                    ridley,
                    running,
                    (random, types::GameFrames(self.frame.finished_frames)),
                    her_energy,
                    content,
                    rules,
                    palette,
                )
            })
            .transpose()?;
        let activation = warning.as_ref().and_then(|output| output.activation);
        let stage = activation
            .map(|activation| activation.stage)
            .or_else(|| {
                ridley
                    .and_then(|run| run.getaway)
                    .and_then(|getaway| getaway.stage)
            })
            .unwrap_or(station.stage);
        let ridley_beaten = room_loading::Run {
            progress: &self.progress,
            ceres: station.stage,
        }
        .ridley_beaten()
            || activation.is_some();
        let mut quake = self.quake.0;
        // Ridley runs first, so the shake he starts is under way as the doors run: each door
        // reads its frames left and starts a shake only on a steady room.
        if let Some(shake) = ridley.and_then(|run| run.shake) {
            world_quake::start_or_stop(&mut quake, ceres_doors::order(shake)?);
        }
        let doors = ceres_doors::run(
            station,
            stage,
            &self.samus.movement.0,
            ridley_beaten,
            random,
            &rules.ceres,
            &mut quake,
        )?;
        let lights = self.lights(content, rules, palette)?;
        let mut vents = station.vents.clone();
        let view = self.camera.0.position();
        let camera = Pair {
            x: view.x.pixel(),
            y: view.y.pixel(),
        };
        let her = match running {
            Running::Play(her) => her,
            Running::Passage => None,
        };
        let steam_hit = boss_ceres_ridley::run_vents(&mut vents, camera, her, &rules.ceres);
        Ok(Enemies {
            ridley,
            warning,
            vents: Some(vents),
            steam_hit,
            doors,
            lights,
            quake,
        })
    }
}

/// What the flyby and its room owners publish when Samus's movement is outside the scope.
pub struct FlybyOutput {
    /// The flyby's complete effects, including its unserved request to movement.
    pub picture: boss_ceres_ridley::GetawayEffects,
    /// The room quake's offsets after the request.
    pub quake: world_quake::Effects,
}

impl super::State {
    /// Advance the same room actors and late flyby effects without stepping Samus.
    ///
    /// The caller provides one complete turn in Ridley's room. The carried Samus position
    /// remains available to the doors. This entry returns the side-free request to movement.
    ///
    /// # Errors
    /// Returns a typed fault for a missing flyby, warning handoff or invalid room turn.
    pub fn step_flyby(
        &mut self,
        passing: &Passing,
        content: &Content,
        rules: &Rules,
    ) -> Result<FlybyOutput, Fault> {
        if !passing.work_begun
            || !passing.work_done
            || self.passage.doors != world_doors::Doors::Play
        {
            return Err(Fault::RidleyGetawayPush);
        }
        let random = self.frame.drawn(passing);
        let mut palette = self.palette;
        let enemies = self.enemies(
            Running::Play(None),
            random,
            passing,
            content,
            rules,
            &mut palette,
        )?;
        let picture = enemies
            .ridley
            .and_then(|run| run.getaway)
            .ok_or(Fault::RidleyGetawayPush)?;
        let mut quake = enemies.quake;
        ridley::apply_getaway(picture, &mut quake, &mut palette);
        let offsets = super::quake::update(&mut quake)?;
        self.station.ran(&enemies);
        self.palette = palette;
        self.quake = canonical::model::Model(quake);
        self.frame = self.frame.after(world_doors::Finishing::Finished, random);
        Ok(FlybyOutput {
            picture,
            quake: offsets,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::production::played::State;
    use crate::production::played::tests::{play, state};
    use crate::production::tests::fixtures;
    use boss_ceres_ridley::{
        CeresDoor, CeresRidley, DoorKind, FlightPhase, Phase, RidleyPhase, Stage, Wait,
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

    #[test]
    fn population_clear_retires_ridley_at_slot_zero_before_later_doors() {
        let ridley = CeresRidley::place(false).unwrap().ridley;
        let door = CeresDoor {
            kind: DoorKind::Door,
            facing: Side::Left,
            position: Pair {
                x: Wrapping(32),
                y: Wrapping(127),
            },
            phase: Phase::Placed,
        };
        let mut room = his_room(ridley, 24, vec![door]);
        room.station.slots.doors = vec![1];
        room.station.stage = Stage::Escape;
        room.station.retire_before(0);
        assert_eq!(room.station.ridley, Some(ridley));
        room.station.retire_before(1);
        assert_eq!(room.station.ridley, None);
        assert_eq!(room.station.doors, vec![door]);
        assert_eq!(room.station.stage, Stage::Escape);
        room.station.retire_before(2);
        assert_eq!(room.station.doors, Vec::new());
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
    /// up and starts retreating, at 30 he hovers on.
    #[test]
    fn her_energy_as_the_frame_starts_reaches_his_hover() {
        let hovering = flying(
            FlightPhase::Hovering,
            Pair { x: 100, y: 40 },
            Pair { x: 0, y: 0 },
        );
        let mut room = his_room(hovering, 29, Vec::new());
        play(&mut room, types::Buttons::default(), Stage::Calm).unwrap();
        let retreating = room.station.ridley.unwrap();
        assert_eq!(retreating.phase, RidleyPhase::Retreating);
        assert_ne!(retreating.speed, hovering.speed);

        let mut room = his_room(hovering, 30, Vec::new());
        play(&mut room, types::Buttons::default(), Stage::Calm).unwrap();
        assert_eq!(
            room.station.ridley.map(|ridley| ridley.phase),
            Some(hovering.phase)
        );
    }

    #[test]
    fn a_restored_stop_places_the_walls_and_the_tip_still_hits() {
        use boss_ceres_ridley::{CoastRuns, FlightTail};
        use types::{Hit, PushFrames};

        let her = fixtures::samus().position;
        let position = Pair {
            x: her.x.pixel() - Wrapping(31),
            y: her.y.pixel() - Wrapping(16),
        };
        let mut tail = FlightTail::comes_loose(position);
        for segment in &mut tail.segments {
            segment.offset = Pair { x: 0, y: 0 };
        }
        let ridley = CeresRidley {
            phase: RidleyPhase::CoastingAbove(CoastRuns::new(0).unwrap()),
            position,
            flight_tail: Some(tail),
            ..CeresRidley::place(false).unwrap().ridley
        };
        let mut room = his_room(ridley, 24, Vec::new());
        let outputs = play(&mut room, types::Buttons::default(), Stage::Calm).unwrap();
        let run = outputs.ridley.unwrap();
        assert_eq!(run.ridley.phase, RidleyPhase::GETAWAY_START);
        assert_eq!(
            run.hit,
            Some(Hit {
                side: Side::Left,
                push: PushFrames::new(5).unwrap(),
            })
        );
        assert_eq!(room.health.0.energy.whole.get(), 9);
        assert_eq!(room.health.0.protection.get(), 95);
        assert_eq!(room.samus.movement.0.knockback.side, Side::Left);
        let backdrop = run.backdrop.unwrap();
        assert_eq!(
            room.station.doors,
            [backdrop.left_wall.door, backdrop.right_wall.door]
        );
        assert!(
            room.station
                .doors
                .iter()
                .all(|wall| wall.phase == Phase::Placed)
        );
        assert_eq!(room.station.stage, Stage::Calm);
        let mut getaway = room.clone();
        let first = play(&mut getaway, types::Buttons::default(), Stage::Calm).unwrap();
        assert_eq!(getaway.station.stage, Stage::Getaway);
        assert!(first.ridley.unwrap().getaway.unwrap().sound.is_some());

        room.station.retire_before(1);
        assert!(room.station.ridley.is_none());
        assert_eq!(
            room.station.doors,
            [backdrop.left_wall.door, backdrop.right_wall.door]
        );
        room.station.retire_before(3);
        assert_eq!(room.station.doors, [backdrop.right_wall.door]);
        room.station.retire_before(4);
        assert_eq!(room.station.doors, []);
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
