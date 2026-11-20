//! Late ordinary movement selects its own prospective pose before charging aim work.

use super::{Caller, Content, Dependency, Rules, State, interruption_caller};
use super::{
    inputs::MovementInputs,
    movement::Movement,
    movement_caller::{Current, Pair, PairOwner, Run},
    movement_pair_contract::{self, CompletionInputs},
};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::Canonical;
use data::timing::{Profile, Template, TemplateEffect};
use samus_movement::suspended::{LocalRule, MovementDecision};
use types::{Action, Facing, Parts};

pub(super) const CALLER: timing::WorkId = timing::WorkId(1006);
pub(super) const ADMISSION: timing::WorkId = timing::WorkId(1007);
pub(super) const AIM_RIGHT: timing::WorkId = PairOwner::AimRight.identity();

/// Each whole physical opportunity qualified by the actual retained pose choice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    caller: Profile,
    admission: Profile,
}

impl Contract {
    /// Import the complete inactive-device caller and pose-choice envelopes.
    /// # Errors
    /// Rejects invalid envelopes or envelopes with device effects.
    pub fn new(caller: &Template, admission: &Template) -> Result<Self, ProductionFault> {
        let bind = |template: &Template| {
            if template
                .runs
                .iter()
                .any(|run| run.effect != TemplateEffect::None)
            {
                return Err(missing());
            }
            template.bind(&[]).map_err(|_| missing())
        };
        Ok(Self {
            caller: bind(caller)?,
            admission: bind(admission)?,
        })
    }
}

/// Retained late movement work and its actual physical progress.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// The upcoming late movement caller has not run.
    #[default]
    Pending,
    /// The active caller precedes the actual pose choice.
    Caller,
    /// The actual caller returned; the prospective decision may now be selected.
    Choosing,
    /// The actual posed answer is held privately until the admission return.
    Admission(Box<Movement>),
    /// Actual pose choice completed and the clean aim opportunity is next.
    AimReady,
    /// Actual Aim request and answer are retained while hardware charges.
    Aim(Box<movement_pair_contract::Stage>),
    /// Actual rightward travel returned; the movement is ready for body fitting.
    FitReady(Box<movement_pair_contract::Stage>),
    /// One shared envelope retains the body-fitting and appearance choices.
    Fit(Box<super::pose_fit_caller::FitWork>),
    /// Completed movement and its actual selected playing publication.
    Complete(Box<super::appearance_caller::Completion>),
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::AimPreparation))
}

impl State {
    pub(super) fn late_movement_inputs(&self) -> Result<MovementInputs, ProductionFault> {
        let Caller::PlayerContinuation {
            run: Run::Complete(run),
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let mut inputs = self.run_inputs()?;
        inputs.sprite_before_movement = run.ahead.figure.0.sprite;
        inputs.animation_ends = run.ahead.inputs.0.animation_ends;
        inputs.spin_turned_far = run.ahead.inputs.0.spin_turned_far;
        if inputs != run.ahead.inputs.0 || self.movement_inputs.as_ref() != Some(&run.ahead.inputs)
        {
            return Err(missing());
        }
        Ok(inputs)
    }

    fn choose_late_pose(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<Movement, ProductionFault> {
        let interruption_caller::Stage::Complete(before) = &self.interruption else {
            return Err(missing());
        };
        let Caller::PlayerContinuation {
            completed,
            remaining,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let pose = completed.0.pose_input.as_ref().ok_or_else(missing)?;
        if self.movement != **before
            || self.movement.state().pose != pose.dependency.pose
            || self.movement.state().pose.action != Action::Running
        {
            return Err(missing());
        }
        let inputs = self.late_movement_inputs()?;
        let world = content
            .movement_room
            .as_ref()
            .ok_or_else(missing)?
            .world(remaining.0.actors.as_ref().ok_or_else(missing)?)
            .map_err(|_| missing())?;
        let body = |_| pose.terminal.radius;
        let cannon = |figure| content.movement.cannon_heading(figure);
        let frame = inputs.frame(&body, &cannon);
        let mut candidate = self.movement.clone();
        candidate
            .offer(&content.movement, &rules.movement, &frame, &world)
            .map_err(|_| missing())?
            .ok_or_else(missing)?;
        self.validate_late_pose(&candidate, content, rules)?;
        Ok(candidate)
    }

    fn validate_late_pose(
        &self,
        candidate: &Movement,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        let Caller::PlayerContinuation { completed, .. } = &self.caller else {
            return Err(missing());
        };
        let pose = completed.0.pose_input.as_ref().ok_or_else(missing)?;
        let interruption_caller::Stage::Complete(interrupted) = &self.interruption else {
            return Err(missing());
        };
        let Movement::Work(retained) = candidate else {
            return Err(missing());
        };
        let Some(MovementDecision::Local {
            rule: LocalRule::PoseChoice { choice, .. },
            before,
            answer,
            inputs,
        }) = retained.model.0.offered.as_ref()
        else {
            return Err(missing());
        };
        let current = self.late_movement_inputs()?;
        let movement_rules = &rules.movement;
        if self.movement != **interrupted
            || self.movement.state().pose != pose.dependency.pose
            || candidate.state() != self.movement.state()
            || candidate.outputs() != self.movement.outputs()
            || candidate.decisions() != self.movement.decisions()
            || content.movement.radius(pose.dependency.pose) != Some(pose.terminal.radius)
            || before != self.movement.state()
            || before != answer
            || choice.pose != pose.terminal.prospective
            || choice.pose.action != Action::Running
            || choice.pose.facing != Facing::Right
            || !choice.aim_step
            || &inputs.rules != movement_rules
            || inputs.pad != current.pad
            || inputs.pressed != current.pressed
            || inputs.items != current.items
            || inputs.animation_ends != current.animation_ends
            || inputs.spin_turned_far != current.spin_turned_far
            || inputs.elevator_moving != current.elevator_moving
            || inputs.cinematic_active != current.cinematic_active
            || inputs.sprite_before_movement != current.sprite_before_movement
            || inputs.parity != current.parity
        {
            return Err(missing());
        }
        Ok(())
    }

    pub(super) fn advance_late_movement(
        &mut self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        let contract = content.late_movement.as_ref().ok_or_else(missing)?;
        match &self.late_movement {
            Stage::Pending => {
                let interruption_caller::Stage::Complete(before) = &self.interruption else {
                    return Err(missing());
                };
                if self.movement != **before {
                    return Err(missing());
                }
                self.begin_late_work(CALLER, contract.caller.clone(), Stage::Caller)?;
            }
            Stage::Caller => self.validate_late_profile(CALLER, &contract.caller)?,
            Stage::Choosing => {
                let candidate = self.choose_late_pose(content, rules)?;
                self.begin_late_work(
                    ADMISSION,
                    contract.admission.clone(),
                    Stage::Admission(Box::new(candidate)),
                )?;
            }
            Stage::Admission(retained) => {
                self.validate_late_pose(retained, content, rules)?;
                self.validate_late_profile(ADMISSION, &contract.admission)?;
            }
            Stage::AimReady => self.begin_late_aim(content, rules)?,
            Stage::Aim(active) => {
                let receipt = active.receipt().ok_or_else(missing)?;
                let inputs = self.late_movement_inputs()?;
                let aim = self.aim_context(content, rules, &inputs)?;
                if receipt.inputs.0 != inputs {
                    return Err(missing());
                }
                active.validate_rules(aim.rules).map_err(|_| missing())?;
                let (profile, _) = aim
                    .contract
                    .physical(receipt, &self.movement, aim.room, aim.actors, aim.content)
                    .map_err(|_| missing())?;
                self.validate_late_profile(AIM_RIGHT, &profile)?;
            }
            Stage::FitReady(_) => {
                let (fit, profile) = self.choose_pose_fit(content, rules)?;
                self.begin_late_work(
                    super::pose_fit_caller::WORK,
                    profile,
                    Stage::Fit(Box::new(fit)),
                )?;
            }
            Stage::Fit(fit) => {
                let profile = self.validate_fit(fit, content)?;
                self.validate_late_profile(super::pose_fit_caller::WORK, &profile)?;
            }
            Stage::Complete(_) => {
                return Err(ProductionFault::Unsupported(Unsupported::Frame(
                    Dependency::SamusPalettePreparation,
                )));
            }
        }
        Ok(())
    }

    fn begin_late_work(
        &mut self,
        work: timing::WorkId,
        profile: Profile,
        stage: Stage,
    ) -> Result<(), ProductionFault> {
        self.timing
            .begin(work, profile)
            .map_err(ProductionFault::FrameHardware)?;
        self.late_movement = stage;
        Ok(())
    }

    fn validate_late_profile(
        &self,
        work: timing::WorkId,
        profile: &Profile,
    ) -> Result<(), ProductionFault> {
        let mut active = self
            .timing
            .work
            .iter()
            .filter(|candidate| candidate.kind == timing::WorkKind::Semantic(work));
        if !active
            .next()
            .is_some_and(|candidate| &candidate.profile == profile)
            || active.next().is_some()
        {
            return Err(missing());
        }
        Ok(())
    }

    pub(super) fn complete_late_work(
        &mut self,
        work: timing::WorkId,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        match (&self.late_movement, work) {
            (Stage::Caller, CALLER) => self.late_movement = Stage::Choosing,
            (Stage::Admission(retained), ADMISSION) => {
                self.validate_late_pose(retained, content, rules)?;
                let mut candidate = retained.as_ref().clone();
                let Caller::PlayerContinuation {
                    completed,
                    remaining,
                    ..
                } = &self.caller
                else {
                    return Err(missing());
                };
                let pose = completed.0.pose_input.as_ref().ok_or_else(missing)?;
                let inputs = self.late_movement_inputs()?;
                let world = content
                    .movement_room
                    .as_ref()
                    .ok_or_else(missing)?
                    .world(remaining.0.actors.as_ref().ok_or_else(missing)?)
                    .map_err(|_| missing())?;
                let body = |_| pose.terminal.radius;
                let cannon = |figure| content.movement.cannon_heading(figure);
                let frame = inputs.frame(&body, &cannon);
                candidate
                    .complete(&content.movement, &rules.movement, &frame, &world)
                    .map_err(|_| missing())?;
                self.movement = candidate;
                self.late_movement = Stage::AimReady;
            }
            (Stage::Fit(_), super::pose_fit_caller::WORK) => {
                self.finish_pose_fit(content, rules)?;
            }
            (Stage::Aim(active), AIM_RIGHT) => {
                let inputs = self.late_movement_inputs()?;
                let aim = self.aim_context(content, rules, &inputs)?;
                let current = aim.current();
                let inputs = CompletionInputs {
                    current: &current,
                    room: aim.room,
                    content: aim.content,
                    rules: aim.rules,
                    contract: aim.contract,
                };
                let mut completed = active.clone();
                let mut movement = self.movement.clone();
                completed
                    .complete(work, &mut movement, &inputs)
                    .map_err(|_| missing())?;
                self.movement = movement;
                self.late_movement = Stage::FitReady(completed);
            }
            _ => return Err(missing()),
        }
        Ok(())
    }

    fn aim_context<'a>(
        &'a self,
        content: &'a Content,
        rules: &'a Rules,
        inputs: &'a MovementInputs,
    ) -> Result<AimContext<'a>, ProductionFault> {
        let Caller::PlayerContinuation { remaining, .. } = &self.caller else {
            return Err(missing());
        };
        Ok(AimContext {
            state: self,
            actors: remaining.0.actors.as_ref().ok_or_else(missing)?,
            room: content.movement_room.as_ref().ok_or_else(missing)?,
            content: &content.movement,
            rules: &rules.movement,
            contract: content.aim_right.as_ref().ok_or_else(missing)?,
            inputs,
        })
    }

    fn begin_late_aim(&mut self, content: &Content, rules: &Rules) -> Result<(), ProductionFault> {
        let inputs = self.late_movement_inputs()?;
        let aim = self.aim_context(content, rules, &inputs)?;
        let current = aim.current();
        let world = aim.room.world(aim.actors).map_err(|_| missing())?;
        let radius = content
            .movement
            .radius(self.movement.state().pose)
            .ok_or_else(missing)?;
        let body = |_| radius;
        let cannon = |figure| content.movement.cannon_heading(figure);
        let frame = inputs.frame(&body, &cannon);
        let mut movement = self.movement.clone();
        let Some(MovementDecision::Local {
            rule: LocalRule::AimTravel { intended },
            before,
            answer,
            ..
        }) = movement
            .offer(aim.content, aim.rules, &frame, &world)
            .map_err(|_| missing())?
        else {
            return Err(missing());
        };
        if intended.0.negative()
            || intended.0.pixel().0 != 1
            || intended.0.sub() != 0
            || before != answer
        {
            return Err(missing());
        }
        movement
            .complete(aim.content, aim.rules, &frame, &world)
            .map_err(|_| missing())?;
        let mut pair = Pair::Idle;
        pair.choose(
            PairOwner::AimRight,
            &mut movement,
            &current,
            aim.room,
            aim.content,
            aim.rules,
        )
        .map_err(|_| missing())?;
        let mut active =
            movement_pair_contract::Stage::offered(pair, aim.rules).map_err(|_| missing())?;
        let (profile, requests) = aim
            .contract
            .physical(
                active.receipt().ok_or_else(missing)?,
                &movement,
                aim.room,
                aim.actors,
                aim.content,
            )
            .map_err(|_| missing())?;
        active.bind(AIM_RIGHT, requests).map_err(|_| missing())?;
        self.begin_late_work(AIM_RIGHT, profile, Stage::Aim(Box::new(active)))?;
        self.movement = movement;
        Ok(())
    }

    pub(super) fn sample_late_aim(
        &mut self,
        events: &[timing::Event],
    ) -> Result<(), ProductionFault> {
        let Stage::Aim(active) = &mut self.late_movement else {
            return Err(missing());
        };
        for event in events {
            if let timing::EventKind::Sample(sample, timing::SampleValue::Byte(value)) = &event.kind
            {
                active
                    .sample(AIM_RIGHT, *sample, *value)
                    .map_err(|_| missing())?;
            }
        }
        Ok(())
    }
}

struct AimContext<'a> {
    state: &'a State,
    actors: &'a super::borrowed_player_preparation::ActorUpdateCompletion,
    room: &'a super::movement_caller::Room,
    content: &'a crate::production::MovementContent,
    rules: &'a samus_movement::Rules,
    contract: &'a movement_pair_contract::PhysicalContract,
    inputs: &'a MovementInputs,
}

impl AimContext<'_> {
    const fn current(&self) -> Current<'_> {
        Current {
            appearance: &self.state.appearance,
            figure: self.state.playing_figure.0,
            actors: self.actors,
            inputs: self.inputs,
        }
    }
}
