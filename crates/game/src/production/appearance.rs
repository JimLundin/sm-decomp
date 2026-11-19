//! Causal animation reads, interrupted work and exactly-once intent delivery.

use canonical::Canonical;
use canonical::model::{Model, Pinned};

#[cfg(test)]
mod tests;

/// Complete interrupted animation and read-only opportunity state.
pub struct WorkPin;
impl Pinned for WorkPin {
    type State = samus_appearance::SuspendedAppearance;
    const NAME: &'static str = "production-appearance-work";
    const SHAPE: u64 = 0xBCB5_B96E_127E_8E92;
}

/// Engine-owned animation continuation and external delivery progress.
/// The clean work is the sole current animation state while it is active.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Work {
    /// Pending decisions, complete current quantities and retained produced intents.
    pub model: Model<WorkPin>,
    sounds_delivered: u64,
    graphics_delivered: u64,
}

/// Intents newly delivered to the actual downstream consumers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outputs {
    /// Ordered animation and appearance sound requests.
    pub sounds: Vec<samus_appearance::PublishedSound>,
    /// Graphical contacts, including contacts with suppressed sound.
    pub graphics: Vec<samus_appearance::GraphicalContact>,
}

/// An invalid imported engine continuation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkFault {
    /// Consumed operation metadata still has pending work or unpublished intents.
    Retirement,
    /// A delivery cursor exceeds its retained produced intents.
    DeliveryCursor,
    /// The consumed opportunity cursor exceeds the retained plan.
    OpportunityCursor,
    /// The retained answer does not belong to the next opportunity or current quantities.
    PendingDecision,
}

impl Work {
    /// Retire only completed and published operation history, keeping current model quantities.
    /// # Errors
    /// Pending answers, unfinished steps and undelivered intents remain owned until consumed.
    pub fn retire(&mut self) -> Result<(), WorkFault> {
        self.validate()?;
        if self.model.0.pending.is_some()
            || self.model.0.consumed != self.model.0.steps.len()
            || usize::try_from(self.sounds_delivered).ok() != Some(self.model.0.sounds.len())
            || usize::try_from(self.graphics_delivered).ok() != Some(self.model.0.graphics.len())
        {
            return Err(WorkFault::Retirement);
        }
        self.model.0.steps.clear();
        self.model.0.consumed = 0;
        self.model.0.completed.clear();
        self.model.0.sounds.clear();
        self.model.0.graphics.clear();
        self.model.0.animation = None;
        self.model.0.ahead = None;
        self.sounds_delivered = 0;
        self.graphics_delivered = 0;
        Ok(())
    }
    /// Retain actual work without deciding or completing an opportunity.
    #[must_use]
    pub const fn new(model: samus_appearance::SuspendedAppearance) -> Self {
        Self {
            model: Model(model),
            sounds_delivered: 0,
            graphics_delivered: 0,
        }
    }

    /// Validate engine-owned cursor bounds before admitting a restored continuation.
    ///
    /// # Errors
    /// Cursor positions outside their owned sequences.
    pub fn validate(&self) -> Result<(), WorkFault> {
        if usize::try_from(self.sounds_delivered)
            .map_or(true, |position| position > self.model.0.sounds.len())
            || usize::try_from(self.graphics_delivered)
                .map_or(true, |position| position > self.model.0.graphics.len())
        {
            return Err(WorkFault::DeliveryCursor);
        }
        if self.model.0.consumed > self.model.0.steps.len() {
            return Err(WorkFault::OpportunityCursor);
        }
        if let Some(pending) = &self.model.0.pending
            && (pending.before != self.model.0.state
                || !matches!(self.model.0.steps.get(self.model.0.consumed),
                    Some(samus_appearance::WorkStep::Animation(rule)) if *rule == pending.rule))
        {
            return Err(WorkFault::PendingDecision);
        }
        Ok(())
    }

    /// Choose the next actual animation answer without applying its effects.
    /// `None` also covers a pending read-only opportunity; it is not completion.
    pub fn offer(&mut self) -> Option<&samus_appearance::OfferedDecision> {
        self.model.0.offer()
    }

    /// Complete exactly one offered animation answer or actual read-only opportunity.
    /// The caller admits dependent timing work before calling this operation.
    pub fn complete(&mut self) -> bool {
        self.model.0.complete()
    }

    /// Deliver produced intents once, retaining consumer positions in the saved owner.
    /// Call on the frame's private candidate and commit it with downstream consumers.
    ///
    /// # Errors
    /// Invalid delivery positions reject before either cursor changes.
    pub fn deliver(&mut self) -> Result<Outputs, WorkFault> {
        self.validate()?;
        let sounds =
            usize::try_from(self.sounds_delivered).map_err(|_| WorkFault::DeliveryCursor)?;
        let graphics =
            usize::try_from(self.graphics_delivered).map_err(|_| WorkFault::DeliveryCursor)?;
        let sounds_delivered =
            u64::try_from(self.model.0.sounds.len()).map_err(|_| WorkFault::DeliveryCursor)?;
        let graphics_delivered =
            u64::try_from(self.model.0.graphics.len()).map_err(|_| WorkFault::DeliveryCursor)?;
        let outputs = Outputs {
            sounds: self.model.0.sounds.iter().skip(sounds).copied().collect(),
            graphics: self
                .model
                .0
                .graphics
                .iter()
                .skip(graphics)
                .copied()
                .collect(),
        };
        self.sounds_delivered = sounds_delivered;
        self.graphics_delivered = graphics_delivered;
        Ok(outputs)
    }
}

/// The complete causal animation feedback, without advancing the playhead.
#[must_use]
pub fn ahead(
    state: &samus_appearance::State,
    clip: &samus_appearance::Clip,
) -> samus_appearance::Ahead {
    samus_appearance::Ahead::of(state.playhead, clip)
}

/// Route the actual contact request to its admitted sender queue.
/// Permission and queue contents come from their current owners at this opportunity.
///
/// # Errors
/// An unqualified cue mapping or an offer during a pending sender service.
pub fn offer_sound(
    state: &mut audio_transport::SenderState,
    sound: samus_appearance::PublishedSound,
    permission: audio_transport::OfferPermission,
) -> Result<(), super::ProductionFault> {
    if sound
        != samus_appearance::PublishedSound::Appearance(samus_appearance::SoundRequest::Footstep)
    {
        return Err(super::ProductionFault::Unsupported(
            super::Unsupported::AppearanceSound(sound),
        ));
    }
    super::sender::advance(
        state,
        audio_transport::SoundGroup::Contact,
        super::SenderInput::Offer {
            cue: audio_transport::CueId::Footstep,
            allowance: audio_transport::QueueAllowance::Six,
            permission,
        },
    )?;
    Ok(())
}
