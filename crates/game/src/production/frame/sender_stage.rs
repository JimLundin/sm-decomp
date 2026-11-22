//! Retained sender receipts bound to physical milestones and the wall publication clock.

use super::Senders;
use crate::production::{ProductionFault, SenderInput, appearance, timing::WorkId};
use audio_transport::{
    CpuMasterTicks, OfferPermission, Request, SenderCompletion, SenderProgress, SenderState,
    SoundGroup,
    sender::{SenderOutput, SenderProfile},
};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use data::timing::{DeviceWrite, MasterTicks, Publication, PublicationPort};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// Complete current premises and the clean result chosen by a no-observation service.
/// This is a receipt, while the writable service remains in its sole sender owner.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    /// Actual group reached by caller scheduling.
    pub group: SoundGroup,
    /// Committed progress before this service.
    pub before: SenderProgress,
    /// Complete FIFO before this service.
    pub queue: std::collections::VecDeque<audio_transport::CueId>,
    /// Current independently supplied SMP reply, unused by these bounded service arms.
    pub reply: Request,
    /// Actual clean chosen result and intrinsic milestones.
    pub chosen: SenderCompletion,
}

/// Complete immutable sender receipt schema.
pub struct ReceiptPin;
impl Pinned for ReceiptPin {
    type State = Receipt;
    const NAME: &'static str = "production-frame-sender-receipt";
    const SHAPE: u64 = 0x318D_8BB8_CD11_9AB8;
}

/// Pending metadata; writable clean service progress belongs to `Senders`.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// No service is pending.
    #[default]
    Idle,
    /// Complete current result awaits full source-envelope qualification.
    Chosen(Model<ReceiptPin>),
    /// The actual reply read is charging before any clean result is available.
    AwaitingReply {
        /// Actual read prefix identity.
        work: WorkId,
        /// The unresolved observation of the actual clean sender.
        receipt: Model<ReceiptPin>,
        /// Actual wall instant when the prefix began.
        wall: MasterTicks,
    },
    /// One qualified physical service is charging.
    Charging {
        /// Actual hardware work identity.
        work: WorkId,
        /// Retained complete result used to qualify the profile.
        receipt: Model<ReceiptPin>,
        /// Actual wall instant of the latest consumed milestone.
        wall: MasterTicks,
        /// Already consumed observation offset for a reply-dependent service.
        observed: Option<u64>,
    },
}

/// Progress through actual producer requests, independent of service retries.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Offers {
    consumed: u64,
}

/// Native publication encoding admitted at the immutable import edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CueBinding {
    /// The checked semantic cue identity.
    pub cue: audio_transport::CueId,
    /// Its checked output port; no SMP input port is substituted.
    pub port: PublicationPort,
    /// Its imported outgoing byte.
    pub value: u8,
}

/// Stage admission or milestone order failed before commit.
#[derive(Debug, PartialEq, Eq)]
pub enum Fault {
    /// Work absent, repeated or not bound.
    Order,
    /// The actual service requires an independently observed reply arm.
    ReplyObservation,
    /// Current sender or pending result does not match the retained choice.
    Owner,
    /// Incorrect physical work, milestone or clock order.
    Milestone,
    /// The cue has no checked encoding for this group.
    Encoding,
    /// Actual production sender or appearance offer failed.
    Production(ProductionFault),
}

fn current(senders: &mut Senders, group: SoundGroup) -> &mut SenderState {
    match group {
        SoundGroup::Action => &mut senders.action.0,
        SoundGroup::Ambient => &mut senders.ambient.0,
        SoundGroup::Contact => &mut senders.contact.0,
    }
}

impl Offers {
    /// Actual producer prefix already consumed by completed transport offers.
    #[must_use]
    pub const fn consumed(&self) -> u64 {
        self.consumed
    }

    /// Validate saved delivery progress against the complete actual producer stream.
    /// # Errors
    /// A lost prefix or invalid retained appearance work.
    pub fn validate(&self, work: &appearance::Work) -> Result<(), Fault> {
        work.validate().map_err(|_| Fault::Owner)?;
        if usize::try_from(self.consumed).map_or(true, |first| first > work.model.0.sounds.len()) {
            return Err(Fault::Owner);
        }
        Ok(())
    }

    /// Offer every newly completed actual appearance request exactly once.
    /// Call on the frame candidate committed with producer and consumer progress.
    /// # Errors
    /// Missing retained prefix, unsupported sound or an active sender service.
    pub fn appearance(
        &mut self,
        senders: &mut Senders,
        work: &appearance::Work,
        permission: OfferPermission,
    ) -> Result<(), Fault> {
        work.validate().map_err(|_| Fault::Owner)?;
        let first = usize::try_from(self.consumed).map_err(|_| Fault::Owner)?;
        let sounds = work.model.0.sounds.get(first..).ok_or(Fault::Owner)?;
        let end = u64::try_from(work.model.0.sounds.len()).map_err(|_| Fault::Owner)?;
        let mut candidate = senders.clone();
        for sound in sounds {
            appearance::offer_sound(&mut candidate.contact.0, *sound, permission)
                .map_err(Fault::Production)?;
        }
        *senders = candidate;
        self.consumed = end;
        Ok(())
    }
}

impl Stage {
    /// Validate restored metadata against its sole current clean service owner.
    /// # Errors
    /// Missing/replaced service quantities or inconsistent retained milestone progress.
    pub fn validate(&self, senders: &Senders) -> Result<(), Fault> {
        senders.validate().map_err(|_| Fault::Owner)?;
        let Some(receipt) = self.receipt() else {
            return if senders.action.0.pending.is_none()
                && senders.ambient.0.pending.is_none()
                && senders.contact.0.pending.is_none()
            {
                Ok(())
            } else {
                Err(Fault::Owner)
            };
        };
        let sender: &SenderState = match receipt.group {
            SoundGroup::Action => &senders.action.0,
            SoundGroup::Ambient => &senders.ambient.0,
            SoundGroup::Contact => &senders.contact.0,
        };
        let pending = sender.pending.as_ref().ok_or(Fault::Owner)?;
        let chosen = &receipt.chosen;
        if matches!(self, Self::AwaitingReply { .. }) {
            return if pending == chosen
                && chosen.observation.is_some()
                && chosen.result.is_none()
                && sender.now == chosen.origin
                && sender.progress == receipt.before
                && sender.queue == receipt.queue
            {
                Ok(())
            } else {
                Err(Fault::Owner)
            };
        }
        let result = pending.result.as_ref().ok_or(Fault::Owner)?;
        let selected = chosen.result.as_ref().ok_or(Fault::Owner)?;
        let before = &receipt.before;
        if &sender.progress != before
            || sender.queue != receipt.queue
            || pending.origin != chosen.origin
            || pending.observation.is_some()
            || result.progress != selected.progress
            || result.queue != selected.queue
            || result.completion != selected.completion
            || result
                .presentation
                .as_ref()
                .is_some_and(|value| selected.presentation.as_ref() != Some(value))
            || (matches!(self, Self::Chosen(_)) && pending != chosen)
        {
            return Err(Fault::Owner);
        }
        let expected = if result.presentation.is_none() {
            selected
                .presentation
                .as_ref()
                .map_or(&chosen.origin, |(at, _)| at)
        } else {
            &chosen.origin
        };
        let expected = if result.presentation.is_some()
            && let Self::Charging {
                observed: Some(read),
                ..
            } = self
        {
            audio_transport::ExactCpuMasterTicks(std::ops::Add::add(&chosen.origin.0, *read))
        } else {
            expected.clone()
        };
        if sender.now != expected {
            return Err(Fault::Owner);
        }
        Ok(())
    }

    /// Begin from actual current hardware time, retaining the clean result without effects.
    /// Reply-reading services remain unsupported until their actual observation is bound.
    /// # Errors
    /// Pending service, invalid current owner or a reply-dependent result.
    pub fn choose(
        &mut self,
        senders: &mut Senders,
        group: SoundGroup,
        wall: MasterTicks,
        profile: SenderProfile,
    ) -> Result<&Receipt, Fault> {
        if !matches!(self, Self::Idle) {
            return Err(Fault::Order);
        }
        self.validate(senders)?;
        let (candidate, receipt) = Self::select(senders, group, wall, profile)?;
        if receipt.chosen.observation.is_some() || receipt.chosen.result.is_none() {
            return Err(Fault::ReplyObservation);
        }
        *senders = candidate;
        *self = Self::Chosen(Model(receipt));
        self.receipt().ok_or(Fault::Order)
    }

    fn select(
        senders: &Senders,
        group: SoundGroup,
        wall: MasterTicks,
        profile: SenderProfile,
    ) -> Result<(Senders, Receipt), Fault> {
        let mut candidate = senders.clone();
        candidate.validate().map_err(|_| Fault::Owner)?;
        let sender = current(&mut candidate, group);
        if sender.pending.is_some() {
            return Err(Fault::Order);
        }
        let outputs = crate::production::sender::advance(
            sender,
            group,
            SenderInput::Advance {
                stop: CpuMasterTicks(wall.0),
                replies: &[],
            },
        )
        .map_err(Fault::Production)?;
        if !outputs.is_empty() {
            return Err(Fault::Owner);
        }
        let before = sender.progress.clone();
        let queue = sender.queue.clone();
        let reply = sender.reply;
        crate::production::sender::advance(sender, group, SenderInput::Begin(profile))
            .map_err(Fault::Production)?;
        let chosen = sender.pending.as_ref().ok_or(Fault::Owner)?.clone();
        Ok((
            candidate,
            Receipt {
                group,
                before,
                queue,
                reply,
                chosen,
            },
        ))
    }

    pub(super) fn begin_contact_reply(
        &mut self,
        senders: &mut Senders,
        work: WorkId,
        wall: MasterTicks,
    ) -> Result<(), Fault> {
        if !matches!(self, Self::Idle) {
            return Err(Fault::Order);
        }
        self.validate(senders)?;
        let (candidate, receipt) =
            Self::select(senders, SoundGroup::Contact, wall, SenderProfile::FastBus)?;
        if receipt.before != SenderProgress::AwaitCue(audio_transport::CueId::Footstep)
            || !receipt.queue.is_empty()
            || receipt.chosen.observation.is_none()
            || receipt.chosen.result.is_some()
        {
            return Err(Fault::ReplyObservation);
        }
        *senders = candidate;
        *self = Self::AwaitingReply {
            work,
            receipt: Model(receipt),
            wall,
        };
        Ok(())
    }

    pub(super) fn consume_contact_reply(
        &mut self,
        senders: &mut Senders,
        work: [WorkId; 2],
        at: MasterTicks,
        intrinsic: u64,
        reply: Request,
    ) -> Result<&Receipt, Fault> {
        self.validate(senders)?;
        let Self::AwaitingReply {
            work: active,
            receipt,
            wall,
        } = self
        else {
            return Err(Fault::Order);
        };
        let [prefix, body] = work;
        let chosen = &receipt.0.chosen;
        let stop =
            audio_transport::ExactCpuMasterTicks(std::ops::Add::add(&chosen.origin.0, intrinsic));
        if *active != prefix
            || at < *wall
            || chosen.observation.as_ref() != Some(&stop)
            || CpuMasterTicks(at.0).0 < stop.0
        {
            return Err(Fault::Milestone);
        }
        let mut candidate = senders.clone();
        let sender = current(&mut candidate, receipt.0.group);
        let outputs = crate::production::sender::advance(
            sender,
            receipt.0.group,
            SenderInput::Advance {
                stop: stop.clone(),
                replies: &[audio_transport::sender::ReplyInput {
                    at: stop.clone(),
                    reply,
                }],
            },
        )
        .map_err(Fault::Production)?;
        if outputs != [SenderOutput::Observation(stop, reply)] {
            return Err(Fault::Milestone);
        }
        let mut observed = receipt.clone();
        observed.0.reply = reply;
        observed.0.chosen = sender.pending.clone().ok_or(Fault::Owner)?;
        *senders = candidate;
        *self = Self::Charging {
            work: body,
            receipt: observed,
            wall: at,
            observed: Some(intrinsic),
        };
        self.receipt().ok_or(Fault::Order)
    }

    /// Complete receipt used by the external full source-envelope qualifier.
    #[must_use]
    pub const fn receipt(&self) -> Option<&Receipt> {
        match self {
            Self::Idle => None,
            Self::Chosen(receipt)
            | Self::AwaitingReply { receipt, .. }
            | Self::Charging { receipt, .. } => Some(&receipt.0),
        }
    }

    /// Bind the fully qualified physical service begun by the central caller.
    /// # Errors
    /// Missing choice or a different wall origin.
    pub fn bind(&mut self, work: WorkId, wall: MasterTicks) -> Result<(), Fault> {
        let Self::Chosen(receipt) = self else {
            return Err(Fault::Order);
        };
        if receipt.0.chosen.origin != CpuMasterTicks(wall.0) {
            return Err(Fault::Milestone);
        }
        *self = Self::Charging {
            work,
            receipt: receipt.clone(),
            wall,
            observed: None,
        };
        Ok(())
    }

    /// Resolve an actual pending request into its imported outgoing publication byte.
    /// The hardware profile owns publication order and its actual wall instant.
    /// # Errors
    /// No pending presentation or the binding names a different cue/group.
    pub fn publication(&self, binding: CueBinding) -> Result<DeviceWrite, Fault> {
        let receipt = self.receipt().ok_or(Fault::Order)?;
        let result = receipt.chosen.result.as_ref().ok_or(Fault::Owner)?;
        let (_, request) = result.presentation.as_ref().ok_or(Fault::Order)?;
        if *request != Request::Cue(binding.cue)
            || audio_transport::effects::group(binding.cue) != receipt.group
            || binding.port.output_class() != Ok(2)
        {
            return Err(Fault::Encoding);
        }
        Ok(DeviceWrite {
            port: binding.port,
            value: binding.value,
            effect: Publication::Downstream,
        })
    }

    /// Consume the actual qualified presentation boundary, independently of wall stalls.
    /// `intrinsic` is service progress established by the completed physical exposure.
    /// # Errors
    /// Incorrect owner, progress, work identity or wall order.
    pub fn present(
        &mut self,
        senders: &mut Senders,
        work: WorkId,
        intrinsic: u64,
        wall: MasterTicks,
    ) -> Result<Request, Fault> {
        let Self::Charging { receipt, .. } = self else {
            return Err(Fault::Order);
        };
        let (_, expected) = receipt
            .0
            .chosen
            .result
            .as_ref()
            .and_then(|result| result.presentation.as_ref())
            .ok_or(Fault::Order)?;
        let expected = *expected;
        self.advance(senders, work, intrinsic, wall, false)?;
        Ok(expected)
    }

    /// Commit the clean result only at actual physical service completion.
    /// Hardware time selects no intrinsic milestone; the full receipt qualifies progress.
    /// # Errors
    /// Early/repeated completion or a different retained service/physical identity.
    pub fn complete(
        &mut self,
        senders: &mut Senders,
        work: WorkId,
        intrinsic: u64,
        wall: MasterTicks,
    ) -> Result<Receipt, Fault> {
        let receipt = self.receipt().ok_or(Fault::Order)?.clone();
        self.advance(senders, work, intrinsic, wall, true)?;
        *self = Self::Idle;
        Ok(receipt)
    }

    fn advance(
        &mut self,
        senders: &mut Senders,
        completed: WorkId,
        intrinsic: u64,
        at: MasterTicks,
        completion: bool,
    ) -> Result<(), Fault> {
        self.validate(senders)?;
        let Self::Charging {
            work,
            receipt,
            wall,
            ..
        } = self
        else {
            return Err(Fault::Order);
        };
        if *work != completed || at < *wall {
            return Err(Fault::Milestone);
        }
        let group = receipt.0.group;
        let mut candidate = senders.clone();
        let sender = current(&mut candidate, group);
        let pending = sender.pending.as_ref().ok_or(Fault::Owner)?;
        let chosen = &receipt.0.chosen;
        let result = pending.result.as_ref().ok_or(Fault::Owner)?;
        let selected = chosen.result.as_ref().ok_or(Fault::Owner)?;
        if sender.progress != receipt.0.before
            || sender.queue != receipt.0.queue
            || pending.origin != chosen.origin
            || pending.observation.is_some()
            || result.progress != selected.progress
            || result.queue != selected.queue
            || result.completion != selected.completion
            || result
                .presentation
                .as_ref()
                .is_some_and(|presentation| selected.presentation.as_ref() != Some(presentation))
            || (result.presentation.is_none()
                && selected
                    .presentation
                    .as_ref()
                    .is_some_and(|(presentation, _)| sender.now != *presentation))
        {
            return Err(Fault::Owner);
        }
        let stop =
            audio_transport::ExactCpuMasterTicks(std::ops::Add::add(&chosen.origin.0, intrinsic));
        let milestone = result
            .presentation
            .as_ref()
            .map_or(&result.completion, |(time, _)| time);
        if stop != *milestone
            || CpuMasterTicks(at.0).0 < stop.0
            || completion != result.presentation.is_none()
        {
            return Err(Fault::Milestone);
        }
        let outputs = crate::production::sender::advance(
            sender,
            group,
            SenderInput::Advance { stop, replies: &[] },
        )
        .map_err(Fault::Production)?;
        if !matches!(outputs.as_slice(), [SenderOutput::Completed(_)] if completion)
            && !matches!(outputs.as_slice(), [SenderOutput::Presentation(_, _)] if !completion)
        {
            return Err(Fault::Milestone);
        }
        *senders = candidate;
        *wall = at;
        Ok(())
    }
}
