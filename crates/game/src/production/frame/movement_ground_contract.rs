//! Whole ground-following choice with retained identity premises.

use super::{
    movement::Movement,
    movement_caller::{Current, Room, RunCompletion},
    movement_support_contract::{self as support, ActorCompletionPin, MovementRulesPin},
};
use crate::production::{MovementContent, timing::WorkId};
use canonical::{Canonical, model::Model};
use samus_movement::suspended::{LocalRule, MovementDecision};

/// Missing actual producer, changed opportunity or unexpected query.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Actual current producers or retained continuation differ.
    Dependency,
    /// An identity premise changed gameplay or queried the world.
    Query,
    /// The whole physical operation is not in the required phase.
    Order,
    /// The actual clean continuation rejected an operation.
    Movement(super::movement::WorkFault),
}

/// Complete actual inputs held through the ground-following choice.
pub struct Inputs<'a> {
    /// Actual completed whole Inside owner establishing the admitted bookkeeping arm.
    pub inside: &'a super::movement_stage::Completion,
    /// Current completed neighbouring producers.
    pub current: &'a Current<'a>,
    /// Complete actual room geometry.
    pub room: &'a Room,
    /// Checked current body and cannon content.
    pub content: &'a MovementContent,
    /// Actual current rule profile.
    pub rules: &'a samus_movement::Rules,
    /// Completed Run carrying the pre-stop horizontal total.
    pub run: &'a RunCompletion,
    /// Actual completed compound support query.
    pub support: &'a support::Completion,
}

/// Actual identity prefix and final retained ground-following decision.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    /// Actual upstream Inside completion retained read-only through this operation.
    pub inside: super::movement_stage::Completion,
    /// External movement unchanged during the physical hold.
    pub before: Movement,
    /// Private continuation with the identity prefix consumed.
    pub chosen: Movement,
    /// Actual `NearWall`, `SpaceJump` and `GroundPull` decisions.
    pub decisions: Model<super::movement::ReceiptsPin>,
    /// Actual neighbouring inputs.
    pub inputs: Model<super::inputs::MovementInputsPin>,
    /// Actual complete rules.
    pub rules: Model<MovementRulesPin>,
    /// Actual completed actors.
    pub actors: Model<ActorCompletionPin>,
    /// Actual room and ordered actor geometry.
    pub world: Model<super::inputs::WorldPin>,
    /// Completed actual horizontal provenance.
    pub run: RunCompletion,
    /// Completed actual compound support provenance.
    pub support: support::Completion,
}

/// Complete ground-following choice publication.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    /// Sole whole physical identity.
    pub work: WorkId,
    /// Actual consumed decisions and producer provenance.
    pub receipt: Box<Receipt>,
}

/// One ground-following choice retained through its whole physical envelope.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// No opportunity selected.
    #[default]
    Idle,
    /// Actual decisions awaiting full qualification.
    Offered(Box<Receipt>),
    /// The whole physical owner is pending.
    Charging(Box<Receipt>),
    /// The final actual local answer was consumed once.
    Complete(Completion),
}

#[derive(Default)]
struct NoQuery(core::cell::Cell<bool>);
impl types::World for NoQuery {
    fn block(&self, _: types::Cell) -> types::Block {
        self.0.set(true);
        types::Block::Solid
    }
    fn lets_through(&self, _: types::Cell, _: types::Way) -> bool {
        self.0.set(true);
        false
    }
    fn solid_enemies(&self) -> &[types::Hitbox] {
        self.0.set(true);
        &[]
    }
}

fn world(inputs: &Inputs<'_>) -> Result<crate::production::world::MovementWorld, Fault> {
    crate::production::world::MovementWorld::new(
        inputs.room.width,
        inputs.room.cells.clone(),
        inputs.room.permissions.clone(),
        inputs
            .current
            .actors
            .solid_boxes()
            .map_err(|_| Fault::Dependency)?,
        inputs.room.boundary,
    )
    .map_err(|_| Fault::Dependency)
}

fn local_inputs_hold(
    actual: &samus_movement::suspended::DecisionInputs,
    current: &super::inputs::MovementInputs,
    rules: &samus_movement::Rules,
) -> bool {
    actual.pad == current.pad
        && actual.pressed == current.pressed
        && actual.items == current.items
        && actual.animation_ends == current.animation_ends
        && actual.spin_turned_far == current.spin_turned_far
        && actual.elevator_moving == current.elevator_moving
        && actual.cinematic_active == current.cinematic_active
        && actual.sprite_before_movement == current.sprite_before_movement
        && actual.parity == current.parity
        && &actual.rules == rules
}

fn identity_holds(
    decision: &MovementDecision,
    expected: &LocalRule,
    state: &samus_movement::Samus,
) -> bool {
    matches!(decision, MovementDecision::Local { rule, before, answer, .. } if rule == expected && before == answer && before == state)
}

impl Stage {
    /// Consume only identity premises on a private candidate and retain `GroundPull`.
    /// # Errors
    /// Wrong order, missing actual producers, effectful prefix or additional query.
    pub fn choose(&mut self, movement: &Movement, inputs: &Inputs<'_>) -> Result<(), Fault> {
        if !matches!(self, Self::Idle) {
            return Err(Fault::Order);
        }
        let radius = inputs
            .content
            .radius(movement.state().pose)
            .ok_or(Fault::Dependency)?;
        let body = |_| radius;
        let cannon = |pose| inputs.content.cannon_heading(pose);
        let frame = inputs.current.inputs.frame(&body, &cannon);
        let query = NoQuery::default();
        let mut candidate = movement.clone();
        let mut decisions = Vec::new();
        for expected in [LocalRule::NearWall, LocalRule::SpaceJump] {
            let outputs = candidate.outputs().cloned().ok_or(Fault::Dependency)?;
            let decision = candidate
                .offer(inputs.content, inputs.rules, &frame, &query)
                .map_err(Fault::Movement)?
                .ok_or(Fault::Query)?;
            if !matches!(&decision, MovementDecision::Local { rule, before, answer, .. } if *rule == expected && before == answer)
            {
                return Err(Fault::Query);
            }
            candidate
                .complete(inputs.content, inputs.rules, &frame, &query)
                .map_err(Fault::Movement)?;
            if candidate.state() != movement.state()
                || candidate.outputs() != Some(&outputs)
                || query.0.get()
            {
                return Err(Fault::Query);
            }
            decisions.push(decision);
        }
        let pull = candidate
            .offer(inputs.content, inputs.rules, &frame, &query)
            .map_err(Fault::Movement)?
            .ok_or(Fault::Query)?;
        if query.0.get()
            || !matches!(&pull, MovementDecision::Local { rule: LocalRule::GroundPull { .. }, before, answer, .. } if before == answer)
        {
            return Err(Fault::Query);
        }
        decisions.push(pull);
        *self = Self::Offered(Box::new(Receipt {
            inside: inputs.inside.clone(),
            before: movement.clone(),
            chosen: candidate,
            decisions: Model(decisions),
            inputs: Model(*inputs.current.inputs),
            rules: Model(inputs.rules.clone()),
            actors: Model(inputs.current.actors.clone()),
            world: Model(world(inputs)?),
            run: inputs.run.clone(),
            support: inputs.support.clone(),
        }));
        Ok(())
    }

    /// Actual pending qualification evidence.
    #[must_use]
    pub const fn receipt(&self) -> Option<&Receipt> {
        match self {
            Self::Offered(receipt) | Self::Charging(receipt) => Some(receipt),
            Self::Idle | Self::Complete(_) => None,
        }
    }

    /// Reject replaced rules before every physical advance.
    /// # Errors
    /// Actual current rules differ from the retained opportunity.
    pub fn validate_rules(&self, rules: &samus_movement::Rules) -> Result<(), Fault> {
        if self
            .receipt()
            .is_some_and(|receipt| &receipt.rules.0 != rules)
        {
            return Err(Fault::Dependency);
        }
        Ok(())
    }

    /// Bind the begun whole ground-following choice.
    /// # Errors
    /// Wrong physical identity or repeated binding.
    pub fn bind(&mut self, work: WorkId) -> Result<(), Fault> {
        let Self::Offered(receipt) = self else {
            return Err(Fault::Order);
        };
        if work != WorkId(1003) {
            return Err(Fault::Dependency);
        }
        *self = Self::Charging(receipt.clone());
        Ok(())
    }

    /// Consume the retained final local answer only at whole completion.
    /// # Errors
    /// Replaced producers, wrong owner, unexpected query or changed gameplay/output.
    pub fn complete(
        &mut self,
        work: WorkId,
        movement: &mut Movement,
        inputs: &Inputs<'_>,
        contract: &PhysicalContract,
    ) -> Result<Completion, Fault> {
        contract.physical(self, movement, inputs)?;
        let Self::Charging(receipt) = self else {
            return Err(Fault::Order);
        };
        if work != WorkId(1003) {
            return Err(Fault::Dependency);
        }
        let radius = inputs
            .content
            .radius(movement.state().pose)
            .ok_or(Fault::Dependency)?;
        let body = |_| radius;
        let cannon = |pose| inputs.content.cannon_heading(pose);
        let frame = inputs.current.inputs.frame(&body, &cannon);
        let query = NoQuery::default();
        let mut candidate = receipt.chosen.clone();
        let outputs = candidate.outputs().cloned().ok_or(Fault::Dependency)?;
        candidate
            .complete(inputs.content, inputs.rules, &frame, &query)
            .map_err(Fault::Movement)?;
        if query.0.get()
            || candidate.state() != movement.state()
            || candidate.outputs() != Some(&outputs)
        {
            return Err(Fault::Query);
        }
        let completion = Completion {
            work,
            receipt: receipt.clone(),
        };
        *movement = candidate;
        *self = Self::Complete(completion.clone());
        Ok(completion)
    }
}

/// Whole immutable physical group selected by actual producer and decision predicates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalContract {
    whole: data::timing::Template,
}
impl PhysicalContract {
    /// Retain an independently admitted complete envelope.
    #[must_use]
    pub const fn new(whole: data::timing::Template) -> Self {
        Self { whole }
    }

    /// Qualify all actual decisions and completed producers without recomputing work.
    /// # Errors
    /// Different retained inputs, missing completed support/Run or unsupported ground choice.
    pub fn physical(
        &self,
        stage: &Stage,
        movement: &Movement,
        inputs: &Inputs<'_>,
    ) -> Result<data::timing::Profile, Fault> {
        let receipt = stage.receipt().ok_or(Fault::Order)?;
        if &receipt.inside != inputs.inside
            || inputs.inside.work != WorkId(15)
            || inputs.inside.receipt.group != super::movement_stage::Group::Inside
            || movement != &receipt.before
            || receipt.inputs.0 != *inputs.current.inputs
            || &receipt.rules.0 != inputs.rules
            || &receipt.actors.0 != inputs.current.actors
            || receipt.world.0 != world(inputs)?
            || &receipt.run != inputs.run
            || &receipt.support != inputs.support
            || inputs.run.movement.work != WorkId(23)
            || inputs.support.work != WorkId(1001)
            || inputs.support.receipt.inputs != receipt.inputs
            || inputs.support.receipt.rules != receipt.rules
            || inputs.support.receipt.actors != receipt.actors
            || inputs.support.receipt.world != receipt.world
        {
            return Err(Fault::Dependency);
        }
        let Movement::Work(before_work) = movement else {
            return Err(Fault::Dependency);
        };
        let run_decisions = &inputs.run.movement.receipt.decisions.0;
        if run_decisions.is_empty()
            || !before_work
                .completed
                .0
                .windows(run_decisions.len())
                .any(|slice| slice == run_decisions)
            || !before_work
                .completed
                .0
                .ends_with(&inputs.support.receipt.decisions.0)
        {
            return Err(Fault::Dependency);
        }
        let [
            _,
            MovementDecision::SupportAnswer { check: floor },
            _,
            MovementDecision::SupportAnswer { check: ceiling },
        ] = inputs.support.receipt.decisions.0.as_slice()
        else {
            return Err(Fault::Dependency);
        };
        if floor.surface != types::SupportSurface::Floor
            || ceiling.surface != types::SupportSurface::Ceiling
            || [floor, ceiling].iter().any(|check| {
                check.result != types::SupportResult::NoSlope || check.placement.0.0 != 0
            })
        {
            return Err(Fault::Dependency);
        }
        let Some(MovementDecision::Local {
            rule: LocalRule::HorizontalDisplacement { retained_total, .. },
            ..
        }) = run_decisions.last()
        else {
            return Err(Fault::Dependency);
        };
        let [near, space, pull] = receipt.decisions.0.as_slice() else {
            return Err(Fault::Dependency);
        };
        if before_work.model.0.horizontal_amount != *retained_total {
            return Err(Fault::Dependency);
        }
        for decision in &receipt.decisions.0 {
            let MovementDecision::Local { inputs: actual, .. } = decision else {
                return Err(Fault::Dependency);
            };
            if !local_inputs_hold(actual, inputs.current.inputs, inputs.rules) {
                return Err(Fault::Dependency);
            }
        }
        for (decision, expected) in [(near, LocalRule::NearWall), (space, LocalRule::SpaceJump)] {
            if !identity_holds(decision, &expected, movement.state()) {
                return Err(Fault::Dependency);
            }
        }
        if movement.state().pose.action != types::Action::Running
            || movement.state().spin != samus_movement::Spin::Off
            || movement.state().seat != samus_movement::Seat::Unseated
            || inputs.rules.ground_pull.0 != 65536
            || !matches!(pull, MovementDecision::Local { rule: LocalRule::GroundPull { retained_total: actual, extra, intended, seated: false }, before, answer, inputs: actual_inputs } if actual == retained_total && *extra == inputs.rules.ground_pull && intended.0.0 == retained_total.0.wrapping_add(extra.0) && before == answer && before == movement.state() && actual_inputs.rules == *inputs.rules)
        {
            return Err(Fault::Dependency);
        }
        let Movement::Work(chosen) = &receipt.chosen else {
            return Err(Fault::Dependency);
        };
        if chosen.model.0.offered.as_ref() != Some(pull) {
            return Err(Fault::Dependency);
        }
        self.whole.bind(&[]).map_err(|_| Fault::Dependency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::num::Wrapping;
    use types::Pair;

    fn owner() -> Movement {
        Movement::new(samus_movement::Samus {
            position: Pair {
                x: Wrapping(100 * 65536),
                y: Wrapping(100 * 65536),
            },
            pose: types::Pose {
                body: types::Body::Upright,
                action: types::Action::Running,
                facing: types::Facing::Right,
                aim: types::Aim::Forward,
                gun: types::Gun::Stowed,
                legs: types::Legs::Together,
            },
            run: samus_movement::Run {
                speed: Wrapping(0),
                way: types::Side::Right,
                carry: samus_movement::Carry::FollowsPad,
                dash: Wrapping(0),
                dashing: samus_movement::Dashing::Off,
            },
            vertical: samus_movement::Vertical {
                speed: Wrapping(0),
                phase: samus_movement::Phase::Ground,
                bounces: samus_movement::Bounces::None,
            },
            spin: samus_movement::Spin::Off,
            seat: samus_movement::Seat::Unseated,
            jump_held: samus_movement::JumpHeld::NONE,
        })
    }

    fn current() -> super::super::inputs::MovementInputs {
        super::super::inputs::MovementInputs {
            sprite_before_movement: types::Sprite::Plain,
            cinematic_active: false,
            pad: types::Buttons::default(),
            pressed: types::Buttons::default(),
            items: types::Set::default(),
            spin_turned_far: false,
            elevator_moving: false,
            parity: types::Parity::Even,
            animation_ends: None,
        }
    }
    fn local(rule: LocalRule) -> MovementDecision {
        let current = current();
        let state = *owner().state();
        MovementDecision::Local {
            rule,
            before: state,
            answer: state,
            inputs: Box::new(samus_movement::suspended::DecisionInputs {
                pad: current.pad,
                pressed: current.pressed,
                items: current.items,
                animation_ends: current.animation_ends,
                spin_turned_far: current.spin_turned_far,
                elevator_moving: current.elevator_moving,
                cinematic_active: current.cinematic_active,
                sprite_before_movement: current.sprite_before_movement,
                parity: current.parity,
                rules: samus_movement::Rules::default(),
            }),
        }
    }
    #[test]
    fn restored_identity_prefix_rejects_changed_gameplay_and_rule_order() {
        let decisions = Model::<super::super::movement::ReceiptsPin>(vec![
            local(LocalRule::NearWall),
            local(LocalRule::SpaceJump),
        ]);
        let bytes = canonical::to_bytes(&decisions);
        drop(decisions);
        let mut restored: Model<super::super::movement::ReceiptsPin> =
            canonical::from_bytes(&bytes).unwrap();
        let state = *owner().state();
        for (decision, rule) in restored
            .0
            .iter()
            .zip([LocalRule::NearWall, LocalRule::SpaceJump])
        {
            assert!(identity_holds(decision, &rule, &state));
        }
        let first = restored.0.first_mut().unwrap();
        assert!(!identity_holds(first, &LocalRule::SpaceJump, &state));
        let MovementDecision::Local { answer, .. } = first else {
            panic!();
        };
        answer.run.dashing = samus_movement::Dashing::On;
        assert!(!identity_holds(first, &LocalRule::NearWall, &state));
    }
    #[test]
    fn current_local_premises_reject_replaced_controls_and_rules() {
        let decision = local(LocalRule::NearWall);
        let MovementDecision::Local { inputs, .. } = decision else {
            panic!();
        };
        let current = current();
        let rules = samus_movement::Rules::default();
        assert!(local_inputs_hold(&inputs, &current, &rules));
        let mut changed = current;
        changed.elevator_moving = true;
        assert!(!local_inputs_hold(&inputs, &changed, &rules));
        changed = current;
        changed.parity = types::Parity::Odd;
        assert!(!local_inputs_hold(&inputs, &changed, &rules));
        let mut changed_rules = rules;
        changed_rules.ground_pull.0 = changed_rules.ground_pull.0.wrapping_add(1);
        assert!(!local_inputs_hold(&inputs, &current, &changed_rules));
    }
}
