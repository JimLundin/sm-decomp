//! Current Contact clear-delay and reply work, with CPU and incoming latches separate.

use super::{
    sender_contract::{Fault, Qualified, Queues, milestones},
    sender_stage::{CueBinding, Receipt},
};
use audio_transport::{CueId, Request, SenderProgress, SoundGroup};
use data::timing::{DeviceWrite, MasterTicks, Profile, Publication, Template};

/// Checked bodies for the empty Contact queue after the cue has matched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContactContinuation {
    delay_hold: Profile,
    delay_clear: Template,
    pub(super) prefix: Profile,
    retry: Template,
    matched: Profile,
    empty: Profile,
    clear_reply_delay: MasterTicks,
    later_reply_delay: Option<MasterTicks>,
}

impl ContactContinuation {
    /// Admit the current delay variants, shared read and result variants, and empty Ready.
    ///
    /// # Errors
    /// Required milestones, delayed external arrival or effect-free work differ.
    pub fn new(
        [delay_hold, delay_clear]: [Template; 2],
        [prefix, retry, matched]: [Template; 3],
        empty: &Template,
        clear_reply_delay: MasterTicks,
    ) -> Result<Self, Fault> {
        let quiet = |template: &Template, duration| {
            if milestones(template)? != (duration, None) {
                return Err(Fault::Profile);
            }
            template.bind(&[]).map_err(|_| Fault::Profile)
        };
        if clear_reply_delay.0 == 0
            || milestones(&delay_clear)? != (218, Some(100))
            || milestones(&retry)? != (100, Some(42))
        {
            return Err(Fault::Profile);
        }
        Ok(Self {
            delay_hold: quiet(&delay_hold, 116)?,
            delay_clear,
            prefix: quiet(&prefix, 62)?,
            retry,
            matched: quiet(&matched, 190)?,
            empty: quiet(empty, 122)?,
            clear_reply_delay,
            later_reply_delay: None,
        })
    }

    /// Admit the independent one-shot Footstep reply after the next actual retry.
    /// # Errors
    /// A response delivered at its CPU trigger would bypass the independently observed delay.
    pub const fn admit_later_reply(&mut self, delay: MasterTicks) -> Result<(), Fault> {
        if delay.0 == 0 {
            return Err(Fault::Profile);
        }
        self.later_reply_delay = Some(delay);
        Ok(())
    }

    pub(super) fn later_reply_at(&self, retry: MasterTicks, now: MasterTicks) -> Option<Request> {
        let arrival = retry.0.checked_add(self.later_reply_delay?.0)?;
        Some(if arrival <= now.0 {
            Request::Cue(CueId::Footstep)
        } else {
            Request::Absent
        })
    }

    pub(super) fn reply_at(&self, retry: Option<MasterTicks>, now: MasterTicks) -> Request {
        if retry
            .and_then(|retry| retry.0.checked_add(self.clear_reply_delay.0))
            .is_some_and(|ready| ready <= now.0)
        {
            Request::Absent
        } else {
            Request::Cue(CueId::Footstep)
        }
    }

    pub(super) fn qualify(
        &self,
        receipt: &Receipt,
        queues: &Queues,
        binding: CueBinding,
        tail: &Template,
    ) -> Result<Qualified, Fault> {
        let chosen = receipt.chosen.result.as_ref().ok_or(Fault::Receipt)?;
        let [head, end] = queues.window(SoundGroup::Contact).positions();
        if receipt.group != SoundGroup::Contact
            || receipt.chosen.observation.is_some()
            || !receipt.queue.is_empty()
            || !chosen.queue.is_empty()
            || head != end
            || binding.cue != CueId::Footstep
            || binding.port.output_class() != Ok(2)
        {
            return Err(Fault::Receipt);
        }
        let publish_zero = |template: &Template| {
            template
                .bind(&[DeviceWrite {
                    port: binding.port,
                    value: 0,
                    effect: Publication::Downstream,
                }])
                .map_err(|_| Fault::Profile)
        };
        let (body, next, publication, completion) = match receipt.before {
            SenderProgress::ClearDelay(turns) if turns.get() == 2 => {
                let SenderProgress::ClearDelay(left) = chosen.progress else {
                    return Err(Fault::Receipt);
                };
                if left.get() != 1 {
                    return Err(Fault::Receipt);
                }
                (self.delay_hold.clone(), chosen.progress.clone(), None, 116)
            }
            SenderProgress::ClearDelay(turns) if turns.get() == 1 => (
                publish_zero(&self.delay_clear)?,
                SenderProgress::AwaitClear,
                Some(100),
                218,
            ),
            SenderProgress::AwaitClear if receipt.reply == Request::Cue(CueId::Footstep) => (
                publish_zero(&self.retry)?,
                SenderProgress::AwaitClear,
                Some(104),
                162,
            ),
            SenderProgress::AwaitClear if receipt.reply == Request::Absent => {
                (self.matched.clone(), SenderProgress::Ready, None, 252)
            }
            SenderProgress::Ready if receipt.reply == Request::Absent => {
                (self.empty.clone(), SenderProgress::Ready, None, 122)
            }
            SenderProgress::Ready
            | SenderProgress::AwaitCue(_)
            | SenderProgress::ClearDelay(_)
            | SenderProgress::AwaitClear
            | SenderProgress::ResetDelay(_) => return Err(Fault::Receipt),
        };
        let origin = &receipt.chosen.origin.0;
        let end = std::ops::Add::add(origin, completion);
        let at = publication.map(|edge| std::ops::Add::add(origin, edge));
        let observed = chosen
            .presentation
            .as_ref()
            .map(|(time, request)| (&time.0, *request));
        if chosen.progress != next
            || chosen.completion.0 != end
            || observed != at.as_ref().map(|at| (at, Request::Absent))
        {
            return Err(Fault::Receipt);
        }
        Ok(Qualified {
            body,
            tail: tail.bind(&[]).map_err(|_| Fault::Profile)?,
            publication,
            completion,
        })
    }
}
