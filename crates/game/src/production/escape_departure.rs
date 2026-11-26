//! The shaft's last room pass, its cartridge write and the two departing cinematics.

use super::played::{departure, explosion, landing_site};
use super::{
    AdmissionFault, ProductionFault, ProductionOutputs, Unsupported, blackout, save_files,
};
use crate::state::video::{DMA_CHANNELS, Video};
use canonical::{Canonical, model::Model};
use data::assets::countdown::IndirectHdmaStart;
use types::ceres_blackout::FlightRoute;

/// Assets of the room, station, flight and receiving loader.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// Timer drawing and cartridge format.
    pub shaft: blackout::Content,
    /// ROM table entries the active shaft channels latch as vblank ends.
    pub raster_opening: [Option<IndirectHdmaStart>; DMA_CHANNELS],
    /// Both cinematic owners and the loader's first graphics.
    pub departure: departure::Content,
}

/// Remaining room owners at the elevator.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Shaft {
    /// The scene, live room picture, profile and cartridge.
    pub blackout: blackout::State,
    /// Room placement retained until the loader opens its destination.
    pub room: departure::Room,
    /// Active and saved maps and shadow tilemaps retained for the landing.
    pub landing: landing_site::State,
    /// Inventory and explored progress retained for the flight.
    pub progress: departure::Remembered,
}

/// The running cinematics retain the cartridge completed at the elevator.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Cinematics {
    /// Station, flight and receiving loader.
    pub run: departure::Run,
    files: Model<save_files::SaveFilesPin>,
}

/// The single selected continuation from the shaft to the Landing Site.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// The room, including its interrupted write and queued picture.
    Shaft(Box<Shaft>),
    /// The station has received dispatch.
    Cinematics(Box<Cinematics>),
}

impl State {
    pub(super) fn validate(&self, content: &Content) -> Result<(), AdmissionFault> {
        match self {
            Self::Shaft(s) => s.blackout.validate().map_err(AdmissionFault::Blackout),
            Self::Cinematics(c) => c
                .run
                .validate(&content.departure)
                .map_err(AdmissionFault::Departure),
        }
    }

    pub(super) const fn files(&self) -> &save_files::State {
        match self {
            Self::Shaft(s) => &s.blackout.owners.0.files,
            Self::Cinematics(c) => &c.files.0,
        }
    }

    pub(super) fn advance(
        &mut self,
        content: &Content,
        rules: departure::Rules,
        input: departure::Input,
    ) -> Result<ProductionOutputs, ProductionFault> {
        match self {
            Self::Shaft(shaft) => {
                let departure::Input::Shaft { work, station } = input else {
                    return Err(ProductionFault::WrongScope);
                };
                let out = shaft
                    .blackout
                    .step(work, &content.shaft)
                    .map_err(|f| ProductionFault::Unsupported(Unsupported::Blackout(f)))?;
                if out
                    .requests
                    .contains(&blackout::Request::DisableShaftInterrupts)
                {
                    restart_indirect_hdma(&mut shaft.blackout.video, &content.raster_opening)
                        .map_err(|why| ProductionFault::Unsupported(Unsupported::Blackout(why)))?;
                }
                let Some(dispatch) = out.dispatch else {
                    return Ok(ProductionOutputs::Blackout(Box::new(out)));
                };
                let mut cinematic = shaft.dispatch(&dispatch);
                cinematic
                    .run
                    .validate(&content.departure)
                    .map_err(|why| ProductionFault::Unsupported(Unsupported::Departure(why)))?;
                let out = cinematic.advance(
                    &content.departure,
                    rules,
                    departure::Input::Scene(station),
                )?;
                *self = Self::Cinematics(Box::new(cinematic));
                Ok(out)
            }
            Self::Cinematics(c) => c.advance(&content.departure, rules, input),
        }
    }
}

impl Cinematics {
    fn advance(
        &mut self,
        content: &departure::Content,
        rules: departure::Rules,
        input: departure::Input,
    ) -> Result<ProductionOutputs, ProductionFault> {
        self.run
            .advance(content, rules, input)
            .map_err(|f| ProductionFault::Unsupported(Unsupported::Departure(f)))
    }
}

impl Shaft {
    fn dispatch(&mut self, receipt: &blackout::Dispatch) -> Cinematics {
        let destination = match receipt.route {
            FlightRoute::ZebesFlight => explosion::Destination::LandingSite,
        };
        let station = explosion::State::start(
            receipt.finished,
            receipt.health,
            receipt.maximum,
            destination,
            receipt.japanese_subtitles,
            receipt.palette,
            receipt.oam,
        );
        let video = std::mem::take(&mut self.blackout.video);
        let carry = departure::Carry {
            room: self.room.clone(),
            landing: self.landing.clone(),
            progress: self.progress.clone(),
            publication: video.committed,
            video: *video,
            random: self.blackout.owners.0.random.number.0,
            work_ready: true,
        };
        Cinematics {
            run: departure::Run {
                scene: departure::Scene::Station(Box::new(station)),
                carry,
            },
            files: Model(Box::new(self.blackout.owners.0.files.clone())),
        }
    }
}

// The beam reloads the active tables at the end of the vertical interval. The
// shaft disables their output before they can transfer a scanline.
fn restart_indirect_hdma(
    video: &mut Video,
    opening: &[Option<IndirectHdmaStart>; DMA_CHANNELS],
) -> Result<(), blackout::Fault> {
    for (channel, header) in video.dma.iter_mut().zip(opening) {
        if let Some(header) = header {
            if !channel.control.indirect
                || channel.source != header.source.offset
                || channel.source_bank != header.source.bank
                || header.lines == 0
            {
                return Err(blackout::Fault::Picture);
            }
            channel.count = header.indirect_address;
            channel.table = channel.source.wrapping_add(3);
            channel.line_counter = header.lines;
        }
    }
    Ok(())
}
