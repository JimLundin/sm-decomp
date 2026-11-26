//! Dispatch the flight from the station's blackout, carrying the running game's state.

mod display;
mod progress;
pub use progress::Remembered;

use super::{explosion, landing_site, zebes};
use crate::production::ProductionOutputs;
use crate::state::video::{VRAM_WORDS, Video};
use canonical::Canonical;
use types::zebes::SubtitleChoice;

/// The shaft placement and map owners retained during the cinematics.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Room {
    /// The shaft's current room, doorway, camera and Samus placement.
    pub placement: landing_site::Placement,
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
    /// Station artwork and the cinematic before the flight.
    pub station: data::assets::explosion::Station,
    /// The flight and the artwork installed by its setup.
    pub flight: data::assets::zebes::Zebes,
    /// Standard gameplay graphics at the start of the room load.
    pub landing: data::assets::landing_site::Graphics,
}

/// Selected rule settings for both cinematic owners.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Station reveal, breakup and departure rules.
    pub station: screens_explosion::Rules,
    /// Flight, caption and departure rules.
    pub flight: screens_zebes::Rules,
}

impl Rules {
    /// The original game's settings.
    pub const REFERENCE: Self = Self {
        station: screens_explosion::Rules::REFERENCE,
        flight: screens_zebes::Rules::REFERENCE,
    };
}

/// A scheduler opportunity or a direct DMA receipt from the platform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    /// Borrowed work completion and post-service music occupancy.
    Scene(explosion::Cut),
    /// Progress of an unfinished receiving loader.
    Loader(landing_site::Work),
}

/// The station or flight whose complete state is carried through the blackout.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum Scene {
    /// The station and its scheduler, including a pending dispatch of the flight.
    Station(Box<explosion::State>),
    /// The flight and its scheduler, including the pending Landing Site destination.
    Flight(Box<zebes::State>),
}

/// The complete continuation from an elevator blackout to the first room load.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Run {
    /// The running or last completed cinematic.
    pub scene: Scene,
    /// The shared owners and unfinished loader.
    pub carry: Carry,
}

/// A missing producer, invalid origin or unavailable drawing transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// One of the carried memories does not have its admitted size.
    Intake,
    /// The station declined its state or inputs.
    Station(explosion::Fault),
    /// The flight declined its state or inputs.
    Flight(zebes::Fault),
    /// A drawing transfer extends outside the receiving page.
    Drawing,
    /// Work or a transfer source is unavailable to the room loader.
    Landing(landing_site::Fault),
    /// Work belongs to another scheduler phase.
    Timing,
}

impl Run {
    pub(in crate::production) fn validate(&self, content: &Content) -> Result<(), Fault> {
        if self.carry.video.memory.vram.len() != VRAM_WORDS
            || self.carry.landing.shadow.bg1.len() != 1024
            || self.carry.landing.shadow.bg2.len() != 1024
        {
            return Err(Fault::Intake);
        }
        match &self.scene {
            Scene::Station(state) => state.validate(&content.station).map_err(Fault::Station),
            Scene::Flight(state) => state.validate(&content.flight).map_err(Fault::Flight),
        }
    }

    /// The scheduler has dispatched the loader.
    #[must_use]
    pub fn loading(&self) -> bool {
        matches!(&self.scene, Scene::Flight(flight) if matches!(flight.flow,zebes::Flow::Loading(_)))
    }

    pub(in crate::production) fn advance(
        &mut self,
        content: &Content,
        rules: Rules,
        input: Input,
    ) -> Result<ProductionOutputs, Fault> {
        if let Input::Loader(work) = input {
            if !self.loading()
                || matches!(work, landing_site::Work::Open { .. }) && !self.carry.work_ready
            {
                return Err(Fault::Timing);
            }
            let Scene::Flight(flight) = &self.scene else {
                return Err(Fault::Timing);
            };
            let mut sequence = types::RandomSequence {
                number: types::RandomNumber(self.carry.random),
            };
            self.carry
                .landing
                .advance(
                    flight.flow,
                    &mut sequence,
                    &mut self.carry.video,
                    &content.landing,
                    work,
                )
                .map_err(Fault::Landing)?;
            self.carry.random = sequence.number.0;
            self.carry.work_ready = false;
            return Ok(ProductionOutputs::LandingSite);
        }
        let Input::Scene(cut) = input else {
            return Err(Fault::Timing);
        };
        let finished = matches!(cut, explosion::Cut::Finished { .. });
        if let Scene::Station(station) = &self.scene
            && station.flow == explosion::Flow::ZebesFlight
        {
            display::opening(&mut self.carry.video, types::zebes::FlightView::MosaicPage);
            let flight = from_station(station);
            flight.validate(&content.flight).map_err(Fault::Flight)?;
            self.scene = Scene::Flight(Box::new(flight));
        }

        let output = match &mut self.scene {
            Scene::Station(station) => {
                let draw = match station.flow {
                    explosion::Flow::Blank(9) => self.carry.work_ready,
                    explosion::Flow::Blank(_) | explosion::Flow::ZebesFlight => false,
                    explosion::Flow::Installing | explosion::Flow::Station => finished,
                };
                if draw {
                    self.carry.draw();
                }
                if station.flow == explosion::Flow::Installing {
                    self.carry.room.placement.door = data::id::DoorwayId(0);
                    display::opening(
                        &mut self.carry.video,
                        types::zebes::FlightView::ShipAndSprites,
                    );
                }
                let out = station
                    .advance(&content.station, rules.station, cut)
                    .map_err(Fault::Station)?;
                for page in &out.pages {
                    self.carry.station_page(page)?;
                }
                if finished {
                    display::station(&mut self.carry.video, station, &out);
                }
                ProductionOutputs::Explosion(Box::new(out))
            }
            Scene::Flight(flight) => {
                if self.carry.work_ready {
                    self.carry.draw();
                }
                let advance = if finished {
                    screens_zebes::FrameAdvance::Finished
                } else {
                    screens_zebes::FrameAdvance::BorrowedLag
                };
                let out = flight
                    .advance(&content.flight, rules.flight, advance)
                    .map_err(Fault::Flight)?;
                if let Some(pages) = &out.pages {
                    pages.install(&mut self.carry.video.memory.vram);
                }
                if out.scene.requests.handoff.is_some() {
                    self.carry
                        .progress
                        .restore_energy(flight.health.0.energy.whole.get());
                }
                if finished {
                    display::flight(&mut self.carry.video, flight, &out);
                }
                ProductionOutputs::Zebes(Box::new(out))
            }
        };
        self.carry.work_ready = finished;
        if finished {
            self.carry.publication = self.carry.video.committed;
        }
        Ok(output)
    }
}

fn from_station(station: &explosion::State) -> zebes::State {
    zebes::State::start(
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
    )
}

impl Carry {
    fn draw(&mut self) {
        let mut sequence = types::RandomSequence {
            number: types::RandomNumber(self.random),
        };
        self.random = game_random::advance_random(&mut sequence).0;
    }

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
