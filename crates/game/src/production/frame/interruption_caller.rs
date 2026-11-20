//! Current inactive interruption and its physical return.

use super::{
    Caller, Content, Dependency, State, animation_caller, movement::Movement, movement_caller::Run,
    movement_down_contract,
};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::Canonical;
use data::timing::{Profile, Template, TemplateEffect};
use samus_movement::suspended::MovementDecision;
use types::{CandidateResult, CollisionPurpose, Direction, MotionResult, MovementWork};

pub(super) const WORK: timing::WorkId = timing::WorkId(1107);

/// Inactive initial timer and knockback direction decoded at original admission.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Admission {
    origin: [u8; 32],
}

impl Admission {
    /// Retain the complete original image identity after independent inactive-field decoding.
    #[must_use]
    pub const fn inactive(origin: [u8; 32]) -> Self {
        Self { origin }
    }

    pub(super) const fn origin(&self) -> [u8; 32] {
        self.origin
    }
}

/// Whole physical envelope qualified for the inactive, stopped-ground-following arm.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    admission: Admission,
    profile: Profile,
}

impl Contract {
    /// Bind the whole envelope, which has no external effects in this arm.
    /// # Errors
    /// Rejects device effects and invalid physical work.
    pub fn new(admission: Admission, template: &Template) -> Result<Self, ProductionFault> {
        if template
            .runs
            .iter()
            .any(|run| run.effect != TemplateEffect::None)
        {
            return Err(missing());
        }
        Ok(Self {
            admission,
            profile: template.bind(&[]).map_err(|_| missing())?,
        })
    }
}

/// Physical lifetime of the interruption opportunity.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// Interruption has not been entered.
    #[default]
    Pending,
    /// The current movement must remain whole through the physical return.
    Charging(Box<Movement>),
    /// The actual physical return preserved this movement continuation.
    Complete(Box<Movement>),
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::InterruptionPreparation))
}

impl State {
    fn validate_interruption_inputs(&self, contract: &Contract) -> Result<(), ProductionFault> {
        let Caller::PlayerContinuation {
            completed,
            remaining,
            run: Run::Complete(run),
            down: movement_down_contract::Stage::Complete(down),
            post_down,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        if !matches!(self.animation, animation_caller::Stage::Complete(_))
            || !self.animation_prefix_holds(post_down)
            || self.liquid_history.as_ref().is_none_or(|history| {
                !history.interruption_holds(&contract.admission, completed, run)
            })
            || remaining.0.actors.as_ref() != Some(&down.receipt.actors.0)
            || !down.published_no_change(&self.movement)
        {
            return Err(missing());
        }
        Ok(())
    }

    pub(super) fn advance_interruption(
        &mut self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let contract = content.interruption.as_ref().ok_or_else(missing)?;
        self.validate_interruption_inputs(contract)?;
        match &self.interruption {
            Stage::Pending => {
                self.timing
                    .begin(WORK, contract.profile.clone())
                    .map_err(ProductionFault::FrameHardware)?;
                self.interruption = Stage::Charging(Box::new(self.movement.clone()));
            }
            Stage::Charging(before) => {
                let mut active = self
                    .timing
                    .work
                    .iter()
                    .filter(|work| work.kind == timing::WorkKind::Semantic(WORK));
                if before.as_ref() != &self.movement
                    || !active
                        .next()
                        .is_some_and(|work| work.profile == contract.profile)
                    || active.next().is_some()
                {
                    return Err(missing());
                }
            }
            Stage::Complete(_) => {
                return Err(ProductionFault::Unsupported(Unsupported::Frame(
                    Dependency::AimPreparation,
                )));
            }
        }
        Ok(())
    }

    pub(super) fn complete_interruption(
        &mut self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let contract = content.interruption.as_ref().ok_or_else(missing)?;
        self.validate_interruption_inputs(contract)?;
        let Stage::Charging(before) = &self.interruption else {
            return Err(missing());
        };
        if before.as_ref() != &self.movement {
            return Err(missing());
        }
        self.interruption = Stage::Complete(before.clone());
        Ok(())
    }
}

impl movement_down_contract::Completion {
    fn published_no_change(&self, movement: &Movement) -> bool {
        let [
            MovementDecision::MotionRequest(request),
            MovementDecision::MotionAnswer {
                request: returned,
                outputs,
                ..
            },
            tail,
        ] = self.receipt.decisions.0.as_slice()
        else {
            return false;
        };
        let (Movement::Work(chosen), Movement::Work(current)) = (&self.receipt.chosen, movement)
        else {
            return false;
        };
        let [MovementWork::Motion(attempt)] = outputs.movement_work.as_slice() else {
            return false;
        };
        let [candidate] = attempt
            .enemy_search
            .as_ref()
            .map_or(&[][..], |search| search.candidates.as_slice())
        else {
            return false;
        };
        self.work == timing::WorkId(1004)
            && request == returned
            && request.direction == Direction::Down
            && request.purpose == CollisionPurpose::GroundFollowing
            && attempt.result == MotionResult::TerrainStopped
            && candidate.result == CandidateResult::Miss
            && outputs.floor_response.is_none()
            && chosen.model.0.offered.as_ref() == Some(tail)
            && current.model.0.offered.is_none()
            && current
                .completed
                .0
                .strip_prefix(chosen.completed.0.as_slice())
                == Some(core::slice::from_ref(tail))
            && movement.state() == self.receipt.chosen.state()
            && movement.outputs() == self.receipt.chosen.outputs()
    }
}
