//! An ordinary Movement opportunity with its reached Contact and graphics consumers.

use super::appearance::{TouchdownArea, TouchdownGraphics};
use super::movement::FigurePin;
use super::sender::OfferCurrent;
use super::{
    AdmissionFault, MovementContent, MovementInputs, ProductionFault, SenderInput, Unsupported,
};
use crate::state::owned::{MovementPin, SenderPin};
use audio_transport::{CpuMasterTicks, CueId, QueueAllowance, SoundGroup};
use canonical::{Canonical, model::Model};
use samus_appearance::Figure;

/// Complete movement, the preceding selected figure and the Contact sender.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Actual carried movement state.
    pub movement: Model<MovementPin>,
    /// Figure held before the next ordinary update.
    pub figure: Model<FigurePin>,
    /// Full current FIFO and sender continuation.
    pub contact: Model<SenderPin>,
}

impl State {
    pub(super) fn validate(&self, content: &MovementContent) -> Result<(), AdmissionFault> {
        content.validate()?;
        if self.figure.0.pose != self.movement.0.pose {
            return Err(AdmissionFault::MovementTouchdownFigure);
        }
        if content.radius(self.movement.0.pose).is_none() {
            return Err(AdmissionFault::MissingPose(self.movement.0.pose));
        }
        super::sender::validate(&self.contact.0, SoundGroup::Contact)
    }
}

/// Current inputs supplied by an ordinary caller at the producer and consumer opportunities.
pub struct Input<'a> {
    /// Current movement stimuli. The playing sprite must agree with the carried figure.
    pub movement: MovementInputs,
    /// Terrain and current eligible solid actors in selection order.
    pub world: &'a dyn types::World,
    /// Current CPU sender opportunity following Movement.
    pub contact_at: CpuMasterTicks,
    /// Transport suppression owners at that opportunity.
    pub suppression: OfferCurrent,
    /// Independently admitted area arm of the later graphics opportunity.
    pub graphics_area: TouchdownArea,
}

/// Actual ordered consumption of Movement's touchdown publications.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Publication {
    /// A typed offer reached the Contact sender, even if transport suppressed it.
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
    rules: &samus_movement::Rules,
    input: Input<'_>,
) -> Result<Outputs, ProductionFault> {
    if input.movement.sprite_before_movement != state.figure.0.sprite
        || input.contact_at.0 <= state.contact.0.now.0
    {
        return Err(ProductionFault::Unsupported(
            Unsupported::MovementTouchdownCurrent,
        ));
    }
    if state.contact.0.pending.is_some() {
        return Err(ProductionFault::Unsupported(Unsupported::SenderService));
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
    let mut publications = Vec::new();
    super::sender::advance(
        &mut state.contact.0,
        SoundGroup::Contact,
        SenderInput::Advance {
            stop: input.contact_at,
            replies: &[],
        },
    )?;
    for request in &movement.touchdown_requests {
        let (cue, allowance) = super::sender::offer_touchdown(
            &mut state.contact.0,
            *request,
            input.suppression.transport_permission(),
        )?;
        publications.push(Publication::Contact { cue, allowance });
    }
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
