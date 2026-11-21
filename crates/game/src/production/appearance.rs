//! Causal animation reads, interrupted work and exactly-once intent delivery.

use canonical::Canonical;
use canonical::model::{Model, Pinned};

#[cfg(test)]
mod cannon_tests;
#[cfg(test)]
mod tests;

/// Complete interrupted animation and read-only opportunity state.
pub struct WorkPin;
impl Pinned for WorkPin {
    type State = samus_appearance::SuspendedAppearance;
    const NAME: &'static str = "production-appearance-work";
    const SHAPE: u64 = 0xBCB5_B96E_127E_8E92;
}

/// Complete independently retained cannon selection delay.
pub struct CannonDelayPin;
impl Pinned for CannonDelayPin {
    type State = samus_appearance::CannonDelay;
    const NAME: &'static str = "production-cannon-delay";
    const SHAPE: u64 = 0x1C2D_9610_8473_F464;
}

/// Complete chosen and consumed actual item-control observation.
pub struct CannonObservationPin;
impl Pinned for CannonObservationPin {
    type State = samus_appearance::CannonObservation;
    const NAME: &'static str = "production-cannon-observation";
    const SHAPE: u64 = 0x645D_83DD_3E18_0090;
}

/// Engine-owned animation continuation and external delivery progress.
/// The clean work is the sole current animation state while it is active.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Work {
    /// Pending decisions, complete current quantities and retained produced intents.
    pub model: Model<WorkPin>,
    /// Independently admitted readiness; scoped animation alone does not supply it.
    pub cannon: Option<Model<CannonDelayPin>>,
    /// Actual control answer retained independently of later HUD selection changes.
    pub cannon_observation: Model<CannonObservationPin>,
    /// Read-only admitted pre-control receipt used to qualify the current delay.
    pub cannon_before: Option<Model<CannonDelayPin>>,
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
    /// A cannon consumer lacks independently admitted readiness or its actual controls.
    Cannon,
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
    /// Retire frame-scoped controls alongside completed animation work.
    ///
    /// # Errors
    /// Unfinished or undelivered work cannot be retired.
    pub(crate) fn retire_frame(&mut self) -> Result<(), WorkFault> {
        if matches!(
            self.cannon_observation.0,
            samus_appearance::CannonObservation::Chosen(_)
        ) {
            return Err(WorkFault::Retirement);
        }
        self.retire()?;
        self.cannon_before = self.cannon;
        self.cannon_observation.0 = samus_appearance::CannonObservation::Unmade;
        Ok(())
    }

    /// Retain actual work without deciding or completing an opportunity.
    #[must_use]
    pub const fn new(model: samus_appearance::SuspendedAppearance) -> Self {
        Self {
            model: Model(model),
            cannon: None,
            cannon_observation: Model(samus_appearance::CannonObservation::Unmade),
            cannon_before: None,
            sounds_delivered: 0,
            graphics_delivered: 0,
        }
    }

    /// Retain initial readiness from an independent observation with the animation owner.
    #[must_use]
    pub const fn with_cannon(
        model: samus_appearance::SuspendedAppearance,
        cannon: samus_appearance::CannonDelay,
    ) -> Self {
        let mut work = Self::new(model);
        work.cannon = Some(Model(cannon));
        work.cannon_before = Some(Model(cannon));
        work
    }

    /// Retain identities from the actual starting HUD selection and chosen controls.
    /// # Errors
    /// Missing readiness or a repeated observation rejects before changing the owner.
    pub fn choose_cannon(
        &mut self,
        before: interface_hud::Selection,
        answer: &interface_hud::ControlAnswer,
    ) -> Result<(), WorkFault> {
        if self.cannon.is_none()
            || self.cannon != self.cannon_before
            || !self
                .cannon_observation
                .0
                .choose(cannon_answer(before, answer))
        {
            return Err(WorkFault::Cannon);
        }
        self.cannon_before = self.cannon;
        Ok(())
    }

    /// Join the actual still-starting selection and retained chosen controls at a hold.
    /// # Errors
    /// Replaced controls or changed lifecycle rejects without changing readiness.
    pub fn validate_cannon_choice(
        &self,
        before: interface_hud::Selection,
        answer: &interface_hud::ControlAnswer,
    ) -> Result<(), WorkFault> {
        if self.cannon.is_none()
            || self.cannon != self.cannon_before
            || self.cannon_observation.0
                != samus_appearance::CannonObservation::Chosen(cannon_answer(before, answer))
        {
            return Err(WorkFault::Cannon);
        }
        Ok(())
    }

    /// Consume the retained clean answer only with the actual completed HUD answer.
    /// # Errors
    /// Missing readiness, a replaced answer or repeated completion rejects atomically.
    pub fn complete_cannon(
        &mut self,
        answer: &interface_hud::ControlAnswer,
    ) -> Result<(), WorkFault> {
        use samus_appearance::{CannonObservation, Look};
        let CannonObservation::Chosen(chosen) = self.cannon_observation.0 else {
            return Err(WorkFault::Cannon);
        };
        if self.cannon != self.cannon_before
            || chosen.after != selected_item(answer.selection)
            || chosen.eligibility != eligibility(answer.inputs.can_switch)
        {
            return Err(WorkFault::Cannon);
        }
        let cannon = self.cannon.as_mut().ok_or(WorkFault::Cannon)?;
        let mut look = Look {
            animation: self.model.0.state,
            cannon: cannon.0,
        };
        if !self.cannon_observation.0.complete(&mut look) {
            return Err(WorkFault::Cannon);
        }
        self.model.0.state = look.animation;
        cannon.0 = look.cannon;
        Ok(())
    }

    /// Join the retained consumed observation to its actual completed HUD producer.
    /// # Errors
    /// Missing, altered or unconsumed readiness and a replaced producer reject.
    pub fn validate_cannon_completion(
        &self,
        before: interface_hud::Selection,
        answer: &interface_hud::ControlAnswer,
    ) -> Result<(), WorkFault> {
        self.validate()?;
        if self.cannon.is_none()
            || self.cannon_observation.0
                != samus_appearance::CannonObservation::Completed(cannon_answer(before, answer))
        {
            return Err(WorkFault::Cannon);
        }
        Ok(())
    }

    /// Validate engine-owned cursor bounds before admitting a restored continuation.
    ///
    /// # Errors
    /// Cursor positions outside their owned sequences.
    pub fn validate(&self) -> Result<(), WorkFault> {
        use samus_appearance::CannonObservation;
        match self.cannon_observation.0 {
            CannonObservation::Unmade => {
                if self.cannon != self.cannon_before {
                    return Err(WorkFault::Cannon);
                }
            }
            CannonObservation::Chosen(_) => {
                if self.cannon.is_none() || self.cannon != self.cannon_before {
                    return Err(WorkFault::Cannon);
                }
            }
            CannonObservation::Completed(answer) => {
                let mut expected = self.cannon_before.ok_or(WorkFault::Cannon)?;
                expected.0.controls_completed(answer);
                if self.cannon.as_ref() != Some(&expected) {
                    return Err(WorkFault::Cannon);
                }
            }
        }
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

const fn selected_item(selection: interface_hud::Selection) -> samus_appearance::SelectedItem {
    use interface_hud::{Ammo, SelectableItem};
    use samus_appearance::SelectedItem;
    match selection.item() {
        None => SelectedItem::None,
        Some(SelectableItem::Ammo(Ammo::Missiles)) => SelectedItem::Missiles,
        Some(SelectableItem::Ammo(Ammo::SuperMissiles)) => SelectedItem::SuperMissiles,
        Some(SelectableItem::Ammo(Ammo::PowerBombs)) => SelectedItem::PowerBombs,
        Some(SelectableItem::GrappleBeam) => SelectedItem::Grapple,
        Some(SelectableItem::XRayScope) => SelectedItem::Xray,
    }
}

const fn eligibility(can_switch: bool) -> samus_appearance::ControlEligibility {
    if can_switch {
        samus_appearance::ControlEligibility::Eligible
    } else {
        samus_appearance::ControlEligibility::Ineligible
    }
}

const fn cannon_answer(
    before: interface_hud::Selection,
    answer: &interface_hud::ControlAnswer,
) -> samus_appearance::ItemControlAnswer {
    samus_appearance::ItemControlAnswer {
        before: selected_item(before),
        after: selected_item(answer.selection),
        eligibility: eligibility(answer.inputs.can_switch),
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
