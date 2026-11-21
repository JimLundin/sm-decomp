//! Selected countdown work and its separately consumed completion receipt.

use super::{Content, Dependency, Rules, State, countdown_scene, hud::Hud};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use data::timing::{Profile, Template, TemplateEffect};
use interface_hud::{CountdownAnswer, CountdownOpportunity, Observation};

/// Whole selected countdown owner through its physical return.
pub const COUNTDOWN: timing::WorkId = timing::WorkId(1300);

/// Immutable complete no-device-effect bill; quantities and selection remain live owners.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Physical {
    /// Full checked selected countdown work.
    pub template: Template,
}

impl Physical {
    /// Validate the full bill before admitting any work.
    /// # Errors
    /// Empty, malformed or device-effectful work is unsupported.
    pub fn profile(&self) -> Result<Profile, ProductionFault> {
        if self.template.runs.is_empty()
            || self
                .template
                .runs
                .iter()
                .any(|part| part.effect != TemplateEffect::None)
        {
            return Err(missing());
        }
        self.template.bind(&[]).map_err(|_| missing())
    }
}

/// Actual delivered answer retained independently from model observation progress.
pub struct AnswerPin;
impl Pinned for AnswerPin {
    type State = CountdownAnswer;
    const NAME: &'static str = "production-countdown-delivery";
    const SHAPE: u64 = 0xA51D_61F2_252B_F7E2;
}

/// Caller progress retains selection and consumption independently from the clean work.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// No countdown selected before actual returned ordinary HUD.
    #[default]
    Pending,
    /// Actual selected work awaits its whole physical return.
    Charging {
        /// Current scene qualification received at the actual opportunity.
        offer: countdown_scene::Offer,
        /// Complete returned ordinary HUD receipt.
        before: Box<Hud>,
    },
    /// Actual return consumed once, independently from clean observation progress.
    Complete {
        /// Same scene offer whose selected work returned.
        offer: countdown_scene::Offer,
        /// Complete unchanged ordinary output and quantities.
        before: Box<Hud>,
        /// Receipt of the actual consumed answer.
        answer: Model<AnswerPin>,
    },
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::CountdownPreparation))
}

impl Stage {
    pub(super) fn holds(&self, state: &State) -> bool {
        let active: Vec<_> = state
            .timing
            .work
            .iter()
            .filter(|work| work.kind == timing::WorkKind::Semantic(COUNTDOWN))
            .collect();
        match self {
            Self::Pending => {
                active.is_empty()
                    && state.countdown_delivered.is_empty()
                    && state.hud.countdown_answer().is_none()
            }
            Self::Charging { before, .. } => {
                active.len() == 1
                    && state.countdown_delivered.is_empty()
                    && state.hud.retains_ordinary(before)
                    && matches!(state.hud.countdown_answer(), Some(Observation::Chosen(_)))
            }
            Self::Complete { before, answer, .. } => {
                active.is_empty()
                    && state.hud.retains_ordinary(before)
                    && state.hud.countdown_answer() == Some(&Observation::Completed(answer.0))
                    && state.countdown_delivered.as_slice() == core::slice::from_ref(answer)
            }
        }
    }
}

impl State {
    pub(super) fn advance_countdown(
        &mut self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        if !self.countdown.holds(self) {
            return Err(missing());
        }
        let offer = self.countdown_scene_offer(content, rules)?;
        let physical = content.countdown.as_ref().ok_or_else(missing)?;
        match &self.countdown {
            Stage::Pending => {
                let before = self.hud.clone();
                let mut hud = before.clone();
                if !hud.begin_countdown()
                    || !hud.choose_countdown(CountdownOpportunity::MaintainCountdown)
                {
                    return Err(missing());
                }
                self.timing
                    .begin(COUNTDOWN, physical.profile()?)
                    .map_err(ProductionFault::FrameHardware)?;
                self.hud = hud;
                self.countdown = Stage::Charging {
                    offer,
                    before: Box::new(before),
                };
            }
            Stage::Charging {
                offer: selected, ..
            } => {
                if *selected != offer || !self.countdown_profile_holds(physical) {
                    return Err(missing());
                }
            }
            Stage::Complete {
                offer: selected, ..
            } => {
                if *selected != offer {
                    return Err(missing());
                }
                return Err(ProductionFault::Unsupported(Unsupported::Frame(
                    Dependency::AnimationPreparation,
                )));
            }
        }
        Ok(())
    }

    fn countdown_profile_holds(&self, physical: &Physical) -> bool {
        physical
            .profile()
            .is_ok_and(|profile| self.has_physical_bill(COUNTDOWN, &profile))
    }

    pub(crate) fn validate_countdown_checkpoint(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), super::Fault> {
        if matches!(self.countdown, Stage::Charging { .. })
            && (self.countdown_scene_offer(content, rules).is_err()
                || content
                    .countdown
                    .as_ref()
                    .is_none_or(|physical| !self.countdown_profile_holds(physical)))
        {
            return Err(super::Fault::InvalidPlayerEntry);
        }
        Ok(())
    }

    pub(super) fn complete_countdown(
        &mut self,
        identity: timing::WorkId,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        let current = self.countdown_scene_offer(content, rules)?;
        let Stage::Charging { offer, before } = &self.countdown else {
            return Err(missing());
        };
        if identity != COUNTDOWN
            || *offer != current
            || !self.hud.retains_ordinary(before)
            || !self.countdown_delivered.is_empty()
            || !matches!(self.hud.countdown_answer(), Some(Observation::Chosen(_)))
        {
            return Err(missing());
        }
        let mut hud = self.hud.clone();
        if !hud.apply_countdown() {
            return Err(missing());
        }
        let Some(Observation::Completed(answer)) = hud.countdown_answer() else {
            return Err(missing());
        };
        let receipt = Model(*answer);
        let completion = Stage::Complete {
            offer: *offer,
            before: before.clone(),
            answer: receipt,
        };
        self.hud = hud;
        self.countdown_delivered.push(receipt);
        self.countdown = completion;
        if !self.countdown.holds(self) {
            return Err(missing());
        }
        Ok(())
    }
}
