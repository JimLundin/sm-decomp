//! Drive the retained request, wait and native return through the actual hardware owner.

use super::{Progress, suffix_caller};
use crate::production::timing;
use data::timing::{ContentRegion, Interrupt, MasterTicks, Template};

/// Actual selected operations' immutable full envelopes and current publication recipes.
pub(super) struct Work<'a> {
    pub preparation: &'a Template,
    pub nmi: &'a Template,
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
            | Self::Video { ready: pending, .. } => pending.validate(ready())?,
            Self::Acknowledged { .. } => return Err(Fault::Order),
        }
        let active_video = matches!(self, Self::Video { .. });
        let (progress, mut events) = timing.step(budget, regions).map_err(Fault::Hardware)?;
        if active_video {
            self.observe(&events)?;
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
                let body = QualifiedNmiBody::qualify(work.nmi, work.recipes, owners)?;
                self.begin_nmi(progress, ready(), body, timing)?;
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
}
