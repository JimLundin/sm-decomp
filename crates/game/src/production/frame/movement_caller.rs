//! Actual Run query and movement decisions retained through one physical owner.

use super::{
    borrowed_player_preparation::{ActorUpdateCompletion, PlayerPreparationFault},
    inputs::{FigurePin, MovementInputs, MovementInputsPin, WorldPin},
    movement::Movement,
    movement_stage::{self, Completion, Group, Receipt},
};
use crate::production::{AppearanceContent, MovementContent, appearance, timing::WorkId};
use canonical::{Canonical, model::Model};

/// Independently admitted current terrain and direction-independent reaction permissions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Room {
    /// Checked loaded-content room backing the current independently decoded geometry.
    /// Unbound component geometry does not claim integrated room ownership.
    pub identity: Option<data::id::RoomId>,
    /// Complete current room width in blocks.
    pub width: u16,
    /// Complete independently decoded current room.
    pub cells: Vec<types::Block>,
    /// Explicit permission for every cell.
    pub permissions: Vec<bool>,
    /// Independently admitted outside-room geometry.
    pub boundary: crate::production::world::Boundary,
}

/// Disjoint actual current producers borrowed at Run's opportunity.
pub struct Current<'a> {
    /// Actual current appearance owner; this query never advances it.
    pub appearance: &'a appearance::Work,
    /// Actual playing figure, before prospective movement selection.
    pub figure: samus_appearance::Figure,
    /// Actual completed actor update and interactive roster.
    pub actors: &'a ActorUpdateCompletion,
    /// Current inputs from completed producer owners.
    pub inputs: &'a MovementInputs,
}

/// Immutable actual current query, published only with whole Run completion.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct AheadCompletion {
    /// Exact physical owner that billed this query.
    pub work: WorkId,
    /// Actual playing figure used to resolve checked clip content.
    pub figure: Model<FigurePin>,
    /// Read-only snapshot of the actual appearance owner at query time.
    pub appearance: appearance::Work,
    /// Actual query results joined to the current movement inputs.
    pub inputs: Model<MovementInputsPin>,
}

/// Actual query and complete movement decisions published together.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct RunCompletion {
    /// Actual current-playing query, never a future selected clip.
    pub ahead: Box<AheadCompletion>,
    /// Actual completed Run decision sequence.
    pub movement: Completion,
}

/// Saved actual Run operation; offered work still requires full source qualification.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Run {
    /// Actual Run has not been chosen.
    #[default]
    Idle,
    /// Actual query and chosen decisions, awaiting immutable profile qualification.
    Offered {
        /// Actual movement opportunity.
        stage: movement_stage::Stage,
        /// Actual query retained through the complete physical operation.
        ahead: Box<AheadCompletion>,
    },
    /// Qualified whole Run is pending in the central hardware owner.
    Charging {
        /// Actual movement opportunity bound to Run's exact identity.
        stage: movement_stage::Stage,
        /// Retained actual query.
        ahead: Box<AheadCompletion>,
    },
    /// Actual query and movement effects were published once.
    Complete(RunCompletion),
}

/// Missing current content, replaced producer or invalid physical order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Run is absent, already chosen or already completed.
    Order,
    /// Actual current owners changed during the physical hold.
    Dependency,
    /// Current movement body has no admitted immutable content.
    Content,
    /// Actual playing figure lacks checked clip content.
    Clip(crate::production::ClipFault),
    /// Current actor identities or geometry are unavailable.
    Actors(PlayerPreparationFault),
    /// Complete room geometry or permissions are invalid.
    World(crate::production::world::WorldFault),
    /// Actual movement opportunity or completion rejected.
    Movement(movement_stage::Fault),
    /// Actual clean request or retained answer could not advance.
    Work(super::movement::WorkFault),
    /// A local-only Run attempted an unqualified world query.
    Query,
}

#[derive(Default)]
struct RunWorld(core::cell::Cell<bool>);

impl types::World for RunWorld {
    fn block(&self, _: types::Cell) -> types::Block {
        self.0.set(true);
        types::Block::Air
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

impl Room {
    pub(super) fn world(
        &self,
        actors: &ActorUpdateCompletion,
    ) -> Result<crate::production::world::MovementWorld, Fault> {
        crate::production::world::MovementWorld::new(
            self.width,
            self.cells.clone(),
            self.permissions.clone(),
            actors.solid_boxes().map_err(Fault::Actors)?,
            self.boundary,
        )
        .map_err(Fault::World)
    }
}

/// Closed paired physical owners; each consumes one actual request and its answer.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairOwner {
    /// Actual ordinary right travel after Run.
    OrdinaryRight,
    /// Actual independent floor support.
    FloorSupport,
    /// Actual independent ceiling support.
    CeilingSupport,
    /// Actual downward ground-following travel.
    GroundFollowingDown,
    /// Actual rightward adjustment of the selected aim.
    AimRight,
}

impl PairOwner {
    /// Unique identity of the complete physical operation.
    #[must_use]
    pub const fn identity(self) -> WorkId {
        WorkId(match self {
            Self::OrdinaryRight => 1000,
            Self::FloorSupport => 1001,
            Self::CeilingSupport => 1002,
            Self::GroundFollowingDown => 1004,
            Self::AimRight => 1008,
        })
    }

    fn accepts(self, request: &samus_movement::suspended::MovementDecision) -> bool {
        use samus_movement::suspended::MovementDecision;
        use types::{CollisionPurpose, Direction, SupportSurface};
        match (self, request) {
            (Self::OrdinaryRight, MovementDecision::MotionRequest(request)) => {
                request.direction == Direction::Right
                    && request.purpose == CollisionPurpose::OrdinaryMove
            }
            (Self::GroundFollowingDown, MovementDecision::MotionRequest(request)) => {
                request.direction == Direction::Down
                    && request.purpose == CollisionPurpose::GroundFollowing
            }
            (Self::AimRight, MovementDecision::MotionRequest(request)) => {
                request.direction == Direction::Right
                    && request.purpose == CollisionPurpose::AimChange
            }
            (Self::FloorSupport, MovementDecision::SupportRequest { surface, .. }) => {
                *surface == SupportSurface::Floor
            }
            (Self::CeilingSupport, MovementDecision::SupportRequest { surface, .. }) => {
                *surface == SupportSurface::Ceiling
            }
            _ => false,
        }
    }
}

/// Complete actual request, returned answer/products and current provider premises.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct PairReceipt {
    /// Actual semantic owner; family recognition alone is never profile admission.
    pub owner: PairOwner,
    /// Complete actual movement before this request.
    pub before: Movement,
    /// Actual request followed by its retained answer and query products.
    pub decisions: Model<super::movement::ReceiptsPin>,
    /// Read-only complete chosen continuation used to detect replacement during a hold.
    pub retained: Movement,
    /// Actual current inputs used to choose the request and query.
    pub inputs: Model<MovementInputsPin>,
    /// Actual decoded current room and completed ordered actor geometry.
    pub world: Model<WorldPin>,
}

/// One paired operation awaiting qualification, charging or completed once.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Pair {
    /// No actual paired request has been chosen.
    #[default]
    Idle,
    /// Actual query was performed once; full source qualification is required next.
    Offered(PairReceipt),
    /// Whole physical operation is active in the sole hardware owner.
    Charging {
        /// Exact actual owner identity.
        work: WorkId,
        /// Complete retained request and answer, never queried again.
        receipt: PairReceipt,
    },
    /// Actual retained answer was applied once after whole physical completion.
    Complete {
        /// Exact completed physical owner.
        work: WorkId,
        /// Complete actual consumed request and answer evidence.
        receipt: PairReceipt,
    },
}

impl Pair {
    /// Query once on a discardable candidate and retain the actual returned answer.
    /// Full immutable source/physical qualification must precede bind and publication.
    /// # Errors
    /// Invalid order, wrong actual request, unavailable current content or geometry.
    pub fn choose(
        &mut self,
        owner: PairOwner,
        movement: &mut Movement,
        current: &Current<'_>,
        room: &Room,
        content: &MovementContent,
        rules: &samus_movement::Rules,
    ) -> Result<(), Fault> {
        use samus_movement::suspended::MovementDecision;
        if !matches!(self, Self::Idle) {
            return Err(Fault::Order);
        }
        let world = room.world(current.actors)?;
        let radius = content
            .radius(movement.state().pose)
            .ok_or(Fault::Content)?;
        let body = |_| radius;
        let cannon = |pose| content.cannon_heading(pose);
        let frame = current.inputs.frame(&body, &cannon);
        let mut candidate = movement.clone();
        let request = candidate
            .offer(content, rules, &frame, &world)
            .map_err(Fault::Work)?
            .ok_or(Fault::Dependency)?;
        if !owner.accepts(&request) {
            return Err(Fault::Dependency);
        }
        candidate
            .complete(content, rules, &frame, &world)
            .map_err(Fault::Work)?;
        let answer = candidate
            .offer(content, rules, &frame, &world)
            .map_err(Fault::Work)?
            .ok_or(Fault::Dependency)?;
        if !matches!(
            (&request, &answer),
            (MovementDecision::MotionRequest(request), MovementDecision::MotionAnswer {
                request: returned, ..
            }) if request == returned
        ) && !matches!(
            (&request, &answer),
            (
                MovementDecision::SupportRequest { .. },
                MovementDecision::SupportAnswer { .. }
            )
        ) {
            return Err(Fault::Dependency);
        }
        let receipt = PairReceipt {
            owner,
            before: movement.clone(),
            decisions: Model(vec![request, answer]),
            retained: candidate.clone(),
            inputs: Model(*current.inputs),
            world: Model(world),
        };
        *movement = candidate;
        *self = Self::Offered(receipt);
        Ok(())
    }

    /// Actual complete qualification evidence for the current physical owner.
    #[must_use]
    pub const fn receipt(&self) -> Option<&PairReceipt> {
        match self {
            Self::Offered(receipt) | Self::Charging { receipt, .. } => Some(receipt),
            Self::Idle | Self::Complete { .. } => None,
        }
    }

    /// Bind an already fully qualified whole profile begun by the hardware caller.
    /// # Errors
    /// Absent/offered identity mismatch or repeated physical binding.
    pub fn bind(&mut self, work: WorkId) -> Result<(), Fault> {
        let Self::Offered(receipt) = self else {
            return Err(Fault::Order);
        };
        if work != receipt.owner.identity() {
            return Err(Fault::Dependency);
        }
        *self = Self::Charging {
            work,
            receipt: receipt.clone(),
        };
        Ok(())
    }

    /// Apply the retained answer after the exact whole physical owner completes.
    /// No external collision query is repeated during a hold or completion.
    /// # Errors
    /// Wrong identity, changed current inputs/geometry or replaced movement continuation.
    pub fn complete(
        &mut self,
        completed: WorkId,
        movement: &mut Movement,
        current: &Current<'_>,
        room: &Room,
        content: &MovementContent,
        rules: &samus_movement::Rules,
    ) -> Result<PairReceipt, Fault> {
        let Self::Charging { work, receipt } = self else {
            return Err(Fault::Order);
        };
        if *work != completed
            || completed != receipt.owner.identity()
            || movement != &receipt.retained
            || *current.inputs != receipt.inputs.0
            || room.world(current.actors)? != receipt.world.0
        {
            return Err(Fault::Dependency);
        }
        let radius = content
            .radius(movement.state().pose)
            .ok_or(Fault::Content)?;
        let body = |_| radius;
        let cannon = |pose| content.cannon_heading(pose);
        let frame = receipt.inputs.0.frame(&body, &cannon);
        let mut candidate = movement.clone();
        candidate
            .complete(content, rules, &frame, &receipt.world.0)
            .map_err(Fault::Work)?;
        let receipt = receipt.clone();
        *movement = candidate;
        *self = Self::Complete {
            work: completed,
            receipt: receipt.clone(),
        };
        Ok(receipt)
    }
}

impl Current<'_> {
    fn holds(&self, query: &AheadCompletion) -> bool {
        let mut inputs = *self.inputs;
        inputs.animation_ends = query.inputs.0.animation_ends;
        inputs.spin_turned_far = query.inputs.0.spin_turned_far;
        self.figure == query.figure.0
            && self.appearance == &query.appearance
            && inputs == query.inputs.0
    }

    fn query(&self, content: &AppearanceContent) -> Result<AheadCompletion, Fault> {
        let clip =
            content
                .get(self.figure)
                .ok_or(Fault::Clip(crate::production::ClipFault::Missing(
                    self.figure,
                )))?;
        let answer = appearance::ahead(&self.appearance.model.0.state, clip);
        let mut inputs = *self.inputs;
        inputs.animation_ends = answer.end;
        inputs.spin_turned_far = answer.turned_far;
        Ok(AheadCompletion {
            work: WorkId(23),
            figure: Model(self.figure),
            appearance: self.appearance.clone(),
            inputs: Model(inputs),
        })
    }
}

impl Run {
    /// Validate the retained semantic owner and exact Run identity after restore.
    /// # Errors
    /// A restored cursor, query identity or receipt belongs to another operation.
    pub fn validate(&self) -> Result<(), Fault> {
        let (stage, ahead, charging) = match self {
            Self::Idle => return Ok(()),
            Self::Complete(completion) => {
                return if completion.ahead.work == WorkId(23)
                    && completion.movement.work == WorkId(23)
                    && completion.movement.receipt.group == Group::Run
                {
                    Ok(())
                } else {
                    Err(Fault::Dependency)
                };
            }
            Self::Offered { stage, ahead } => (stage, ahead, false),
            Self::Charging { stage, ahead } => (stage, ahead, true),
        };
        ahead.appearance.validate().map_err(|_| Fault::Dependency)?;
        if ahead.work != WorkId(23)
            || stage
                .receipt()
                .is_none_or(|receipt| receipt.group != Group::Run)
            || !matches!(
                (charging, stage),
                (false, movement_stage::Stage::Offered(_))
                    | (
                        true,
                        movement_stage::Stage::Charging {
                            work: WorkId(23),
                            ..
                        }
                    )
            )
        {
            return Err(Fault::Dependency);
        }
        Ok(())
    }

    /// Choose the actual current query and complete Run receipt on a private candidate.
    /// Full source qualification must precede binding or exposing intermediate effects.
    /// # Errors
    /// Invalid order, missing current owners/content or unsupported actual movement.
    pub fn choose(
        &mut self,
        movement: &mut Movement,
        current: &Current<'_>,
        appearance_content: &AppearanceContent,
        movement_content: &MovementContent,
        rules: &samus_movement::Rules,
    ) -> Result<(), Fault> {
        if !matches!(self, Self::Idle) {
            return Err(Fault::Order);
        }
        let ahead = current.query(appearance_content)?;
        let world = RunWorld::default();
        let radius = movement_content
            .radius(movement.state().pose)
            .ok_or(Fault::Content)?;
        let body = |_| radius;
        let cannon = |pose| movement_content.cannon_heading(pose);
        let frame = ahead.inputs.0.frame(&body, &cannon);
        let inputs = movement_stage::Inputs {
            content: movement_content,
            rules,
            frame: &frame,
            world: &world,
        };
        let mut candidate = movement.clone();
        let mut stage = movement_stage::Stage::Idle;
        stage
            .choose(Group::Run, &mut candidate, &inputs)
            .map_err(Fault::Movement)?;
        if world.0.get() {
            return Err(Fault::Query);
        }
        *self = Self::Offered {
            stage,
            ahead: Box::new(ahead),
        };
        *movement = candidate;
        Ok(())
    }

    /// Complete actual decisions for immutable physical contract qualification.
    #[must_use]
    pub const fn receipt(&self) -> Option<&Receipt> {
        match self {
            Self::Offered { stage, .. } | Self::Charging { stage, .. } => stage.receipt(),
            Self::Idle | Self::Complete(_) => None,
        }
    }

    /// Actual retained current-playing query for physical contract qualification.
    #[must_use]
    pub const fn query(&self) -> Option<&AheadCompletion> {
        match self {
            Self::Offered { ahead, .. } | Self::Charging { ahead, .. } => Some(ahead),
            Self::Idle | Self::Complete(_) => None,
        }
    }

    /// Bind Run after full immutable source qualification and hardware begin.
    /// # Errors
    /// Run was not offered, is already bound, or has a different actual identity.
    pub fn bind(&mut self, work: WorkId) -> Result<(), Fault> {
        let Self::Offered { stage, ahead } = self else {
            return Err(Fault::Order);
        };
        if work != ahead.work {
            return Err(Fault::Dependency);
        }
        let mut stage = stage.clone();
        stage.bind(work).map_err(Fault::Movement)?;
        *self = Self::Charging {
            stage,
            ahead: ahead.clone(),
        };
        Ok(())
    }

    /// Publish the actual query and retained movement answer at whole Run completion.
    /// Any attempted world query discards both private candidates before publication.
    /// # Errors
    /// Wrong physical identity, replaced current producers or invalid movement completion.
    pub fn complete(
        &mut self,
        work: WorkId,
        movement: &mut Movement,
        current: &Current<'_>,
        movement_content: &MovementContent,
        rules: &samus_movement::Rules,
    ) -> Result<RunCompletion, Fault> {
        let Self::Charging { stage, ahead } = self else {
            return Err(Fault::Order);
        };
        if work != ahead.work
            || !current.holds(ahead)
            || stage.receipt().is_none_or(|receipt| {
                receipt.decisions.0.iter().any(|decision| {
                    matches!(decision, samus_movement::suspended::MovementDecision::Local {
                        inputs, ..
                    } if inputs.rules != *rules)
                })
            })
        {
            return Err(Fault::Dependency);
        }
        let radius = movement_content
            .radius(movement.state().pose)
            .ok_or(Fault::Content)?;
        let body = |_| radius;
        let cannon = |pose| movement_content.cannon_heading(pose);
        let frame = ahead.inputs.0.frame(&body, &cannon);
        let world = RunWorld::default();
        let inputs = movement_stage::Inputs {
            content: movement_content,
            rules,
            frame: &frame,
            world: &world,
        };
        let mut candidate = movement.clone();
        let mut stage = stage.clone();
        let result = RunCompletion {
            ahead: ahead.clone(),
            movement: stage
                .complete(work, &mut candidate, &inputs)
                .map_err(Fault::Movement)?,
        };
        if world.0.get() {
            return Err(Fault::Query);
        }
        *movement = candidate;
        *self = Self::Complete(result.clone());
        Ok(result)
    }
}

/// Immutable source predicates; these quantities validate actual work and never seed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunRequired {
    /// Actual dash entering Run.
    pub dash_before: types::Fixed,
    /// Actual speed entering Run.
    pub speed_before: types::Fixed,
    /// Checked immutable dash gain.
    pub dash_gain: types::Fixed,
    /// Checked whole-word dash cap used by the selected source arm.
    pub dash_cap_whole: u16,
    /// Checked immutable running acceleration.
    pub speed_step: types::Fixed,
    /// Checked immutable running top speed.
    pub speed_top: types::Fixed,
    /// Checked whole-word horizontal displacement limit.
    pub horizontal_limit_whole: u16,
}

/// Actual completed producers needed by the selected Run predicate.
pub struct RunDependencies<'a> {
    /// This opportunity uses the carried running state after a native return.
    pub recurring: bool,
    /// Actual completed current environment, including liquid and equipment premises.
    pub environment: &'a super::borrowed_player_preparation::EnvironmentCompletion,
    /// Actual completed Inside, HUD and `MotionReset` receipts.
    pub selection: super::borrowed_player_preparation::CompletedPlayerSelection<'a>,
    /// Actual current simulation clock from its selected owner.
    pub clocks: &'a super::borrowed_player_preparation::CollectionClocks,
}

/// Whole local-only Run envelope selected by exact current predicates and decisions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunContract {
    required: RunRequired,
    whole: data::timing::Template,
    saturated: Option<data::timing::Template>,
}

impl RunContract {
    /// Retain independently gated immutable predicates and the complete physical envelope.
    #[must_use]
    pub const fn new(required: RunRequired, whole: data::timing::Template) -> Self {
        Self {
            required,
            whole,
            saturated: None,
        }
    }

    /// Admit physical work when both incoming running speed components are at their caps.
    pub fn set_saturated(&mut self, profile: data::timing::Template) {
        self.saturated = Some(profile);
    }

    /// Qualify exact actual Run decisions and completed producer dependencies.
    /// No expected answer or future state is substituted for actual movement work.
    /// # Errors
    /// Missing completed owners, different semantic arms, operands or retained actual answer.
    pub fn physical(
        &self,
        run: &Run,
        movement: &Movement,
        dependencies: &RunDependencies<'_>,
    ) -> Result<&data::timing::Template, Fault> {
        use samus_movement::suspended::{LocalRule, MovementDecision};
        use types::Parts;
        run.validate()?;
        let receipt = run.receipt().ok_or(Fault::Order)?;
        let query = run.query().ok_or(Fault::Order)?;
        let [rise, start, dash, speed, displacement] = receipt.decisions.0.as_slice() else {
            return Err(Fault::Dependency);
        };
        let local = |decision: &MovementDecision| {
            if let MovementDecision::Local { before, answer, .. } = decision {
                Some((*before, *answer))
            } else {
                None
            }
        };
        let (initial, rise_answer) = local(rise).ok_or(Fault::Dependency)?;
        let (start_before, start_answer) = local(start).ok_or(Fault::Dependency)?;
        let (dash_before, dash_answer) = local(dash).ok_or(Fault::Dependency)?;
        let (speed_before, speed_answer) = local(speed).ok_or(Fault::Dependency)?;
        let (last_before, last_answer) = local(displacement).ok_or(Fault::Dependency)?;
        if !matches!(
            rise,
            MovementDecision::Local {
                rule: LocalRule::RiseEnds,
                ..
            }
        ) || !matches!(
            start,
            MovementDecision::Local {
                rule: LocalRule::FrameStart,
                ..
            }
        ) || initial != rise_answer
            || rise_answer != start_before
            || start_before != start_answer
            || start_answer != dash_before
            || dash_answer != speed_before
            || speed_answer != last_before
            || last_before != last_answer
        {
            return Err(Fault::Dependency);
        }
        self.validate_run_admission(initial, query, dependencies)?;
        self.validate_run_decision_inputs(&receipt.decisions.0, query)?;
        let dash_at_cap = self.dash_path(dash, dash_before, dash_answer)?;
        let MovementDecision::Local {
            rule:
                LocalRule::SpeedBuilding {
                    before,
                    step,
                    sum,
                    top,
                },
            ..
        } = speed
        else {
            return Err(Fault::Dependency);
        };
        if *before != speed_before.run.speed
            || *step != self.required.speed_step
            || *top != self.required.speed_top
            || sum.0 != before.0.wrapping_add(step.0)
            || sum.pixel() != top.pixel()
            || types::part_below(sum.sub(), top.sub())
            || sum.sub() == top.sub()
        {
            return Err(Fault::Dependency);
        }
        let mut expected_speed = speed_before;
        expected_speed.run.speed = *top;
        let total =
            core::num::Wrapping(last_before.run.speed.0.wrapping_add(last_before.run.dash.0));
        if speed_answer != expected_speed
            || !types::part_below(total.pixel().0, self.required.horizontal_limit_whole)
            || total.negative()
            || !matches!(displacement, MovementDecision::Local {
                rule: LocalRule::HorizontalDisplacement { intended, retained_total }, ..
            } if intended.0 == total && *retained_total == total)
            || movement.state() != &last_answer
        {
            return Err(Fault::Dependency);
        }
        if dash_at_cap {
            if before != top {
                return Err(Fault::Dependency);
            }
            self.saturated.as_ref().ok_or(Fault::Dependency)
        } else {
            Ok(&self.whole)
        }
    }

    fn dash_path(
        &self,
        dash: &samus_movement::suspended::MovementDecision,
        dash_before: samus_movement::Samus,
        dash_answer: samus_movement::Samus,
    ) -> Result<bool, Fault> {
        use samus_movement::suspended::{LocalRule, MovementDecision};
        use types::Parts;
        let MovementDecision::Local {
            rule: LocalRule::DashBuilding { before, gain, cap },
            ..
        } = dash
        else {
            return Err(Fault::Dependency);
        };
        if *before != dash_before.run.dash
            || *gain != self.required.dash_gain
            || cap.pixel().0 != self.required.dash_cap_whole
        {
            return Err(Fault::Dependency);
        }
        let dash_at_cap = before == cap;
        if !dash_at_cap && !types::part_below(before.pixel().0, cap.pixel().0) {
            return Err(Fault::Dependency);
        }
        let mut expected_dash = dash_before;
        expected_dash.run.dash = if dash_at_cap {
            *cap
        } else {
            core::num::Wrapping(before.0.wrapping_add(gain.0))
        };
        expected_dash.run.dashing = samus_movement::Dashing::On;
        if dash_answer != expected_dash {
            return Err(Fault::Dependency);
        }
        Ok(dash_at_cap)
    }

    fn validate_run_admission(
        &self,
        initial: samus_movement::Samus,
        query: &AheadCompletion,
        dependencies: &RunDependencies<'_>,
    ) -> Result<(), Fault> {
        use super::borrowed_player_preparation::{ClockDisposition, LiquidSurface};
        if initial != dependencies.selection.motion_reset.previous.0
            || dependencies.selection.motion_reset.work != WorkId(18)
            || dependencies.selection.inside_hud.inside.work != WorkId(15)
            || dependencies.selection.inside_hud.inside.receipt.group != Group::Inside
            || dependencies.selection.inside_hud.controls.work != WorkId(16)
            || dependencies.clocks.simulation != ClockDisposition::Running
            || dependencies.environment.dependency.water != LiquidSurface::Inactive
            || dependencies.environment.dependency.lava_acid != LiquidSurface::Inactive
            || dependencies.environment.dependency.equipment != query.inputs.0.items
            || query.inputs.0.items[types::Item::GravitySuit]
            || query.inputs.0.items[types::Item::SpeedBooster]
            || !query.inputs.0.pad.any(types::button::RUN)
            || query.inputs.0.animation_ends.is_some()
            || query.inputs.0.spin_turned_far
            || query.appearance.model.0.state.stage != types::BoostStage::Stage0
            || query.figure.0.pose.action != types::Action::Running
            || initial.pose.action != types::Action::Running
            || initial.run.carry != samus_movement::Carry::FollowsPad
            || initial.run.dashing != samus_movement::Dashing::On
            || initial.run.way != types::Side::Right
            || (!dependencies.recurring
                && (initial.run.dash != self.required.dash_before
                    || initial.run.speed != self.required.speed_before))
        {
            return Err(Fault::Dependency);
        }
        Ok(())
    }

    fn validate_run_decision_inputs(
        &self,
        decisions: &[samus_movement::suspended::MovementDecision],
        query: &AheadCompletion,
    ) -> Result<(), Fault> {
        use samus_movement::suspended::MovementDecision;
        for decision in decisions {
            let MovementDecision::Local { inputs, .. } = decision else {
                return Err(Fault::Dependency);
            };
            if inputs.pad != query.inputs.0.pad
                || inputs.pressed != query.inputs.0.pressed
                || inputs.items != query.inputs.0.items
                || inputs.animation_ends.is_some()
                || inputs.spin_turned_far
                || inputs.rules.dash_gain != self.required.dash_gain
                || inputs.rules.speeds.running.step != self.required.speed_step
                || inputs.rules.speeds.running.top != self.required.speed_top
                || inputs.rules.speed_cap != samus_movement::SpeedCap::Wordwise
            {
                return Err(Fault::Dependency);
            }
        }
        Ok(())
    }
}
