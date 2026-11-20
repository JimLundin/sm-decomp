//! Actual movement opportunities retained across whole physical operations.

use super::movement::{Movement, ReceiptsPin, WorkFault};
use crate::production::{MovementContent, timing::WorkId};
use canonical::{Canonical, model::Model};
use samus_movement::suspended::{LocalRule, MovementDecision};

#[cfg(test)]
mod tests;

/// A semantic operation whose complete physical profile is charged once.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    /// One complete inside-body query and its retained reaction answer.
    Inside,
    /// One actual offered request, returned answer or local decision.
    Decision,
    /// No-effect rise/frame-start prefix, then actual dash, speed and displacement.
    Run,
    /// Actual support-dependent ground-following choice.
    GroundPull,
}

/// Complete actual decisions submitted to the physical profile qualifier.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    /// Semantic operation, independent of source coordinates.
    pub group: Group,
    /// Earlier consumed decisions followed by the final retained offered decision.
    pub decisions: Model<ReceiptsPin>,
}

/// Actual operation consumed after its matching physical owner completed.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    /// Read-only actual continuation immediately before the final answer was consumed.
    pub retained: Movement,
    /// Completed physical owner identity.
    pub work: WorkId,
    /// Actual chosen requests and answers consumed by this operation.
    pub receipt: Receipt,
}

/// Metadata only; the complete clean continuation remains in the Movement owner.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// No physical operation has been chosen.
    #[default]
    Idle,
    /// Actual decisions await qualification against a complete source envelope.
    Offered(Receipt),
    /// A qualified whole operation is pending in the sole hardware owner.
    Charging {
        /// Actual physical operation identity.
        work: WorkId,
        /// Chosen decisions, never recomputed during a hold.
        receipt: Receipt,
    },
}

/// Fresh external premises borrowed at the actual movement opportunity.
pub struct Inputs<'a> {
    /// Checked current body and cannon content.
    pub content: &'a MovementContent,
    /// Rules current at this opportunity.
    pub rules: &'a samus_movement::Rules,
    /// Current controls and independently admitted neighbouring outputs.
    pub frame: &'a samus_movement::Frame<'a>,
    /// Current geometry and reaction permissions.
    pub world: &'a dyn types::World,
}

/// An invalid caller order or unqualified semantic sequence rejects atomically.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Operation already chosen, not bound, or already completed.
    Order,
    /// Actual offered rule does not belong to this qualified operation shape.
    Sequence,
    /// Physical completion or retained offered answer does not match the choice.
    Completion,
    /// Actual clean movement admission or completion failed.
    Work(WorkFault),
}

impl Inputs<'_> {
    fn offer(&self, movement: &mut Movement) -> Result<MovementDecision, Fault> {
        movement
            .offer(self.content, self.rules, self.frame, self.world)
            .map_err(Fault::Work)?
            .ok_or(Fault::Sequence)
    }

    fn complete(&self, movement: &mut Movement) -> Result<(), Fault> {
        movement
            .complete(self.content, self.rules, self.frame, self.world)
            .map_err(Fault::Work)
    }
}

impl Stage {
    /// Choose actual decisions and retain their final answer.
    /// Inside queries current room geometry once while choosing; its later completion
    /// publishes that retained answer. Other requests query at physical completion.
    /// Intermediate local effects stay inside the sole movement owner. The caller must
    /// qualify the full receipt, including current operands/content and group invariants,
    /// before binding a profile or exposing those effects to a dependent reader.
    /// # Errors
    /// A pending operation, unsupported sequence or unavailable current content.
    pub fn choose(
        &mut self,
        group: Group,
        movement: &mut Movement,
        inputs: &Inputs<'_>,
    ) -> Result<&Receipt, Fault> {
        if !matches!(self, Self::Idle) {
            return Err(Fault::Order);
        }
        let mut candidate = movement.clone();
        let mut decisions = Vec::new();
        let first = inputs.offer(&mut candidate)?;
        match group {
            Group::Inside => {
                if !matches!(first, MovementDecision::InsideRequest(_)) {
                    return Err(Fault::Sequence);
                }
                decisions.push(first);
                inputs.complete(&mut candidate)?;
                let answer = inputs.offer(&mut candidate)?;
                if !matches!(answer, MovementDecision::InsideAnswer(_)) {
                    return Err(Fault::Sequence);
                }
                decisions.push(answer);
            }
            Group::Decision => decisions.push(first),
            Group::GroundPull => {
                if !matches!(
                    first,
                    MovementDecision::Local {
                        rule: LocalRule::GroundPull { .. },
                        ..
                    }
                ) {
                    return Err(Fault::Sequence);
                }
                decisions.push(first);
            }
            Group::Run => {
                Self::choose_run(first, &mut candidate, inputs, &mut decisions)?;
            }
        }
        *movement = candidate;
        *self = Self::Offered(Receipt {
            group,
            decisions: Model(decisions),
        });
        let Self::Offered(receipt) = self else {
            return Err(Fault::Order);
        };
        Ok(receipt)
    }

    fn choose_run(
        first: MovementDecision,
        candidate: &mut Movement,
        inputs: &Inputs<'_>,
        decisions: &mut Vec<MovementDecision>,
    ) -> Result<(), Fault> {
        let prefix_outputs = candidate.outputs().cloned().ok_or(Fault::Sequence)?;
        if !matches!(&first, MovementDecision::Local {
            rule: LocalRule::RiseEnds, before, answer: after, ..
        } if before == after)
        {
            return Err(Fault::Sequence);
        }
        decisions.push(first);
        inputs.complete(candidate)?;
        if candidate.outputs() != Some(&prefix_outputs) {
            return Err(Fault::Sequence);
        }
        let start = inputs.offer(candidate)?;
        if !matches!(&start, MovementDecision::Local {
            rule: LocalRule::FrameStart, before, answer: after, ..
        } if before == after)
        {
            return Err(Fault::Sequence);
        }
        decisions.push(start);
        inputs.complete(candidate)?;
        if candidate.outputs() != Some(&prefix_outputs) {
            return Err(Fault::Sequence);
        }
        let first = inputs.offer(candidate)?;
        if !matches!(
            first,
            MovementDecision::Local {
                rule: LocalRule::DashBuilding { .. } | LocalRule::Stride,
                ..
            }
        ) {
            return Err(Fault::Sequence);
        }
        decisions.push(first);
        inputs.complete(candidate)?;
        let speed = inputs.offer(candidate)?;
        if !matches!(
            speed,
            MovementDecision::Local {
                rule: LocalRule::SpeedBuilding { .. } | LocalRule::Stride,
                ..
            }
        ) {
            return Err(Fault::Sequence);
        }
        decisions.push(speed);
        inputs.complete(candidate)?;
        let displacement = inputs.offer(candidate)?;
        if !matches!(
            displacement,
            MovementDecision::Local {
                rule: LocalRule::HorizontalDisplacement { .. },
                ..
            }
        ) {
            return Err(Fault::Sequence);
        }
        decisions.push(displacement);
        Ok(())
    }

    /// Retained complete actual decisions for source-envelope qualification or revalidation.
    #[must_use]
    pub const fn receipt(&self) -> Option<&Receipt> {
        match self {
            Self::Idle => None,
            Self::Offered(receipt) | Self::Charging { receipt, .. } => Some(receipt),
        }
    }

    /// Bind an already qualified whole profile begun by the central hardware caller.
    /// Merely recognizing a rule family is insufficient qualification.
    /// # Errors
    /// The actual operation is absent or already bound.
    pub fn bind(&mut self, work: WorkId) -> Result<(), Fault> {
        let Self::Offered(receipt) = self else {
            return Err(Fault::Order);
        };
        *self = Self::Charging {
            work,
            receipt: receipt.clone(),
        };
        Ok(())
    }

    /// Apply the final retained decision once at actual whole-operation completion.
    /// Requests read the current world here; returned/local answers remain fixed.
    /// # Errors
    /// Wrong physical identity, replaced answer or unavailable admitted current content.
    pub fn complete(
        &mut self,
        completed: WorkId,
        movement: &mut Movement,
        inputs: &Inputs<'_>,
    ) -> Result<Completion, Fault> {
        let Self::Charging { work, receipt } = self else {
            return Err(Fault::Order);
        };
        if *work != completed {
            return Err(Fault::Completion);
        }
        let Movement::Work(current) = movement else {
            return Err(Fault::Completion);
        };
        if current.model.0.offered.as_ref() != receipt.decisions.0.last() {
            return Err(Fault::Completion);
        }
        let result = Completion {
            retained: movement.clone(),
            work: *work,
            receipt: receipt.clone(),
        };
        inputs.complete(movement)?;
        *self = Self::Idle;
        Ok(result)
    }
}
