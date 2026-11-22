//! Drive the retained request, wait and native return through the actual hardware owner.

use super::{Progress, suffix_caller};
use crate::production::timing;
use data::timing::{
    ContentRegion, Interrupt, MasterTicks, Profile, Sample, Template, TemplateEffect,
};

/// Whole shared prefix and both quiet controller tail envelopes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControllerTail {
    prefix: Template,
    reset: Profile,
    held_repeat: Profile,
}

impl ControllerTail {
    /// Split the checked common body immediately after the actual `ControllerHigh` sample.
    ///
    /// # Errors
    /// Rejects a missing sample, unexpected split or effects in either tail.
    pub fn new(
        whole: &Template,
        prefix_exposures: u64,
        held_repeat: &Template,
    ) -> Result<Self, suffix_caller::Fault> {
        use suffix_caller::Fault;
        let last_sample = whole
            .runs
            .iter()
            .position(|run| {
                run.count == 1 && run.effect == TemplateEffect::Sample(Sample::ControllerHigh)
            })
            .ok_or(Fault::Profile)?;
        let runs = whole.runs.get(..=last_sample).ok_or(Fault::Profile)?;
        if runs.iter().map(|run| u64::from(run.count)).sum::<u64>() != prefix_exposures {
            return Err(Fault::Profile);
        }
        let bind = |template: &Template| {
            if template
                .runs
                .iter()
                .any(|run| run.effect != TemplateEffect::None)
            {
                return Err(Fault::Profile);
            }
            template.bind(&[]).map_err(|_| Fault::Profile)
        };
        Ok(Self {
            prefix: Template {
                runs: runs.to_vec(),
            },
            reset: bind(&Template {
                runs: whole
                    .runs
                    .get(last_sample.saturating_add(1)..)
                    .ok_or(Fault::Profile)?
                    .to_vec(),
            })?,
            held_repeat: bind(held_repeat)?,
        })
    }

    pub(super) fn at_sample(
        &self,
        state: &super::State,
        held: u16,
    ) -> Result<&Profile, suffix_caller::Fault> {
        if held == 0 {
            return Err(suffix_caller::Fault::Profile);
        }
        let repeats = state
            .frame_history
            .as_ref()
            .and_then(|history| history.controller_tail_repeats(held))
            .ok_or(suffix_caller::Fault::MissingProducer)?;
        Ok(if repeats {
            &self.held_repeat
        } else {
            &self.reset
        })
    }
}

/// Actual selected operations' immutable full envelopes and current publication recipes.
pub(super) struct Work<'a> {
    pub preparation: &'a Template,
    pub nmi: &'a Template,
    pub controller_tail: Option<&'a ControllerTail>,
    pub requires_reset_sample: bool,
    pub prefix_and_poll: &'a Template,
    pub recipes: &'a [suffix_caller::PublicationRecipe],
}

impl suffix_caller::Suffix {
    pub(super) fn advance_native_return(
        &mut self,
        owners: &suffix_caller::NmiOwners<'_>,
        work: &Work<'_>,
        timing: &mut timing::State,
        regions: &[ContentRegion],
        budget: MasterTicks,
    ) -> Result<(Progress, Vec<timing::Event>), suffix_caller::Fault> {
        use suffix_caller::{Fault, PreparedVideo, QualifiedNmiBody, ReadyOwners};
        let ready = || ReadyOwners {
            state: owners.ready.state,
            prepared: owners.ready.prepared,
            remaining: owners.ready.remaining,
            builder: owners.ready.builder,
        };
        match self {
            Self::Unread => {
                self.begin_preparation(PreparedVideo::qualify(ready())?, work.preparation, timing)?;
            }
            Self::Requested { .. } => self.begin_wait(timing)?,
            Self::Preparing { ready: pending }
            | Self::Waiting { ready: pending }
            | Self::Entering { ready: pending }
            | Self::Video { ready: pending, .. }
            | Self::ControllerTail { ready: pending, .. } => pending.validate(ready())?,
            Self::Acknowledged { .. } => return Err(Fault::Order),
        }
        let active_video = matches!(self, Self::Video { .. });
        let (progress, mut events) = timing.step(budget, regions).map_err(Fault::Hardware)?;
        if active_video {
            self.observe(&events)?;
            self.validate_reset_sample(work, owners.ready.state)?;
        }
        match progress {
            timing::Progress::Advanced => {}
            timing::Progress::Completed(identity) if identity == suffix_caller::PREPARATION => {
                self.complete_preparation(identity, ready())?;
            }
            timing::Progress::Interrupt(Interrupt::Nmi) => {
                let prefix = suffix_caller::bind_idle_prefix_and_poll(
                    work.prefix_and_poll,
                    work.recipes,
                    owners,
                )?;
                let admission = suffix_caller::preview_idle_poll(timing, regions, &prefix)?;
                events.extend(self.enter_nmi(progress, timing, &admission, regions, &prefix)?);
            }
            timing::Progress::HandlerEntered(identity) if identity == suffix_caller::NMI_ENTRY => {
                let template = work.controller_tail.map_or(work.nmi, |tail| &tail.prefix);
                let mut body = QualifiedNmiBody::qualify(template, work.recipes, owners)?;
                if work.controller_tail.is_some() {
                    body.completion = timing::HandlerCompletion::Continue;
                }
                self.begin_nmi(progress, ready(), body, timing)?;
            }
            timing::Progress::HandlerPortionCompleted(suffix_caller::NMI_VIDEO) => {
                let Self::Video {
                    ready,
                    body,
                    sampled,
                    controller,
                } = self
                else {
                    return Err(Fault::Order);
                };
                if body.completion != timing::HandlerCompletion::Continue
                    || usize::try_from(*sampled) != Ok(body.samples.len())
                {
                    return Err(Fault::Completion);
                }
                let tail = work.controller_tail.ok_or(Fault::Profile)?;
                let held = controller.word().ok_or(Fault::Completion)?;
                let profile = tail.at_sample(owners.ready.state, held)?;
                timing
                    .begin_handler_portion(
                        suffix_caller::NMI_CONTROLLER_TAIL,
                        profile.clone(),
                        timing::HandlerCompletion::NativeReturn,
                    )
                    .map_err(Fault::Hardware)?;
                *self = Self::ControllerTail {
                    ready: ready.clone(),
                    controller: controller.clone(),
                };
            }
            timing::Progress::HandlerReturned(identity) if identity == suffix_caller::NMI_ENTRY => {
                self.complete_nmi(progress)?;
                return Ok((Progress::Completed, events));
            }
            timing::Progress::Completed(_)
            | timing::Progress::Interrupt(_)
            | timing::Progress::HandlerEntered(_)
            | timing::Progress::HandlerReturned(_)
            | timing::Progress::HandlerPortionCompleted(_)
            | timing::Progress::HandlerNeedsWork(_)
            | timing::Progress::Idle => return Err(Fault::Completion),
        }
        Ok((Progress::Advanced, events))
    }

    fn validate_reset_sample(
        &self,
        work: &Work<'_>,
        state: &super::State,
    ) -> Result<(), suffix_caller::Fault> {
        if work.requires_reset_sample
            && let Self::Video { controller, .. } = self
            && let Some(held) = controller.word()
            && (held == 0
                || state
                    .frame_history
                    .as_ref()
                    .and_then(|history| history.controller_tail_repeats(held))
                    != Some(false))
        {
            return Err(suffix_caller::Fault::Profile);
        }
        Ok(())
    }
}
