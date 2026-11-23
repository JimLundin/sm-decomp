//! An ordinary Movement opportunity with its reached Contact and graphics consumers.

use super::appearance::{TouchdownArea, TouchdownGraphics};
use super::audio::Content as AudioContent;
use super::movement::FigurePin;
use super::sender::OfferCurrent;
use super::{AdmissionFault, MovementContent, MovementInputs, ProductionFault, Unsupported};
use crate::state::owned::{AudioPin, MovementPin};
use audio_transport::frame::FrameCadence;
use audio_transport::{CueId, QueueAllowance};
use canonical::{Canonical, model::Model};
use samus_appearance::Figure;

/// Complete movement, the preceding selected figure and the audio transport its offer reaches.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Actual carried movement state.
    pub movement: Model<MovementPin>,
    /// Figure held before the next ordinary update.
    pub figure: Model<FigurePin>,
    /// The complete audio transport, whose Contact queue takes the touchdown offer.
    pub audio: Model<AudioPin>,
}

impl State {
    pub(super) fn validate(
        &self,
        content: &MovementContent,
        audio: &AudioContent,
    ) -> Result<(), AdmissionFault> {
        content.validate()?;
        if self.figure.0.pose != self.movement.0.pose {
            return Err(AdmissionFault::MovementTouchdownFigure);
        }
        if content.radius(self.movement.0.pose).is_none() {
            return Err(AdmissionFault::MissingPose(self.movement.0.pose));
        }
        super::audio::validate(&self.audio.0, audio)
    }
}

/// Current inputs supplied by an ordinary caller at the producer and consumer opportunities.
pub struct Input<'a> {
    /// Current movement stimuli. The playing sprite must agree with the carried figure.
    pub movement: MovementInputs,
    /// Terrain and current eligible solid actors in selection order.
    pub world: &'a dyn types::World,
    /// Transport suppression owners when Movement offers its sound.
    pub suppression: OfferCurrent,
    /// The frame's audio cadence after Movement's offers, borrowed from the platform.
    pub audio: FrameCadence,
    /// Independently admitted area arm of the later graphics opportunity.
    pub graphics_area: TouchdownArea,
}

/// Actual ordered consumption of Movement's touchdown publications.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Publication {
    /// A typed offer reached the Contact queue, even if transport suppressed it.
    Contact {
        /// The cue offered to Contact.
        cue: CueId,
        /// Actual producer allowance used by the sender.
        allowance: QueueAllowance,
    },
    /// A graphics request reached Appearance with the selected figure.
    Graphics(TouchdownGraphics),
}

/// Complete Movement output and the ordered consumer calls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outputs {
    /// Every ordinary Movement receipt.
    pub movement: samus_movement::Outcome,
    /// Consumer calls in the order they were completed.
    pub publications: Vec<Publication>,
}

pub(super) fn advance(
    state: &mut State,
    content: &MovementContent,
    audio: &AudioContent,
    rules: &samus_movement::Rules,
    input: &Input<'_>,
) -> Result<Outputs, ProductionFault> {
    if input.movement.sprite_before_movement != state.figure.0.sprite {
        return Err(ProductionFault::Unsupported(
            Unsupported::MovementTouchdownCurrent,
        ));
    }
    let movement = super::movement::advance_scoped(
        &mut state.movement.0,
        content,
        rules,
        &input.movement,
        input.world,
    )?;
    let figure = Figure {
        pose: state.movement.0.pose,
        sprite: movement.sprite,
    };
    let offers = movement
        .touchdown_requests
        .iter()
        .map(|request| super::sender::offer_touchdown(*request, input.suppression))
        .collect::<Result<Vec<_>, _>>()?;
    super::audio::play(&mut state.audio.0, audio, &offers, input.audio);
    let mut publications: Vec<_> = offers
        .iter()
        .map(|offer| Publication::Contact {
            cue: offer.cue,
            allowance: offer.allowance,
        })
        .collect();
    if movement.touchdown_graphics {
        publications.push(Publication::Graphics(
            super::appearance::touchdown_graphics(figure, input.graphics_area),
        ));
    }
    state.figure = Model(figure);
    Ok(Outputs {
        movement,
        publications,
    })
}
