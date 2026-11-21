//! The opening physical publication acknowledgment and quiet Music work.

use super::borrowed_receipts::{
    FrameContinuationAfterNmi, MusicCompletionReceipt, MusicCountdown, MusicQueue, MusicRequest,
    MusicServiceReceipt,
};
use super::{Caller, Dependency, Prelude, PreludeSource, Senders};
use crate::production::timing::{self, WorkId};
use crate::production::{ProductionFault, Unsupported};
use canonical::Canonical;
use data::timing::{MasterTicks, Template, TemplateEffect};

pub(super) const ACKNOWLEDGMENT: WorkId = WorkId(1);
pub(super) const MUSIC: WorkId = WorkId(2);

const fn music_missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::MusicPreparation))
}

pub(super) fn begin_acknowledgment(
    caller: &mut Caller,
    timing: &mut timing::State,
    template: &Template,
) -> Result<(), ProductionFault> {
    let Caller::AwaitingAcknowledgment { prelude, receipt } = caller else {
        return Err(ProductionFault::WrongScope);
    };
    let receipt = receipt.ok_or(ProductionFault::Unsupported(Unsupported::Frame(
        Dependency::FramePublicationAcknowledgment,
    )))?;
    let profile = template
        .bind(&[])
        .map_err(|_| ProductionFault::FrameHardware(timing::Fault::InvalidProfile))?;
    timing
        .begin(ACKNOWLEDGMENT, profile)
        .map_err(ProductionFault::FrameHardware)?;
    *caller = Caller::Acknowledgment {
        prelude: prelude.clone(),
        receipt,
    };
    Ok(())
}

pub(super) fn complete_acknowledgment(caller: &mut Caller) -> Result<(), ProductionFault> {
    let Caller::Acknowledgment { prelude, receipt } = caller else {
        return Err(ProductionFault::WrongScope);
    };
    *caller = Caller::Music {
        prelude: prelude.clone(),
        publication: *receipt,
    };
    Ok(())
}

fn music_result(
    current: Option<MusicServiceReceipt>,
    prelude: &PreludeSource,
) -> Result<MusicCompletionReceipt, ProductionFault> {
    let current = current.ok_or_else(music_missing)?;
    let result = match (current.countdown, current.queue) {
        (MusicCountdown::Ready, MusicQueue::Empty) => MusicCompletionReceipt {
            dependency: current,
            terminal: current,
            request: MusicRequest::None,
        },
    };
    if let PreludeSource::Origin(source) = prelude
        && source.music_completion.as_ref() != Some(&result)
    {
        return Err(music_missing());
    }
    Ok(result)
}

pub(super) fn begin_music(
    caller: &mut Caller,
    timing: &mut timing::State,
    current: Option<MusicServiceReceipt>,
    template: &Template,
) -> Result<(), ProductionFault> {
    let Caller::Music {
        prelude,
        publication,
    } = caller
    else {
        return Err(ProductionFault::WrongScope);
    };
    music_result(current, prelude)?;
    if template
        .runs
        .iter()
        .any(|run| run.effect != TemplateEffect::None)
    {
        return Err(music_missing());
    }
    let profile = template
        .bind(&[])
        .map_err(|_| ProductionFault::FrameHardware(timing::Fault::InvalidProfile))?;
    timing
        .begin(MUSIC, profile)
        .map_err(ProductionFault::FrameHardware)?;
    *caller = Caller::MusicRunning {
        prelude: prelude.clone(),
        publication: *publication,
    };
    Ok(())
}

pub(super) fn complete_music(
    caller: &mut Caller,
    current: &mut Option<MusicServiceReceipt>,
) -> Result<(), ProductionFault> {
    let Caller::MusicRunning {
        prelude,
        publication,
    } = caller
    else {
        return Err(ProductionFault::WrongScope);
    };
    let music = music_result(*current, prelude)?;
    let mut prelude = prelude.clone();
    if let PreludeSource::Origin(source) = &mut prelude {
        source.music_completion = None;
    }
    *current = Some(music.terminal);
    *caller = Caller::Haze {
        prelude,
        publication: *publication,
        music,
    };
    Ok(())
}

/// An admitted opening opportunity before Haze and the later Frame owners are available.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Opening {
    caller: Caller,
    timing: timing::State,
    acknowledgment_ended_at: Option<MasterTicks>,
    music: Option<MusicServiceReceipt>,
    senders: Senders,
}

/// Checked exposures of the two reached owners.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profiles {
    /// Actual preceding publication acknowledgment work.
    pub acknowledgment: Template,
    /// Actual effect-free quiet Music work.
    pub music: Template,
}

fn validate_senders(senders: &Senders) -> Result<(), ProductionFault> {
    use audio_transport::{CpuMasterTicks, CueId, Request, SenderProgress, SoundGroup};

    if senders.validate().is_err() {
        return Err(ProductionFault::WrongScope);
    }
    for group in [SoundGroup::Action, SoundGroup::Ambient, SoundGroup::Contact] {
        let sender = senders.get(group);
        let supported = matches!(
            (&sender.progress, sender.reply),
            (SenderProgress::Ready, Request::Absent)
        ) || (group == SoundGroup::Contact
            && matches!(
                (&sender.progress, sender.reply),
                (SenderProgress::AwaitClear, Request::Cue(CueId::Footstep))
            ));
        if !supported
            || sender.pending.is_some()
            || sender.now != CpuMasterTicks(0)
            || !sender.queue.is_empty()
        {
            return Err(ProductionFault::WrongScope);
        }
    }
    Ok(())
}

fn bare_template(template: &Template) -> Result<(), ProductionFault> {
    if template
        .runs
        .iter()
        .any(|run| run.effect != TemplateEffect::None)
    {
        return Err(ProductionFault::FrameHardware(
            timing::Fault::InvalidProfile,
        ));
    }
    template
        .validate()
        .map_err(|_| ProductionFault::FrameHardware(timing::Fault::InvalidProfile))
}

impl Opening {
    /// Admit actual hardware and current owners without choosing a future preparation result.
    ///
    /// # Errors
    /// Hardware or sender state is invalid, is already charging, or does not share the origin.
    pub fn new(
        timing: timing::State,
        senders: Senders,
        receipt: FrameContinuationAfterNmi,
        current: MusicServiceReceipt,
        music: MusicCompletionReceipt,
    ) -> Result<Self, ProductionFault> {
        timing.validate().map_err(ProductionFault::FrameHardware)?;
        validate_senders(&senders)?;
        if timing.hardware.now != MasterTicks(0) || !timing.work.is_empty() {
            return Err(ProductionFault::WrongScope);
        }
        let caller = Caller::AwaitingAcknowledgment {
            prelude: PreludeSource::Origin(Box::new(Prelude {
                music_completion: Some(music),
                gameplay_completion: None,
                actor_completion: None,
                debug_completion: None,
                palette_completion: None,
                player_entry_current: None,
                player_entry_offers: None,
                player_continuation_current: None,
                player_continuation_offers: None,
            })),
            receipt: Some(receipt),
        };
        Ok(Self {
            caller,
            timing,
            acknowledgment_ended_at: None,
            music: Some(current),
            senders,
        })
    }

    /// Current canonical hardware, including charged run and refresh progress.
    #[must_use]
    pub const fn timing(&self) -> &timing::State {
        &self.timing
    }

    /// Current semantic caller phase.
    #[must_use]
    pub const fn caller(&self) -> &Caller {
        &self.caller
    }

    fn validate_elapsed(&self, profiles: &Profiles) -> Result<(), ProductionFault> {
        let now = self.timing.hardware.now.0;
        let intrinsic = |template: &Template| {
            template
                .runs
                .iter()
                .try_fold(0_u64, |total, run| {
                    total.checked_add(
                        u64::from(run.count).saturating_mul(u64::from(run.cycle.ticks())),
                    )
                })
                .ok_or(ProductionFault::FrameHardware(timing::Fault::TimeOverflow))
        };
        if matches!(
            self.caller,
            Caller::AwaitingAcknowledgment { .. } | Caller::Acknowledgment { .. }
        ) {
            if self.acknowledgment_ended_at.is_some() {
                return Err(ProductionFault::WrongScope);
            }
            if let Some(work) = self.timing.work.first()
                && now < work.cursor.intrinsic.0
            {
                return Err(ProductionFault::WrongScope);
            }
            return Ok(());
        }

        let acknowledgment = self
            .acknowledgment_ended_at
            .ok_or(ProductionFault::WrongScope)?
            .0;
        if acknowledgment < intrinsic(&profiles.acknowledgment)? {
            return Err(ProductionFault::WrongScope);
        }
        let elapsed = now
            .checked_sub(acknowledgment)
            .ok_or(ProductionFault::WrongScope)?;
        let valid = if matches!(self.caller, Caller::Haze { .. }) {
            elapsed >= intrinsic(&profiles.music)?
        } else if let Some(work) = self.timing.work.first() {
            elapsed >= work.cursor.intrinsic.0
        } else {
            elapsed == 0
        };
        if !valid {
            return Err(ProductionFault::WrongScope);
        }
        Ok(())
    }

    pub(in crate::production) fn validate(
        &self,
        profiles: &Profiles,
    ) -> Result<(), ProductionFault> {
        validate_senders(&self.senders)?;
        bare_template(&profiles.acknowledgment)?;
        bare_template(&profiles.music)?;
        self.timing
            .validate()
            .map_err(ProductionFault::FrameHardware)?;
        if self.timing.interrupt_boundary.is_some() || self.timing.pending_handler.is_some() {
            return Err(ProductionFault::WrongScope);
        }
        let (Caller::AwaitingAcknowledgment {
            prelude,
            receipt: Some(_),
        }
        | Caller::Acknowledgment { prelude, .. }
        | Caller::Music { prelude, .. }
        | Caller::MusicRunning { prelude, .. }
        | Caller::Haze { prelude, .. }) = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let origin = prelude.origin().ok_or(ProductionFault::WrongScope)?;
        if origin.gameplay_completion.is_some()
            || origin.actor_completion.is_some()
            || origin.debug_completion.is_some()
            || origin.palette_completion.is_some()
            || origin.player_entry_current.is_some()
            || origin.player_entry_offers.is_some()
            || origin.player_continuation_current.is_some()
            || origin.player_continuation_offers.is_some()
        {
            return Err(ProductionFault::WrongScope);
        }
        if let Caller::Haze { music, .. } = &self.caller {
            if origin.music_completion.is_some() || self.music != Some(music.terminal) {
                return Err(music_missing());
            }
        } else {
            music_result(self.music, prelude)?;
        }
        if matches!(
            self.caller,
            Caller::Acknowledgment { .. } | Caller::MusicRunning { .. }
        ) {
            self.validate_charge(self.current_template(profiles)?)?;
        } else if !self.timing.work.is_empty()
            || (matches!(self.caller, Caller::AwaitingAcknowledgment { .. })
                && self.timing.hardware.now != MasterTicks(0))
        {
            return Err(ProductionFault::WrongScope);
        }
        self.validate_elapsed(profiles)
    }

    pub(in crate::production) fn advance(
        &mut self,
        profiles: &Profiles,
        budget: MasterTicks,
    ) -> Result<(timing::Progress, Vec<timing::Event>), ProductionFault> {
        let template = self.current_template(profiles)?;
        if matches!(self.caller, Caller::AwaitingAcknowledgment { .. }) {
            begin_acknowledgment(&mut self.caller, &mut self.timing, template)?;
        } else if matches!(self.caller, Caller::Music { .. }) {
            begin_music(&mut self.caller, &mut self.timing, self.music, template)?;
        }
        let (progress, events) = self
            .timing
            .step(budget, &[])
            .map_err(ProductionFault::FrameHardware)?;
        if progress == timing::Progress::Completed(ACKNOWLEDGMENT) {
            complete_acknowledgment(&mut self.caller)?;
            self.acknowledgment_ended_at = Some(self.timing.hardware.now);
        } else if progress == timing::Progress::Completed(MUSIC) {
            complete_music(&mut self.caller, &mut self.music)?;
        } else if progress != timing::Progress::Advanced {
            return Err(ProductionFault::FrameHardware(
                timing::Fault::InterruptBoundary,
            ));
        }
        Ok((progress, events))
    }

    const fn current_template<'a>(
        &self,
        profiles: &'a Profiles,
    ) -> Result<&'a Template, ProductionFault> {
        if matches!(
            self.caller,
            Caller::AwaitingAcknowledgment { .. } | Caller::Acknowledgment { .. }
        ) {
            Ok(&profiles.acknowledgment)
        } else if matches!(
            self.caller,
            Caller::Music { .. } | Caller::MusicRunning { .. }
        ) {
            Ok(&profiles.music)
        } else if matches!(self.caller, Caller::Haze { .. }) {
            Err(super::haze::missing())
        } else {
            Err(ProductionFault::WrongScope)
        }
    }

    fn validate_charge(&self, template: &Template) -> Result<(), ProductionFault> {
        let expected = if matches!(self.caller, Caller::Acknowledgment { .. }) {
            ACKNOWLEDGMENT
        } else if matches!(self.caller, Caller::MusicRunning { .. }) {
            MUSIC
        } else {
            return Err(ProductionFault::WrongScope);
        };
        let profile = template
            .bind(&[])
            .map_err(|_| ProductionFault::FrameHardware(timing::Fault::InvalidProfile))?;
        let [held] = self.timing.work.as_slice() else {
            return Err(ProductionFault::WrongScope);
        };
        if held.kind != timing::WorkKind::Semantic(expected) || held.profile != profile {
            return Err(ProductionFault::WrongScope);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::production::{
        AdmissionFault, AdvanceInput, Production, ProductionContent, ProductionRules,
        ProductionSeed,
    };
    use crate::state::owned::OwnedState;
    use audio_transport::{CpuMasterTicks, Request, SenderProgress, SenderState};
    use data::timing::{Cycle, ExposureKind, TemplateRun};

    #[test]
    fn opening_checkpoint_refuses_a_changed_inactive_sender_clock() {
        let hardware = timing::tests::state(20, 20);
        let sender = SenderState {
            queue: std::collections::VecDeque::new(),
            progress: SenderProgress::Ready,
            reply: Request::Absent,
            now: CpuMasterTicks(0),
            pending: None,
        };
        let senders = Senders::new(sender.clone(), sender.clone(), sender).unwrap();
        let publication = FrameContinuationAfterNmi {
            publication: super::super::borrowed_receipts::PublicationOrigin {
                prior_request: super::super::borrowed_receipts::SoftwareRequest::Acknowledged,
            },
        };
        let current = MusicServiceReceipt {
            countdown: MusicCountdown::Ready,
            queue: MusicQueue::Empty,
        };
        let music = MusicCompletionReceipt {
            dependency: current,
            terminal: current,
            request: MusicRequest::None,
        };
        let start = Opening::new(hardware, senders, publication, current, music).unwrap();
        let profile = Template {
            runs: vec![TemplateRun {
                count: 1,
                cycle: Cycle::Six,
                kind: ExposureKind::Internal,
                poll_before: false,
                effect: TemplateEffect::None,
            }],
        };
        let content = ProductionContent::FrameOpening(Box::new(Profiles {
            acknowledgment: profile.clone(),
            music: profile,
        }));
        let rules = ProductionRules::FrameOpening;
        let mut production = Production::admit(
            ProductionSeed::FrameOpening(Box::new(start)),
            content.clone(),
            rules.clone(),
        )
        .unwrap();
        let opening = loop {
            production
                .advance(AdvanceInput::Frame {
                    budget: MasterTicks(1),
                })
                .unwrap();
            let OwnedState::FrameOpening(opening) = production.state() else {
                panic!("expected an opening owner");
            };
            if matches!(opening.caller(), Caller::MusicRunning { .. })
                && opening.timing().hardware.charge_debt != 0
            {
                break opening;
            }
        };
        let saved = production.save().unwrap();
        assert_eq!(
            Production::restore(&saved, content.clone(), rules.clone()).unwrap(),
            production
        );
        let mut stale = opening.clone();
        stale.senders.contact.0.now = CpuMasterTicks(1);
        let altered = production.encode_save(stale);
        assert!(matches!(
            Production::restore(&altered, content, rules),
            Err(AdmissionFault::Frame(fault))
                if matches!(*fault, super::super::Fault::Production(ProductionFault::WrongScope))
        ));
    }
}
