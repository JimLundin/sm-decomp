//! Idle protection and actor-list closing, followed by a distinct held-history write.

use super::{Caller, Content, Dependency, State, sender_caller::Cursor};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::Canonical;
use data::timing::Profile;

pub(super) const HURT: timing::WorkId = timing::WorkId(1424);
pub(super) const FINAL_HISTORY: timing::WorkId = timing::WorkId(1425);

/// Immutable quiet profiles for actual tail opportunities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    /// Complete quiet hurt/timer and actor-list return profile.
    pub hurt: Profile,
    /// Complete final debug/held-history profile.
    pub history: Profile,
    /// Independently checked current debug shortcut and spare display inactivity.
    pub diagnostics_inactive: bool,
}

/// Current independent actor-list cursor states, retired only at actual hurt/list return.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Current {
    /// Current invincibility, knockback and projectile timers, qualified independently.
    pub timers: [u16; 3],
    /// Both complete current actor cursors.
    pub cursors: [u16; 2],
}

/// Progress of the two distinct tail services on either side of Sender/OAM.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// Hurt timer and list close work has not started.
    #[default]
    Pending,
    /// Whole hurt/list physical envelope awaits return.
    Closing,
    /// Both lists closed; Sender and OAM can proceed.
    Closed,
    /// Actual final history remains in progress.
    History,
    /// Actual final held history returned.
    Complete,
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::FinalFrameHistory))
}

impl State {
    pub(super) fn advance_frame_tail(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let contract = content.frame_tail.as_ref().ok_or_else(missing)?;
        let (work, profile, next) = match self.frame_tail {
            Stage::Pending => {
                let current = self.frame_tail_current.as_ref().ok_or_else(missing)?;
                let Caller::PlayerContinuation { remaining, .. } = &self.caller else {
                    return Err(missing());
                };
                if current.timers != [0; 3] || remaining.0.actors.is_none() {
                    return Err(missing());
                }
                (HURT, &contract.hurt, Stage::Closing)
            }
            Stage::Closed
                if matches!(self.sender_progress, Cursor::Done)
                    && matches!(
                        self.return_progress,
                        super::frame_return::Stage::Ready { .. }
                    )
                    && self
                        .frame_history
                        .as_ref()
                        .is_some_and(super::frame_history::History::drawn)
                    && contract.diagnostics_inactive =>
            {
                (FINAL_HISTORY, &contract.history, Stage::History)
            }
            Stage::Closing | Stage::History => {
                let (work, profile) = if self.frame_tail == Stage::Closing {
                    (HURT, &contract.hurt)
                } else {
                    (FINAL_HISTORY, &contract.history)
                };
                let mut active = self
                    .timing
                    .work
                    .iter()
                    .filter(|held| held.kind == timing::WorkKind::Semantic(work));
                if active.next().is_some_and(|held| &held.profile == profile)
                    && active.next().is_none()
                {
                    return Ok(());
                }
                return Err(missing());
            }
            Stage::Closed | Stage::Complete => return Err(missing()),
        };
        self.timing
            .begin(work, profile.clone())
            .map_err(ProductionFault::FrameHardware)?;
        self.frame_tail = next;
        Ok(())
    }

    pub(super) fn complete_frame_tail(
        &mut self,
        work: timing::WorkId,
    ) -> Result<(), ProductionFault> {
        match (&self.frame_tail, work) {
            (Stage::Closing, HURT) => {
                let current = self.frame_tail_current.as_mut().ok_or_else(missing)?;
                if current.timers != [0; 3] {
                    return Err(missing());
                }
                current.cursors = [u16::MAX; 2];
                self.frame_tail = Stage::Closed;
            }
            (Stage::History, FINAL_HISTORY) => {
                if !self
                    .frame_history
                    .as_mut()
                    .is_some_and(super::frame_history::History::finish_held)
                {
                    return Err(missing());
                }
                self.frame_tail = Stage::Complete;
            }
            _ => return Err(missing()),
        }
        Ok(())
    }
}
