//! The Ceres station's actors in her room, and the order in which the room's enemies run:
//! Ridley (`ridley`), then the Ceres doors (`ceres_doors`), then the shaft's overlay with its
//! lights (`shaft`).
//!
//! The room's enemies run between her moves and her reaction. The Ceres doors read where she was
//! at the end of the last frame, and Ridley's tail reads her as her moves leave her. Fetching the next room's contents lets the last room's actors go and places its own.

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

/// One run of the room's enemies: Ridley's, each Ceres door's and the shaft's lights.
#[derive(Default)]
pub(super) struct Enemies {
    /// Ridley's run; none outside his room.
    pub(super) ridley: Option<RidleyRun>,
    /// Each Ceres door's run, in the room's order of its enemies.
    pub(super) doors: Vec<boss_ceres_ridley::Run>,
    /// What the shaft's lights show; none without its overlay.
    pub(super) lights: Option<Lights>,
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
    /// shaft's overlay. The baby's colours and the beacon's go to `palette`.
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
        let ridley = station
            .ridley
            .map(|ridley| ridley::run(ridley, running, random, content, rules, palette))
            .transpose()?;
        let ridley_beaten = room_loading::Run {
            progress: passing.progress,
            ceres: station.stage,
        }
        .ridley_beaten();
        let doors = ceres_doors::run(
            station,
            &self.samus.movement.0,
            ridley_beaten,
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
}
