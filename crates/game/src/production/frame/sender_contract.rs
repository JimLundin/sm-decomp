//! Current sender qualification and physical queue representation.

pub use super::contact_sender::ContactContinuation;
use super::{Senders, sender_stage::Receipt};
use audio_transport::{CueId, Request, SenderProgress, SenderState, SoundGroup};
use canonical::Canonical;
use data::timing::{Profile, Template};
use std::collections::VecDeque;

#[cfg(test)]
mod tests;

/// Invalid current queue representation or producer transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Cursors do not represent the actual sole clean FIFO.
    Queue,
    /// The observed producer transition is not an append or retained dequeue.
    Transition,
    /// Current borrowed execution regime is outside the qualified source arm.
    Current,
    /// Complete physical shape or selected intrinsic milestone differs.
    Profile,
    /// Actual clean service differs from the full current source predicate.
    Receipt,
}

/// Retained physical fetch regime; CPU execution metadata never becomes model state.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BusRegime {
    /// Qualified six-clock instruction fetches.
    Fast,
    /// Eight-clock instruction fetches require a separately qualified template.
    Slow,
}

/// Independently borrowed current quantities preserved through the guarded caller scope.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Current {
    /// Current global service downtime.
    pub downtime: u8,
    /// Current physical instruction-fetch regime.
    pub bus: BusRegime,
}

impl Current {
    /// Check actual retained execution quantities at a dependent service opportunity.
    /// # Errors
    /// Unqualified downtime or physical fetch regime.
    pub const fn validate(self) -> Result<(), Fault> {
        if self.downtime != 0 || !matches!(self.bus, BusRegime::Fast) {
            return Err(Fault::Current);
        }
        Ok(())
    }
}

/// One complete service body and its separately charged caller continuation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    group: SoundGroup,
    body: data::timing::Template,
    pub(super) tail: data::timing::Template,
}

/// Actual current publications bound to the whole qualified physical service.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Qualified {
    /// Whole service body, bound from the actual chosen request.
    pub body: data::timing::Profile,
    /// Caller work beginning only after clean service completion.
    pub tail: data::timing::Profile,
    /// Intrinsic progress at the completed publication exposure.
    pub publication: Option<u64>,
    /// Intrinsic progress at clean service completion.
    pub completion: u64,
}

/// Complete immutable caller and three actual sender service envelopes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalContent {
    /// Caller admission before the first service.
    pub admission: Admission,
    /// Action service and its caller continuation.
    pub action: Contract,
    /// Ambient service and its caller continuation.
    pub ambient: Contract,
    /// Contact service and its caller continuation.
    pub contact: Contract,
    /// Companion scope for an actual reply to an already delivered Contact cue.
    pub contact_retry: Option<ContactRetry>,
    /// Qualified clear and empty phases of the same current Contact sender.
    pub contact_continuation: Option<ContactContinuation>,
}

/// Bounded native Contact retry with an independently admitted incoming sample.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContactRetry {
    pub(super) prefix: Profile,
    remainder: Template,
    reply: Request,
    matching: Option<ContactMatch>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ContactMatch {
    after_retry: data::timing::MasterTicks,
    remainder: Profile,
}

impl ContactRetry {
    pub(super) const SAMPLE: u64 = 58;
    const PREFIX: u64 = 62;

    /// Bind both physical portions and the actual at-use external input.
    /// # Errors
    /// Invalid exposure/split, output port or reply arm.
    pub fn new(prefix: &Template, remainder: Template, reply: Request) -> Result<Self, Fault> {
        if milestones(prefix)? != (Self::PREFIX, None)
            || milestones(&remainder)? != (100, Some(42))
            || reply != Request::Absent
        {
            return Err(Fault::Profile);
        }
        Ok(Self {
            prefix: prefix.bind(&[]).map_err(|_| Fault::Profile)?,
            remainder,
            reply,
            matching: None,
        })
    }

    /// Admit one scoped matching external publication after an actual retry.
    ///
    /// # Errors
    /// Unqualified response delay or physical continuation.
    pub fn admit_match(
        &mut self,
        prefix: &Template,
        remainder: &Template,
        after_retry: data::timing::MasterTicks,
    ) -> Result<(), Fault> {
        if after_retry.0 == 0
            || prefix.bind(&[]).map_err(|_| Fault::Profile)? != self.prefix
            || milestones(remainder)? != (148, None)
        {
            return Err(Fault::Profile);
        }
        self.matching = Some(ContactMatch {
            after_retry,
            remainder: remainder.bind(&[]).map_err(|_| Fault::Profile)?,
        });
        Ok(())
    }

    pub(super) fn reply_at(
        &self,
        retry: Option<data::timing::MasterTicks>,
        now: data::timing::MasterTicks,
    ) -> Request {
        let reached = self
            .matching
            .as_ref()
            .zip(retry)
            .is_some_and(|(matching, retry)| {
                retry
                    .0
                    .checked_add(matching.after_retry.0)
                    .is_some_and(|at| at <= now.0)
            });
        if reached {
            Request::Cue(CueId::Footstep)
        } else {
            self.reply
        }
    }

    pub(super) fn qualify(
        &self,
        receipt: &Receipt,
        queues: &Queues,
        binding: super::sender_stage::CueBinding,
        caller_tail: &Template,
    ) -> Result<Qualified, Fault> {
        let chosen = receipt.chosen.result.as_ref().ok_or(Fault::Receipt)?;
        let cue = CueId::Footstep;
        if receipt.group != SoundGroup::Contact
            || receipt.before != SenderProgress::AwaitCue(cue)
            || receipt.chosen.observation.is_some()
            || !receipt.queue.is_empty()
            || queues.contact.positions() != [1, 1]
            || !chosen.queue.is_empty()
            || binding.cue != cue
            || binding.port.output_class() != Ok(2)
        {
            return Err(Fault::Receipt);
        }
        if receipt.reply == Request::Cue(cue) {
            let matching = self.matching.as_ref().ok_or(Fault::Receipt)?;
            if !matches!(chosen.progress, SenderProgress::ClearDelay(delay) if delay.get() == 2)
                || chosen.completion.0 != std::ops::Add::add(&receipt.chosen.origin.0, 210_u64)
                || chosen.presentation.is_some()
            {
                return Err(Fault::Receipt);
            }
            return Ok(Qualified {
                body: matching.remainder.clone(),
                tail: caller_tail.bind(&[]).map_err(|_| Fault::Profile)?,
                publication: None,
                completion: 210,
            });
        }
        if receipt.reply != self.reply
            || chosen.progress != SenderProgress::AwaitCue(cue)
            || chosen.completion.0 != std::ops::Add::add(&receipt.chosen.origin.0, 162_u64)
            || chosen.presentation.as_ref().is_none_or(|(at, request)| {
                at.0 != std::ops::Add::add(&receipt.chosen.origin.0, 104_u64)
                    || *request != Request::Cue(cue)
            })
        {
            return Err(Fault::Receipt);
        }
        let write = data::timing::DeviceWrite {
            port: binding.port,
            value: binding.value,
            effect: data::timing::Publication::Downstream,
        };
        Ok(Qualified {
            body: self.remainder.bind(&[write]).map_err(|_| Fault::Profile)?,
            tail: caller_tail.bind(&[]).map_err(|_| Fault::Profile)?,
            publication: Some(104),
            completion: 162,
        })
    }
}

pub(super) fn milestones(template: &data::timing::Template) -> Result<(u64, Option<u64>), Fault> {
    use data::timing::TemplateEffect;
    template.validate().map_err(|_| Fault::Profile)?;
    let mut ticks = 0_u64;
    let mut publication = None;
    for run in &template.runs {
        let duration = u64::from(run.count)
            .checked_mul(u64::from(run.cycle.ticks()))
            .ok_or(Fault::Profile)?;
        ticks = ticks.checked_add(duration).ok_or(Fault::Profile)?;
        match run.effect {
            TemplateEffect::None => {}
            TemplateEffect::Sample(_) => return Err(Fault::Profile),
            TemplateEffect::Publish(_) => {
                if run.count != 1 || publication.replace(ticks).is_some() {
                    return Err(Fault::Profile);
                }
            }
        }
    }
    Ok((ticks, publication))
}

impl Contract {
    /// Import complete source-qualified body and tail without runtime source navigation.
    /// # Errors
    /// Physical effects or intrinsic boundaries differ from the bounded source arm.
    pub fn new(
        group: SoundGroup,
        body: data::timing::Template,
        tail: data::timing::Template,
    ) -> Result<Self, Fault> {
        let expected = match group {
            SoundGroup::Action | SoundGroup::Ambient => ((122, None), (190, None)),
            SoundGroup::Contact => ((398, Some(212)), (130, None)),
        };
        if milestones(&body)? != expected.0 || milestones(&tail)? != expected.1 {
            return Err(Fault::Profile);
        }
        Ok(Self { group, body, tail })
    }

    /// Validate the entire actual clean choice and bind its checked publication.
    /// # Errors
    /// Current predicates, selected result, queue positions or milestones differ.
    pub fn qualify(
        &self,
        receipt: &Receipt,
        queues: &Queues,
        current: Current,
        binding: super::sender_stage::CueBinding,
    ) -> Result<Qualified, Fault> {
        current.validate()?;
        let window = queues.window(receipt.group);
        window.validate(&receipt.queue)?;
        let result = receipt.chosen.result.as_ref().ok_or(Fault::Receipt)?;
        if receipt.group != self.group
            || receipt.before != SenderProgress::Ready
            || receipt.chosen.observation.is_some()
            || !result.queue.is_empty()
        {
            return Err(Fault::Receipt);
        }
        let (completion, publication) = milestones(&self.body)?;
        let end = std::ops::Add::add(&receipt.chosen.origin.0, completion);
        if result.completion.0 != end {
            return Err(Fault::Receipt);
        }
        let writes = match self.group {
            SoundGroup::Action | SoundGroup::Ambient => {
                if !receipt.queue.is_empty()
                    || window.head != window.tail
                    || result.progress != SenderProgress::Ready
                    || result.presentation.is_some()
                {
                    return Err(Fault::Receipt);
                }
                Vec::new()
            }
            SoundGroup::Contact => {
                let at = std::ops::Add::add(
                    &receipt.chosen.origin.0,
                    publication.ok_or(Fault::Profile)?,
                );
                if window.positions() != [0, 1]
                    || receipt.queue.iter().copied().ne([CueId::Footstep])
                    || result.progress != SenderProgress::AwaitCue(CueId::Footstep)
                    || result.presentation.as_ref().is_none_or(|(time, request)| {
                        time.0 != at || *request != Request::Cue(CueId::Footstep)
                    })
                    || binding.cue != CueId::Footstep
                    || binding.port.output_class() != Ok(2)
                {
                    return Err(Fault::Receipt);
                }
                vec![data::timing::DeviceWrite {
                    port: binding.port,
                    value: binding.value,
                    effect: data::timing::Publication::Downstream,
                }]
            }
        };
        Ok(Qualified {
            body: self.body.bind(&writes).map_err(|_| Fault::Profile)?,
            tail: self.tail.bind(&[]).map_err(|_| Fault::Profile)?,
            publication,
            completion,
        })
    }
}

/// Caller admission work, separate from the three actual service bodies and tails.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Admission(data::timing::Template);

impl Admission {
    /// Import the complete bounded admission envelope.
    /// # Errors
    /// Unexpected effects or a different intrinsic physical extent.
    pub fn new(template: data::timing::Template) -> Result<Self, Fault> {
        if milestones(&template)? != (276, None) {
            return Err(Fault::Profile);
        }
        Ok(Self(template))
    }

    /// Qualify current owners without freezing their later service queues.
    /// # Errors
    /// Invalid borrowed regime, queue representation or current service mode.
    pub fn qualify(
        &self,
        senders: &Senders,
        queues: &Queues,
        current: Current,
    ) -> Result<data::timing::Profile, Fault> {
        current.validate()?;
        senders.validate().map_err(|_| Fault::Receipt)?;
        queues.validate(senders)?;
        self.qualify_groups(senders, &SenderProgress::Ready)
    }

    pub(super) fn qualify_retry(
        &self,
        senders: &Senders,
        queues: &Queues,
        current: Current,
    ) -> Result<Profile, Fault> {
        current.validate()?;
        senders.validate().map_err(|_| Fault::Receipt)?;
        queues.validate(senders)?;
        if !senders.contact.0.queue.is_empty() || queues.contact.positions() != [1, 1] {
            return Err(Fault::Receipt);
        }
        self.qualify_groups(senders, &SenderProgress::AwaitCue(CueId::Footstep))
    }

    pub(super) fn qualify_continuation(
        &self,
        senders: &Senders,
        queues: &Queues,
        current: Current,
    ) -> Result<Profile, Fault> {
        current.validate()?;
        senders.validate().map_err(|_| Fault::Receipt)?;
        queues.validate(senders)?;
        let contact = &senders.contact.0;
        if !contact.queue.is_empty()
            || !matches!(
                contact.progress,
                SenderProgress::ClearDelay(_) | SenderProgress::AwaitClear | SenderProgress::Ready
            )
        {
            return Err(Fault::Receipt);
        }
        self.qualify_groups(senders, &contact.progress)
    }

    fn qualify_groups(
        &self,
        senders: &Senders,
        contact: &SenderProgress,
    ) -> Result<Profile, Fault> {
        for group in [SoundGroup::Action, SoundGroup::Ambient, SoundGroup::Contact] {
            let sender = sender(senders, group);
            let required = if group == SoundGroup::Contact {
                contact
            } else {
                &SenderProgress::Ready
            };
            if &sender.progress != required || sender.pending.is_some() {
                return Err(Fault::Receipt);
            }
        }
        self.0.bind(&[]).map_err(|_| Fault::Profile)
    }
}

/// Physical positions only; the actual cue FIFO belongs to the clean sender.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct QueueWindow {
    head: u8,
    tail: u8,
}

impl QueueWindow {
    /// Import independently qualified current positions once at the frame origin.
    /// # Errors
    /// Positions outside the sixteen-slot physical representation.
    pub const fn new(head: u8, tail: u8) -> Result<Self, Fault> {
        if head >= 16 || tail >= 16 {
            return Err(Fault::Queue);
        }
        Ok(Self { head, tail })
    }

    /// Current physical positions, used only to qualify a source envelope.
    #[must_use]
    pub const fn positions(self) -> [u8; 2] {
        [self.head, self.tail]
    }

    fn validate(self, queue: &VecDeque<CueId>) -> Result<(), Fault> {
        if self.head >= 16
            || self.tail >= 16
            || usize::from(self.tail.wrapping_sub(self.head) % 16) != queue.len()
        {
            return Err(Fault::Queue);
        }
        Ok(())
    }
}

/// Origin-derived positions for the three actual sender owners.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Queues {
    action: QueueWindow,
    ambient: QueueWindow,
    contact: QueueWindow,
}

fn sender(senders: &Senders, group: SoundGroup) -> &SenderState {
    match group {
        SoundGroup::Action => &senders.action.0,
        SoundGroup::Ambient => &senders.ambient.0,
        SoundGroup::Contact => &senders.contact.0,
    }
}

impl Queues {
    /// Import positions alongside, rather than in place of, actual clean queues.
    /// # Errors
    /// A physical window disagrees with its actual current sender FIFO.
    pub fn new(
        action: QueueWindow,
        ambient: QueueWindow,
        contact: QueueWindow,
        senders: &Senders,
    ) -> Result<Self, Fault> {
        let queues = Self {
            action,
            ambient,
            contact,
        };
        queues.validate(senders)?;
        Ok(queues)
    }

    /// Check restored positions against all actual current FIFOs.
    /// # Errors
    /// An invalid window or lost queue transition.
    pub fn validate(&self, senders: &Senders) -> Result<(), Fault> {
        for group in [SoundGroup::Action, SoundGroup::Ambient, SoundGroup::Contact] {
            self.window(group).validate(&sender(senders, group).queue)?;
        }
        Ok(())
    }

    /// The current physical window, never reconstructed from queue length.
    #[must_use]
    pub const fn window(&self, group: SoundGroup) -> QueueWindow {
        match group {
            SoundGroup::Action => self.action,
            SoundGroup::Ambient => self.ambient,
            SoundGroup::Contact => self.contact,
        }
    }

    const fn window_mut(&mut self, group: SoundGroup) -> &mut QueueWindow {
        match group {
            SoundGroup::Action => &mut self.action,
            SoundGroup::Ambient => &mut self.ambient,
            SoundGroup::Contact => &mut self.contact,
        }
    }

    /// Track actual append results on the same candidate as the producer commit.
    /// # Errors
    /// A producer replaced existing cues, shortened the FIFO or changed progress.
    pub fn offered(
        &mut self,
        group: SoundGroup,
        before: &SenderState,
        after: &SenderState,
    ) -> Result<(), Fault> {
        let mut window = self.window(group);
        window.validate(&before.queue)?;
        if before.progress != after.progress
            || before.reply != after.reply
            || before.now != after.now
            || before.pending != after.pending
            || after.queue.len() < before.queue.len()
            || !before
                .queue
                .iter()
                .eq(after.queue.iter().take(before.queue.len()))
        {
            return Err(Fault::Transition);
        }
        let added = after
            .queue
            .len()
            .checked_sub(before.queue.len())
            .ok_or(Fault::Transition)?;
        let added = u8::try_from(added).map_err(|_| Fault::Queue)?;
        window.tail = window.tail.wrapping_add(added) % 16;
        window.validate(&after.queue)?;
        *self.window_mut(group) = window;
        Ok(())
    }

    /// Qualify the bounded actual Footstep append after the completed producer offers it.
    /// Call on the same private candidate as the actual Appearance sound delivery.
    /// # Errors
    /// Suppressed offer, wrong native window or a result other than the one-cue enqueue.
    pub fn footstep(
        &mut self,
        before: &SenderState,
        after: &SenderState,
        permission: audio_transport::OfferPermission,
    ) -> Result<(), Fault> {
        if permission != audio_transport::OfferPermission::Enabled
            || self.contact.positions() != [0, 0]
            || !before.queue.is_empty()
            || before.progress != SenderProgress::Ready
            || after.queue.iter().copied().ne([CueId::Footstep])
        {
            return Err(Fault::Receipt);
        }
        self.offered(SoundGroup::Contact, before, after)
    }

    /// Track the actual completed retained result, without duplicating its queue.
    /// # Errors
    /// Missing result, replaced output or a transition other than prefix removal.
    pub fn completed(&mut self, receipt: &Receipt, after: &SenderState) -> Result<(), Fault> {
        let mut window = self.window(receipt.group);
        window.validate(&receipt.queue)?;
        let result = receipt.chosen.result.as_ref().ok_or(Fault::Transition)?;
        let removed = receipt
            .queue
            .len()
            .checked_sub(result.queue.len())
            .ok_or(Fault::Transition)?;
        if after.now != result.completion {
            return Err(Fault::Transition);
        }
        if after.pending.is_some()
            || after.progress != result.progress
            || after.queue != result.queue
            || after.reply != receipt.reply
            || !receipt.queue.iter().skip(removed).eq(result.queue.iter())
        {
            return Err(Fault::Transition);
        }
        let removed = u8::try_from(removed).map_err(|_| Fault::Queue)?;
        window.head = window.head.wrapping_add(removed) % 16;
        window.validate(&after.queue)?;
        *self.window_mut(receipt.group) = window;
        Ok(())
    }
}
