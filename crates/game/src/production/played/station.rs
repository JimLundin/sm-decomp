//! The Ceres station's actors: its doors, the shaft's elevator pad and lights, and Ridley.
//!
//! The room's enemies run before she moves, reading where she was at the end of the last frame;
//! her motion meets the boxes of the Ceres doors that were solid as the frame began.
//!
//! In the Ceres shaft as she arrives, the elevator's pad holds her: while it does, her moves
//! and her animation don't run, and the pad sets her height after her frame. The shaft's
//! overlay, run with the room's enemies, steps the lights, whose beacon sets screen colours.
//!
//! In Ridley's room he runs first of the room's enemies, the baby in his hold, and the door of
//! his room after him. Fetching his room's contents places him: the HUD meets a boss, the
//! stage is calm and his room's colours are the fade's goal. The baby's colours go to the
//! screen as it glows.

use super::super::room_loading::{self, CeresActors};
use super::super::{ridley, shaft};
use super::{Content, Fault, Passing, Rules};
use boss_ceres_ridley::{
    CeresDoor, CeresRidley, Lights, PadAndCover, RidleyPhase, RidleyRun, Stage,
};
use types::{Hitbox, Pair, Parts};
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

/// One run of the room's enemies: Ridley's, each Ceres door's and the shaft's lights.
#[derive(Default)]
pub(super) struct Enemies {
    pub(super) ridley: Option<RidleyRun>,
    pub(super) doors: Vec<boss_ceres_ridley::Run>,
    pub(super) lights: Option<Lights>,
}

impl State {
    /// Ridley's drawing as the frame leaves him, with the doors he draws; none outside his
    /// room. It is produced for drawing, which this loop doesn't compare.
    #[must_use]
    pub fn ridley_drawing(&self) -> Option<ridley::Drawing> {
        self.ridley
            .as_ref()
            .map(|ridley| ridley::drawing(ridley, &self.doors))
    }

    /// The pad's run, which sets `samus`'s height while it holds her.
    pub(super) fn pad(&self, samus: &mut samus_movement::Samus, rules: &Rules) -> PadAndCover {
        shaft::run_pad(self.pad_and_cover, samus, &rules.ceres)
    }

    /// The room's `geometry`, with the Ceres doors that block her.
    pub(super) fn world<'a>(
        &self,
        geometry: &'a super::super::world::MovementWorld,
    ) -> WithDoors<'a> {
        WithDoors::new(geometry, &self.doors)
    }

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
    /// shaft's overlay. The baby's colours and the beacon's go to `palette`.
    pub(super) fn enemies(
        &self,
        passage_under_way: bool,
        random: types::RandomSequence,
        passing: &Passing<'_>,
        content: &Content,
        rules: &Rules,
        palette: &mut Palette,
    ) -> Result<Enemies, Fault> {
        let station = &self.station;
        let ridley = station
            .ridley
            .map(|ridley| run_ridley(ridley, passage_under_way, random, content, rules, palette))
            .transpose()?;
        let doors = run_doors(
            &station.doors,
            station.shaft_overlay,
            &self.samus.movement.0,
            station.stage,
            room_loading::Run {
                progress: passing.progress,
                ceres: station.stage,
            }
            .ridley_beaten(),
            random,
            &rules.ceres,
        )?;
        let lights = self.lights(content, rules, palette)?;
        Ok(Enemies {
            ridley,
            doors,
            lights,
        })
    }

    /// The shaft's lights, on a frame the room's enemies run: with the overlay in the room, the
    /// beacon's step sets the colours it names.
    fn lights(
        &self,
        content: &Content,
        rules: &Rules,
        palette: &mut Palette,
    ) -> Result<Option<Lights>, Fault> {
        if !self.station.shaft_overlay {
            return Ok(None);
        }
        let lights = shaft::lights(
            self.frame.finished_frames,
            self.passage.doors,
            self.station.stage,
            &rules.ceres,
        )
        .map_err(Fault::Lights)?;
        shaft::show(lights, &content.rooms.actors.beacon, palette).map_err(Fault::Lights)?;
        Ok(Some(lights))
    }
}

/// One run of each of the room's Ceres doors, in its order of its enemies, reading where she
/// was at the end of the last frame. In the escape a later piece owns the shaft's overlay.
fn run_doors(
    doors: &[CeresDoor],
    shaft_overlay: bool,
    her: &samus_movement::Samus,
    stage: Stage,
    ridley_beaten: bool,
    random: types::RandomSequence,
    rules: &boss_ceres_ridley::Rules,
) -> Result<Vec<boss_ceres_ridley::Run>, Fault> {
    if stage == Stage::Escape && (shaft_overlay || !doors.is_empty()) {
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
    Ok(doors.iter().map(|door| door.run(&inputs, rules)).collect())
}

/// One run of Ridley, the baby in his hold: its colour set goes to the screen at once. Awake,
/// a later piece runs him.
fn run_ridley(
    ridley: CeresRidley,
    passage_under_way: bool,
    random: types::RandomSequence,
    content: &Content,
    rules: &Rules,
    palette: &mut Palette,
) -> Result<RidleyRun, Fault> {
    if ridley.phase == RidleyPhase::Awake {
        return Err(Fault::RidleyAwake);
    }
    let run = ridley.run(passage_under_way, random.number.0, &rules.ceres);
    if let Some(colours) = run.colours {
        ridley::show_baby(colours, &content.rooms.actors.ceres_ridley, palette)
            .map_err(|ridley::ColourFault| Fault::BabyColours)?;
    }
    Ok(run)
}

/// The room's geometry, with the Ceres doors that block her after its other solid actors.
///
/// The original meets the boxes in slot order, and the first that blocks her decides the move.
/// The geometry's boxes keep no slots, so the doors go last: the admission accepts only rooms
/// whose other solid actors sit in slots before the doors'.
pub(super) struct WithDoors<'a> {
    geometry: &'a super::super::world::MovementWorld,
    solid: Vec<Hitbox>,
}

impl<'a> WithDoors<'a> {
    fn new(geometry: &'a super::super::world::MovementWorld, doors: &[CeresDoor]) -> Self {
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
