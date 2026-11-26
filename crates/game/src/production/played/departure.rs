//! Dispatch the flight from the station's blackout, carrying the running game's state.

mod display;
mod progress;
pub use progress::Remembered;

use super::{explosion, landing_site, zebes};
use crate::production::{
    AdvanceInput, Production, ProductionContent, ProductionFault, ProductionOutputs,
    ProductionRules, ProductionSeed, SavedProduction,
};
use crate::state::owned::OwnedState;
use crate::state::video::{VRAM_WORDS, Video};
use canonical::Canonical;
use std::sync::Arc;
use types::zebes::SubtitleChoice;

/// The shaft placement and map owners retained during the cinematics.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Room {
    /// Current room header.
    pub room: u16,
    /// Most recently selected door.
    pub door: u16,
    /// Camera position in the room.
    pub camera: [u16; 2],
    /// Samus's position in the room.
    pub samus: [u16; 2],
    /// Station selected at the elevator origin, before the loader opens its destination.
    pub station: u16,
}

/// The continuing game owners outside the selected cinematic.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Carry {
    /// Physical display memory and committed registers.
    pub video: Video,
    /// The shaft and saved map.
    pub room: Room,
    /// Retained maps and shadow tilemaps, then the unfinished receiving loader.
    pub landing: landing_site::State,
    /// Persistent inventory, map status and events, including whole energy.
    pub progress: Remembered,
    /// Most recent finished display publication before an unfinished loader cut.
    pub publication: crate::state::video::Registers,
    /// Current game-random sequence value, retained across both scenes.
    pub random: u16,
    /// A new frame of work can open on the next cut.
    pub work_ready: bool,
}

/// Static assets reached after the station publishes its blackout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// The flight and the artwork installed by its setup.
    pub flight: data::assets::zebes::Zebes,
    /// Standard gameplay graphics at the start of the room load.
    pub landing: data::assets::landing_site::Graphics,
}

/// Publications of one cinematic cut or an unfinished room load.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Output {
    /// Existing production owner and its drawing outputs.
    Scene(ProductionOutputs),
    /// The receiver has opened or advanced work; the game frame is unfinished.
    Landing,
}

/// A single scheduled cinematic and the owners shared across scene transitions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    scene: Production,
    content: Arc<Content>,
    carry: Carry,
}

/// Canonical scene and game continuation with its admitted immutable content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Saved {
    scene: SavedProduction,
    content: Arc<Content>,
    carry: Vec<u8>,
}

/// A missing producer, invalid origin, checkpoint or drawing transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Admission requires a running station and the full physical VRAM.
    Intake,
    /// An existing cinematic declined the frame.
    Scene(ProductionFault),
    /// The running station's handoff is invalid for the receiving scene.
    Handoff,
    /// A drawing write exceeds the size of the receiving page.
    Drawing,
    /// Work or the next transfer source is unavailable to the loader.
    Landing(landing_site::Fault),
    /// The borrowed scheduler has no receipt for the continuation.
    Timing,
    /// A canonical checkpoint could not be decoded or admitted.
    Checkpoint,
}

impl Run {
    /// Begin with an already admitted station and the live game owners.
    ///
    /// # Errors
    /// Refuses an origin outside the station or incomplete VRAM.
    pub fn start(
        scene: Production,
        content: Content,
        carry: Carry,
    ) -> Result<Self, Fault> {
        if !matches!(scene.state(), OwnedState::Explosion(_))
            || carry.video.memory.vram.len() != VRAM_WORDS
        {
            return Err(Fault::Intake);
        }
        Ok(Self {
            scene,
            content: Arc::new(content),
            carry,
        })
    }

    /// The currently scheduled production entry.
    #[must_use]
    pub const fn scene(&self) -> &Production {
        &self.scene
    }

    /// The carried physical state, room and next work opportunity.
    #[must_use]
    pub const fn carry(&self) -> &Carry {
        &self.carry
    }

    /// Capture both the selected scene and the state shared with its successor.
    ///
    /// # Errors
    /// Refuses a scene without canonical checkpoint support.
    pub fn save(&self) -> Result<Saved, Fault> {
        Ok(Saved {
            scene: self.scene.save().map_err(|_| Fault::Checkpoint)?,
            content: Arc::clone(&self.content),
            carry: canonical::to_bytes(&self.carry),
        })
    }

    /// Restore the scene, physical memory and pending work from their canonical encodings.
    ///
    /// # Errors
    /// Refuses an invalid checkpoint or changed content.
    pub fn restore(saved: &Saved) -> Result<Self, Fault> {
        let context = &saved.scene.context;
        let scene =
            Production::restore(&saved.scene, context.content.clone(), context.rules.clone())
                .map_err(|_| Fault::Checkpoint)?;
        Ok(Self {
            scene,
            content: Arc::clone(&saved.content),
            carry: canonical::from_bytes(&saved.carry).map_err(|_| Fault::Checkpoint)?,
        })
    }

    /// Advance one externally timed cut. Commit the scene and its receivers together.
    ///
    /// # Errors
    /// The next loader or an unavailable drawing write leaves the whole run untouched.
    pub fn advance(&mut self, cut: explosion::Cut, loader_work: Option<landing_site::Work>) -> Result<Output, Fault> {
        let mut next = self.clone();
        let output = if let OwnedState::Zebes(flight) = next.scene.state()
            && matches!(flight.flow, zebes::Flow::Loading(_))
        {
            let work = loader_work.ok_or(Fault::Timing)?;
            let mut sequence = types::RandomSequence { number: types::RandomNumber(next.carry.random) };
            next.carry.landing.advance(flight.flow, &mut sequence, &mut next.carry.video, &next.content.landing, work).map_err(Fault::Landing)?;
            next.carry.random = sequence.number.0;
            next.carry.work_ready = false;
            Output::Landing
        } else {
            Output::Scene(next.step(cut)?)
        };
        *self = next;
        Ok(output)
    }

    fn step(&mut self, cut: explosion::Cut) -> Result<ProductionOutputs, Fault> {
        let finished = matches!(cut, explosion::Cut::Finished { .. });
        if let OwnedState::Explosion(station) = self.scene.state()
            && station.flow == explosion::Flow::ZebesFlight
        {
            display::opening(&mut self.carry.video, types::zebes::FlightView::MosaicPage);
            let state = zebes::State::start(
                station.finished,
                station.health.0,
                station.maximum,
                station.palette,
                station.oam,
                if station.japanese_subtitles {
                    SubtitleChoice::WithJapanese
                } else {
                    SubtitleChoice::EnglishOnly
                },
                match station.destination {
                    explosion::Destination::LandingSite => zebes::LoadDestination::LandingSite,
                    explosion::Destination::CeresStart => zebes::LoadDestination::CeresStart,
                },
            );
            self.scene = Production::admit(
                ProductionSeed::Zebes(Box::new(state)),
                ProductionContent::Zebes(Box::new(self.content.flight.clone())),
                ProductionRules::Zebes(screens_zebes::Rules::REFERENCE),
            )
            .map_err(|_| Fault::Handoff)?;
        }
        let (input, draw) = match self.scene.state() {
            OwnedState::Explosion(station) => {
                let draw = match station.flow {
                    explosion::Flow::Blank(9) => self.carry.work_ready,
                    explosion::Flow::Blank(_) | explosion::Flow::ZebesFlight => false,
                    explosion::Flow::Installing | explosion::Flow::Station => finished,
                };
                if station.flow == explosion::Flow::Installing {
                    self.carry.room.door = 0;
                    display::opening(&mut self.carry.video, types::zebes::FlightView::ShipAndSprites);
                }
                (AdvanceInput::Explosion(cut), draw)
            }
            OwnedState::Zebes(_) => (
                AdvanceInput::Zebes(if finished {
                    screens_zebes::FrameAdvance::Finished
                } else {
                    screens_zebes::FrameAdvance::BorrowedLag
                }),
                self.carry.work_ready,
            ),
            _ => return Err(Fault::Intake),
        };
        if draw {
            let mut sequence = types::RandomSequence {
                number: types::RandomNumber(self.carry.random),
            };
            self.carry.random = game_random::advance_random(&mut sequence).0;
        }
        self.carry.work_ready = finished;
        let output = self.scene.advance(input).map_err(Fault::Scene)?;
        match (&output, self.scene.state()) {
            (ProductionOutputs::Explosion(out), OwnedState::Explosion(state)) => {
                for page in &out.pages {
                    self.carry.station_page(page)?;
                }
                if finished { display::station(&mut self.carry.video,state,out); }
            }
            (ProductionOutputs::Zebes(out), OwnedState::Zebes(state)) => {
                if let Some(pages) = &out.pages {
                    pages.install(&mut self.carry.video.memory.vram);
                }
                if out.scene.requests.handoff.is_some() { self.carry.progress.restore_energy(state.health.0.energy.whole.get()); }
                if finished { display::flight(&mut self.carry.video,state,out); }
            }
            _ => return Err(Fault::Intake),
        }
        if finished { self.carry.publication = self.carry.video.committed; }
        Ok(output)
    }
}

impl Carry {
    fn station_page(&mut self, page: &explosion::Pages) -> Result<(), Fault> {
        match page {
            explosion::Pages::Station {
                map,
                graphics,
                sprites,
            } => {
                let words = self
                    .video
                    .memory
                    .vram
                    .get_mut(..map.len())
                    .ok_or(Fault::Drawing)?;
                if words.len() != graphics.len() || !sprites.len().is_multiple_of(2) {
                    return Err(Fault::Drawing);
                }
                for ((word, &map), &tile) in words.iter_mut().zip(map).zip(graphics) {
                    *word = u16::from_le_bytes([map, tile]);
                }
                let words = self
                    .video
                    .memory
                    .vram
                    .get_mut(0x6000..)
                    .ok_or(Fault::Drawing)?;
                if words.len() != sprites.len() / 2 {
                    return Err(Fault::Drawing);
                }
                for (word, &pair) in words.iter_mut().zip(sprites.as_chunks::<2>().0) {
                    *word = u16::from_le_bytes(pair);
                }
                self.landing.shadow.bg2.fill(0x002f);
            }
            explosion::Pages::Ship(region, map) => self.low_map(
                match region {
                    screens_explosion::MapRegion::Upper => 0,
                    screens_explosion::MapRegion::Lower => 0x300,
                },
                map,
            )?,
        }
        Ok(())
    }

    fn low_map(&mut self, start: usize, map: &[u8]) -> Result<(), Fault> {
        let words = self
            .video
            .memory
            .vram
            .get_mut(start..start.saturating_add(map.len()))
            .ok_or(Fault::Drawing)?;
        for (word, &cell) in words.iter_mut().zip(map) {
            *word = (*word & 0xff00) | u16::from(cell);
        }
        Ok(())
    }

}
