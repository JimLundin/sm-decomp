//! Whole downward ground-following travel and its retained identity tail.

use super::{
    movement::Movement,
    movement_caller::{Current, Room},
    movement_ground_contract as ground,
    movement_support_contract::{ActorCompletionPin, MovementRulesPin},
};
use crate::production::{MovementContent, timing::WorkId};
use canonical::{Canonical, model::Model};
use samus_movement::suspended::{LocalRule, MovementDecision};

/// Unsupported actual query arm, changed producer or incomplete arithmetic closure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Actual producers or retained continuation differ.
    Dependency,
    /// Unexpected query, reaction or nonidentity tail.
    Query,
    /// Wrong physical order or repeated operation.
    Order,
    /// Missing, extra or unequal live product samples.
    Samples,
    /// Actual clean movement rejected an operation.
    Movement(super::movement::WorkFault),
}

/// Current actual producers borrowed through the whole downward operation.
pub struct Inputs<'a> {
    /// Actual completed whole Inside owner establishing the admitted bookkeeping arm.
    pub inside: &'a super::movement_stage::Completion,
    /// Current completed actors and neighbouring inputs.
    pub current: &'a Current<'a>,
    /// Complete actual room.
    pub room: &'a Room,
    /// Checked current body and cannon content.
    pub content: &'a MovementContent,
    /// Complete actual current rules.
    pub rules: &'a samus_movement::Rules,
    /// Actual preceding completed ground-following choice.
    pub ground: &'a ground::Completion,
}

/// Independently admitted current Room and exact raw terrain/actor classifications.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceAdmission {
    /// Full independently decoded actual Room.
    pub room: Room,
    /// Exact raw plain-solid classification; never a terrain replacement.
    pub plain_solid: Vec<bool>,
    /// Raw traversed identity joined to actual completed interactive membership.
    pub raw_actor: super::borrowed_preparation::ActorSlotIdentity,
}

/// Actual query, retained answer and private continuation with its tail offered.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    /// Actual upstream Inside completion retained read-only through this operation.
    pub inside: super::movement_stage::Completion,
    /// External owner unchanged through physical charging.
    pub before: Movement,
    /// Actual matching answer consumed privately, identity tail retained.
    pub chosen: Movement,
    /// Actual request, matching answer and `VerticalSpeedGrowth` tail.
    pub decisions: Model<super::movement::ReceiptsPin>,
    /// Actual current neighbouring inputs.
    pub inputs: Model<super::inputs::MovementInputsPin>,
    /// Complete actual current rules.
    pub rules: Model<MovementRulesPin>,
    /// Complete actual completed actor owners.
    pub actors: Model<ActorCompletionPin>,
    /// Actual decoded Room and ordered actor boxes.
    pub world: Model<super::inputs::WorldPin>,
    /// Actual preceding completed `GroundPull` and retained displacement.
    pub ground: ground::Completion,
}

/// Actual complete downward operation, including the consumed identity tail.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    /// Sole whole physical identity.
    pub work: WorkId,
    /// Actual retained query/answer and producer provenance.
    pub receipt: Box<Receipt>,
}

/// One once-queried operation and its sole live arithmetic task.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// No actual opportunity selected.
    #[default]
    Idle,
    /// Actual work awaiting whole-source qualification.
    Offered(Box<Receipt>),
    /// Whole work and active task8 samples remain pending.
    Charging {
        /// Actual query and unchanged external owner.
        receipt: Box<Receipt>,
        /// Live arithmetic closure for this exact request.
        pending: Pending,
    },
    /// Actual chosen continuation and outputs published once.
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
fn identity(decision: &MovementDecision, state: &samus_movement::Samus) -> bool {
    matches!(decision, MovementDecision::Local { rule: LocalRule::VerticalSpeedGrowth, before, answer, .. } if before == answer && before == state)
}
fn current_local(decision: &MovementDecision, inputs: &Inputs<'_>) -> bool {
    let MovementDecision::Local { inputs: actual, .. } = decision else {
        return false;
    };
    let current = inputs.current.inputs;
    actual.pad == current.pad
        && actual.pressed == current.pressed
        && actual.items == current.items
        && actual.animation_ends == current.animation_ends
        && actual.spin_turned_far == current.spin_turned_far
        && actual.elevator_moving == current.elevator_moving
        && actual.cinematic_active == current.cinematic_active
        && actual.sprite_before_movement == current.sprite_before_movement
        && actual.parity == current.parity
        && &actual.rules == inputs.rules
}

impl Stage {
    /// Query the actual world once on a private candidate and retain the actual tail.
    /// # Errors
    /// Wrong order, unavailable content, unexpected request or nonidentity tail.
    pub fn choose(&mut self, movement: &Movement, inputs: &Inputs<'_>) -> Result<(), Fault> {
        if !matches!(self, Self::Idle) {
            return Err(Fault::Order);
        }
        let geometry = world(inputs)?;
        let radius = inputs
            .content
            .radius(movement.state().pose)
            .ok_or(Fault::Dependency)?;
        let body = |_| radius;
        let cannon = |pose| inputs.content.cannon_heading(pose);
        let frame = inputs.current.inputs.frame(&body, &cannon);
        let mut candidate = movement.clone();
        let request = candidate
            .offer(inputs.content, inputs.rules, &frame, &geometry)
            .map_err(Fault::Movement)?
            .ok_or(Fault::Query)?;
        if !matches!(&request, MovementDecision::MotionRequest(request) if request.direction == types::Direction::Down && request.purpose == types::CollisionPurpose::GroundFollowing)
        {
            return Err(Fault::Query);
        }
        candidate
            .complete(inputs.content, inputs.rules, &frame, &geometry)
            .map_err(Fault::Movement)?;
        let answer = candidate
            .offer(inputs.content, inputs.rules, &frame, &geometry)
            .map_err(Fault::Movement)?
            .ok_or(Fault::Query)?;
        if !matches!((&request, &answer), (MovementDecision::MotionRequest(request), MovementDecision::MotionAnswer { request: actual, .. }) if request == actual)
        {
            return Err(Fault::Query);
        }
        let no_query = NoQuery::default();
        candidate
            .complete(inputs.content, inputs.rules, &frame, &no_query)
            .map_err(Fault::Movement)?;
        let outputs = candidate.outputs().cloned().ok_or(Fault::Dependency)?;
        let tail = candidate
            .offer(inputs.content, inputs.rules, &frame, &no_query)
            .map_err(Fault::Movement)?
            .ok_or(Fault::Query)?;
        if no_query.0.get()
            || !identity(&tail, candidate.state())
            || !current_local(&tail, inputs)
            || candidate.outputs() != Some(&outputs)
        {
            return Err(Fault::Query);
        }
        *self = Self::Offered(Box::new(Receipt {
            inside: inputs.inside.clone(),
            before: movement.clone(),
            chosen: candidate,
            decisions: Model(vec![request, answer, tail]),
            inputs: Model(*inputs.current.inputs),
            rules: Model(inputs.rules.clone()),
            actors: Model(inputs.current.actors.clone()),
            world: Model(geometry),
            ground: inputs.ground.clone(),
        }));
        Ok(())
    }
    /// Actual pending source qualification evidence.
    #[must_use]
    pub const fn receipt(&self) -> Option<&Receipt> {
        match self {
            Self::Offered(receipt) | Self::Charging { receipt, .. } => Some(receipt),
            Self::Idle | Self::Complete(_) => None,
        }
    }
    /// Reject changed actual rules before every physical advance.
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
    /// Bind task8 to the sole begun downward physical operation.
    /// # Errors
    /// Wrong whole identity or repeated binding.
    pub fn bind(&mut self, work: WorkId, requests: Requests) -> Result<(), Fault> {
        let Self::Offered(receipt) = self else {
            return Err(Fault::Order);
        };
        let pending = Pending::new(work, requests)?;
        *self = Self::Charging {
            receipt: receipt.clone(),
            pending,
        };
        Ok(())
    }
    /// Deliver only this active owner's actual full product samples.
    /// # Errors
    /// Wrong active owner, device, order or extra sample.
    pub fn sample(
        &mut self,
        work: WorkId,
        device: data::timing::Sample,
        value: u8,
    ) -> Result<(), Fault> {
        let Self::Charging { pending, .. } = self else {
            return Err(Fault::Order);
        };
        pending.sample(work, device, value)
    }
    /// Consume the retained identity tail and publish all actual effects atomically.
    /// # Errors
    /// Changed current producers, incomplete task8, replaced answer or unexpected tail effects.
    pub fn complete(
        &mut self,
        work: WorkId,
        movement: &mut Movement,
        inputs: &Inputs<'_>,
        contract: &PhysicalContract,
    ) -> Result<Completion, Fault> {
        let (_, requests) = contract.physical(self, movement, inputs)?;
        let Self::Charging { receipt, pending } = self else {
            return Err(Fault::Order);
        };
        if pending.requests != requests {
            return Err(Fault::Dependency);
        }
        pending.complete(work)?;
        let candidate = Self::consume_identity(receipt, inputs)?;
        let completion = Completion {
            work,
            receipt: receipt.clone(),
        };
        *movement = candidate;
        *self = Self::Complete(completion.clone());
        Ok(completion)
    }

    fn consume_identity(receipt: &Receipt, inputs: &Inputs<'_>) -> Result<Movement, Fault> {
        let mut candidate = receipt.chosen.clone();
        let state = *candidate.state();
        let outputs = candidate.outputs().cloned().ok_or(Fault::Dependency)?;
        let radius = inputs.content.radius(state.pose).ok_or(Fault::Dependency)?;
        let body = |_| radius;
        let cannon = |pose| inputs.content.cannon_heading(pose);
        let frame = inputs.current.inputs.frame(&body, &cannon);
        let query = NoQuery::default();
        candidate
            .complete(inputs.content, inputs.rules, &frame, &query)
            .map_err(Fault::Movement)?;
        if query.0.get() || candidate.state() != &state || candidate.outputs() != Some(&outputs) {
            return Err(Fault::Query);
        }
        Ok(candidate)
    }
}

impl Completion {
    pub(super) fn validate(
        &self,
        movement: &Movement,
        inputs: &Inputs<'_>,
        contract: &PhysicalContract,
    ) -> Result<(), Fault> {
        let mut expected = Stage::Idle;
        expected.choose(&self.receipt.before, inputs)?;
        if self.work != WorkId(1004)
            || expected.receipt() != Some(self.receipt.as_ref())
            || &Stage::consume_identity(&self.receipt, inputs)? != movement
        {
            return Err(Fault::Dependency);
        }
        contract.physical(&expected, &self.receipt.before, inputs)?;
        Ok(())
    }
}

/// Whole admitted downward envelope, never a primitive or family-only bill.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalContract {
    admission: SourceAdmission,
    template: data::timing::Template,
    odd_two_column: Option<data::timing::Template>,
}
impl PhysicalContract {
    /// Retain independently gated current predicates and the complete physical envelope.
    #[must_use]
    pub const fn new(admission: SourceAdmission, template: data::timing::Template) -> Self {
        Self {
            admission,
            template,
            odd_two_column: None,
        }
    }

    /// Admit the independently qualified odd-parity, two-column physical owner.
    pub fn set_odd_two_column(&mut self, template: data::timing::Template) {
        self.odd_two_column = Some(template);
    }

    /// Whether admission includes the distinct odd-parity physical owner.
    #[must_use]
    pub const fn has_odd_two_column(&self) -> bool {
        self.odd_two_column.is_some()
    }
    /// Validate every actual query and producer before any physical advance or return.
    /// # Errors
    /// Changed dependencies, different geometry/reaction or incompatible complete math task.
    pub fn physical(
        &self,
        stage: &Stage,
        movement: &Movement,
        inputs: &Inputs<'_>,
    ) -> Result<(data::timing::Profile, Requests), Fault> {
        use types::Parts;
        let receipt = stage.receipt().ok_or(Fault::Order)?;
        let intended = self.ground_intent(receipt, movement, inputs)?;
        let [
            MovementDecision::MotionRequest(request),
            answered @ MovementDecision::MotionAnswer {
                request: returned,
                answer,
                outputs,
            },
            tail,
        ] = receipt.decisions.0.as_slice()
        else {
            return Err(Fault::Query);
        };
        let body = inputs
            .content
            .radius(movement.state().pose)
            .ok_or(Fault::Dependency)?;
        if request != returned
            || request.bounds.at != movement.state().position
            || request.bounds.r != body
            || request.direction != types::Direction::Down
            || request.purpose != types::CollisionPurpose::GroundFollowing
            || request.distance != intended.0
            || request.distance.negative()
            || request.distance.0 == 0
            || request.distance.pixel().0 >= 32768
        {
            return Err(Fault::Query);
        }
        let [types::MovementWork::Motion(attempt)] = outputs.movement_work.as_slice() else {
            return Err(Fault::Query);
        };
        Self::validate_enemy_miss(request, attempt, body, inputs)?;
        let (row, template) = self.validate_terrain_stop(
            request,
            answered,
            &attempt.effect,
            movement.state().vertical.speed,
            inputs.room,
            inputs.current.inputs.parity,
        )?;
        let mut expected_state = *movement.state();
        expected_state.position = answer.to;
        let mut expected_outputs = movement.outputs().cloned().ok_or(Fault::Dependency)?;
        expected_outputs.reached.extend(outputs.reached.clone());
        expected_outputs
            .collision_tests
            .extend(outputs.collision_tests.clone());
        expected_outputs
            .movement_work
            .extend(outputs.movement_work.clone());
        expected_outputs.travelled[types::Direction::Down] = answer.accepted;
        if outputs.floor_response.is_some() {
            expected_outputs.floor_response = outputs.floor_response;
        }
        if receipt.chosen.state() != &expected_state
            || receipt.chosen.outputs() != Some(&expected_outputs)
            || !identity(tail, &expected_state)
            || !current_local(tail, inputs)
        {
            return Err(Fault::Query);
        }
        let Movement::Work(chosen) = &receipt.chosen else {
            return Err(Fault::Query);
        };
        let Movement::Work(before) = movement else {
            return Err(Fault::Dependency);
        };
        let mut expected_completed = before.completed.0.clone();
        expected_completed.extend(receipt.decisions.0.iter().take(2).cloned());
        if chosen.model.0.offered.as_ref() != Some(tail) || chosen.completed.0 != expected_completed
        {
            return Err(Fault::Query);
        }
        let requests = Requests {
            row,
            width: u8::try_from(inputs.room.width).map_err(|_| Fault::Query)?,
        };
        if requests.width == 0 {
            return Err(Fault::Query);
        }
        Ok((requests.bind(template)?, requests))
    }

    fn ground_intent(
        &self,
        receipt: &Receipt,
        movement: &Movement,
        inputs: &Inputs<'_>,
    ) -> Result<types::TravelDistance, Fault> {
        let actors = inputs.current.actors;
        if &receipt.inside != inputs.inside
            || inputs.inside.work != WorkId(15)
            || inputs.inside.receipt.group != super::movement_stage::Group::Inside
            || inputs.ground.receipt.inside != receipt.inside
            || movement != &receipt.before
            || inputs.room != &self.admission.room
            || self.admission.plain_solid.len() != inputs.room.cells.len()
            || receipt.inputs.0 != *inputs.current.inputs
            || &receipt.rules.0 != inputs.rules
            || &receipt.actors.0 != actors
            || receipt.world.0 != world(inputs)?
            || &receipt.ground != inputs.ground
            || inputs.ground.work != WorkId(1003)
            || actors.interactive.as_slice() != [self.admission.raw_actor]
            || actors.dependency.clocks.simulation
                != super::borrowed_player_preparation::ClockDisposition::Running
            || actors.dependency.clocks.enemy
                != super::borrowed_player_preparation::ClockDisposition::Running
            || inputs.room.boundary != crate::production::world::Boundary::Air
        {
            return Err(Fault::Dependency);
        }
        let Movement::Work(before_work) = movement else {
            return Err(Fault::Dependency);
        };
        if !before_work
            .completed
            .0
            .ends_with(&inputs.ground.receipt.decisions.0)
            || inputs.ground.receipt.inputs != receipt.inputs
            || inputs.ground.receipt.rules != receipt.rules
            || inputs.ground.receipt.actors != receipt.actors
            || inputs.ground.receipt.world != receipt.world
        {
            return Err(Fault::Dependency);
        }
        let Some(MovementDecision::Local {
            rule:
                LocalRule::GroundPull {
                    retained_total,
                    extra,
                    intended,
                    seated: false,
                },
            ..
        }) = inputs.ground.receipt.decisions.0.last()
        else {
            return Err(Fault::Dependency);
        };
        if before_work.model.0.horizontal_amount != *retained_total
            || *extra != inputs.rules.ground_pull
            || extra.0 != 65536
            || intended.0.0 != retained_total.0.wrapping_add(extra.0)
            || movement.state().seat != samus_movement::Seat::Unseated
            || movement.state().pose.action != types::Action::Running
        {
            return Err(Fault::Dependency);
        }
        Ok(*intended)
    }

    fn validate_enemy_miss(
        request: &samus_movement::suspended::MotionRequest,
        attempt: &types::MotionAttempt,
        body: types::Radius,
        inputs: &Inputs<'_>,
    ) -> Result<(), Fault> {
        use types::Parts;
        let search = attempt.enemy_search.as_ref().ok_or(Fault::Query)?;
        let [candidate] = search.candidates.as_slice() else {
            return Err(Fault::Query);
        };
        let boxes = inputs
            .current
            .actors
            .solid_boxes()
            .map_err(|_| Fault::Dependency)?;
        let [actor] = boxes.as_slice() else {
            return Err(Fault::Query);
        };
        let dx = candidate
            .centre
            .x
            .0
            .checked_sub(request.bounds.at.x.pixel().0)
            .ok_or(Fault::Query)?;
        let separation = dx.checked_sub(candidate.radius.x.0).ok_or(Fault::Query)?;
        let fraction = u32::from(request.bounds.at.y.sub())
            .checked_add(u32::from(request.distance.sub()))
            .ok_or(Fault::Query)?;
        request
            .bounds
            .at
            .y
            .pixel()
            .0
            .checked_add(request.distance.pixel().0)
            .and_then(|whole| whole.checked_add(2))
            .ok_or(Fault::Query)?;
        if dx >= 32768
            || separation < body.x.0
            || fraction <= 65536
            || fraction >= 131_072
            || search.supplied_count.0 != 1
            || candidate.candidate.0 != 0
            || candidate.result != types::CandidateResult::Miss
            || candidate.centre != actor.centre
            || candidate.radius != actor.radius
            || attempt.start != request.bounds.at
            || attempt.radius != body
            || attempt.direction != request.direction
            || attempt.purpose != request.purpose
            || attempt.world != types::RequestedWorld::EnemiesThenBlocks
            || attempt.intended.0 != request.distance
            || attempt.result != types::MotionResult::TerrainStopped
        {
            return Err(Fault::Query);
        }
        Ok(())
    }

    fn validate_terrain_stop(
        &self,
        request: &samus_movement::suspended::MotionRequest,
        decision: &MovementDecision,
        effect: &types::RequestEffect,
        speed: types::Fixed,
        room: &Room,
        parity: types::Parity,
    ) -> Result<(u8, &data::timing::Template), Fault> {
        use types::Parts;
        let MovementDecision::MotionAnswer {
            answer, outputs, ..
        } = decision
        else {
            return Err(Fault::Query);
        };
        let body = request.bounds.r;
        let left = request
            .bounds
            .at
            .x
            .pixel()
            .0
            .checked_sub(body.x.0)
            .ok_or(Fault::Query)?;
        let right = request
            .bounds
            .at
            .x
            .pixel()
            .0
            .checked_add(body.x.0)
            .and_then(|whole| whole.checked_sub(1))
            .ok_or(Fault::Query)?;
        let leading = request
            .bounds
            .at
            .y
            .pixel()
            .0
            .checked_add(request.distance.pixel().0)
            .and_then(|whole| whole.checked_add(body.y.0))
            .ok_or(Fault::Query)?;
        let left_column = left >> 4;
        let right_column = right >> 4;
        let (x, template) = match parity {
            types::Parity::Even if left_column == right_column => (left_column, &self.template),
            types::Parity::Odd if left_column.checked_add(1) == Some(right_column) => (
                right_column,
                self.odd_two_column.as_ref().ok_or(Fault::Dependency)?,
            ),
            types::Parity::Even | types::Parity::Odd => return Err(Fault::Query),
        };
        let cell = types::Cell { x, y: leading >> 4 };
        let index = usize::from(cell.y)
            .checked_mul(usize::from(room.width))
            .and_then(|base| base.checked_add(usize::from(cell.x)))
            .ok_or(Fault::Query)?;
        let centre = cell
            .y
            .checked_mul(16)
            .and_then(|whole| whole.checked_sub(body.y.0))
            .ok_or(Fault::Query)?;
        let gap = centre
            .checked_sub(request.bounds.at.y.pixel().0)
            .ok_or(Fault::Query)?;
        let [test] = outputs.collision_tests.as_slice() else {
            return Err(Fault::Query);
        };
        let [reached] = outputs.reached.as_slice() else {
            return Err(Fault::Query);
        };
        if left > right
            || right >= 4096
            || leading >= 4096
            || cell.x >= room.width
            || index >= 32768
            || gap >= 32768
            || room.cells.get(index) != Some(&types::Block::Solid)
            || self.admission.plain_solid.get(index) != Some(&true)
            || test.cell != cell
            || test.direction != request.direction
            || test.purpose != request.purpose
            || test.vertical_motion.is_some()
            || reached.cell != cell
            || reached.way != types::Way::Move(types::Direction::Down)
            || answer.stop.is_none()
            || answer.accepted.0.pixel().0 != gap
            || answer.accepted.0.sub() != 0
            || answer.gone.0 != gap
            || answer.to.x != request.bounds.at.x
            || answer.to.y.pixel().0 != centre
            || answer.to.y.sub() != 65535
            || (parity == types::Parity::Odd && (gap != 0 || answer.to != request.bounds.at))
            || *effect != types::RequestEffect::Travel(answer.accepted)
            || outputs.pending_aim.is_some()
            || outputs
                .floor_response
                .is_some_and(|response| response.speed.0 != speed)
        {
            return Err(Fault::Query);
        }
        Ok((u8::try_from(cell.y).map_err(|_| Fault::Query)?, template))
    }
}

/// Actual geometric multiply request for one complete paired physical owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requests {
    /// Actual first queried cell row.
    row: u8,
    /// Actual admitted complete room width.
    width: u8,
}

impl Requests {
    /// Bind the complete owner with its exact publication and sample sequence.
    /// # Errors
    /// Missing, extra, reordered or incompatible arithmetic effects.
    pub fn bind(self, template: &data::timing::Template) -> Result<data::timing::Profile, Fault> {
        use data::timing::{DeviceWrite, Publication, Sample, TemplateEffect};
        let values = vec![self.row, self.width];
        let mut values = values.into_iter();
        let mut writes = Vec::new();
        let mut count = 0_u8;
        for run in &template.runs {
            for _ in 0..run.count {
                match run.effect {
                    TemplateEffect::None => {}
                    TemplateEffect::Publish(port) => {
                        let value = values.next().ok_or(Fault::Samples)?;
                        let write = DeviceWrite::from_cpu_port(port.0, value)
                            .map_err(|_| Fault::Samples)?;
                        if count != 0
                            || !matches!(
                                (writes.len(), &write.effect),
                                (0, Publication::Multiplicand(_)) | (1, Publication::Multiplier(_))
                            )
                        {
                            return Err(Fault::Samples);
                        }
                        writes.push(write);
                    }
                    TemplateEffect::Sample(sample) => {
                        if !values.as_slice().is_empty()
                            || !matches!(
                                (count, sample),
                                (0, Sample::ProductLow) | (1, Sample::ProductHigh)
                            )
                        {
                            return Err(Fault::Samples);
                        }
                        count = count.checked_add(1).ok_or(Fault::Samples)?;
                    }
                }
            }
        }
        if values.next().is_some() || count != 2 {
            return Err(Fault::Samples);
        }
        template.bind(&writes).map_err(|_| Fault::Samples)
    }
}

/// Live arithmetic closure held by one exact actual paired physical operation.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Pending {
    work: WorkId,
    requests: Requests,
    count: u8,
    bytes: [u8; 2],
}

impl Pending {
    /// The actual current operands bound to this in-flight hardware request.
    #[must_use]
    pub const fn requests(&self) -> Requests {
        self.requests
    }

    /// Retain actual derived request operands; source products never initialize these bytes.
    /// # Errors
    /// The requests belong to another actual owner or have an invalid room width.
    pub fn new(work: WorkId, requests: Requests) -> Result<Self, Fault> {
        if work != WorkId(1004) || requests.width == 0 {
            return Err(Fault::Dependency);
        }
        Ok(Self {
            work,
            requests,
            count: 0,
            bytes: [0; 2],
        })
    }

    /// Retain only this active owner's actual hardware low/high bytes in order.
    /// # Errors
    /// Wrong owner, wrong device or duplicated/reordered sample.
    pub fn sample(
        &mut self,
        work: WorkId,
        sample: data::timing::Sample,
        value: u8,
    ) -> Result<(), Fault> {
        use data::timing::Sample;
        if work != self.work
            || !matches!(
                (self.count, sample),
                (0, Sample::ProductLow) | (1, Sample::ProductHigh)
            )
        {
            return Err(Fault::Samples);
        }
        let byte = self
            .bytes
            .get_mut(usize::from(self.count))
            .ok_or(Fault::Samples)?;
        *byte = value;
        self.count = self.count.checked_add(1).ok_or(Fault::Samples)?;
        Ok(())
    }

    /// Require both actual samples and their actual row/width product before publication.
    /// # Errors
    /// Wrong owner, absent samples or unequal actual product.
    pub fn complete(&self, work: WorkId) -> Result<(), Fault> {
        if work != self.work
            || self.count != 2
            || u16::from_le_bytes(self.bytes)
                != u16::from(self.requests.row).wrapping_mul(u16::from(self.requests.width))
        {
            return Err(Fault::Samples);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data::timing::Sample;

    #[test]
    fn held_product_requires_both_actual_bytes_and_exact_work_after_restore() {
        let work = WorkId(1004);
        let mut pending = Pending::new(work, Requests { row: 3, width: 7 }).unwrap();
        assert_eq!(pending.complete(work), Err(Fault::Samples));
        let untouched = pending.clone();
        assert_eq!(
            pending.sample(WorkId(1001), Sample::ProductLow, 21),
            Err(Fault::Samples)
        );
        assert_eq!(pending, untouched);
        pending.sample(work, Sample::ProductLow, 21).unwrap();
        let bytes = canonical::to_bytes(&pending);
        let mut pending: Pending = canonical::from_bytes(&bytes).unwrap();
        assert_eq!(pending.complete(work), Err(Fault::Samples));
        pending.sample(work, Sample::ProductHigh, 1).unwrap();
        assert_eq!(pending.complete(work), Err(Fault::Samples));
        assert_eq!(pending.complete(WorkId(1001)), Err(Fault::Samples));
    }

    #[test]
    fn actual_product_checks_full_width_and_rejects_extra_samples() {
        let work = WorkId(1004);
        let mut pending = Pending::new(
            work,
            Requests {
                row: 255,
                width: 255,
            },
        )
        .unwrap();
        let [low, high] = 65025_u16.to_le_bytes();
        assert_eq!(
            pending.sample(work, Sample::ProductHigh, high),
            Err(Fault::Samples)
        );
        pending.sample(work, Sample::ProductLow, low).unwrap();
        pending.sample(work, Sample::ProductHigh, high).unwrap();
        assert_eq!(pending.complete(work), Ok(()));
        let complete = pending.clone();
        assert_eq!(
            pending.sample(work, Sample::ProductHigh, high),
            Err(Fault::Samples)
        );
        assert_eq!(pending, complete);
    }
}

#[cfg(test)]
mod tail_tests {
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
    fn restored_growth_tail_rejects_changed_state_or_another_rule() {
        let decisions = Model::<super::super::movement::ReceiptsPin>(vec![local(
            LocalRule::VerticalSpeedGrowth,
        )]);
        let bytes = canonical::to_bytes(&decisions);
        drop(decisions);
        let mut restored: Model<super::super::movement::ReceiptsPin> =
            canonical::from_bytes(&bytes).unwrap();
        let state = *owner().state();
        let decision = restored.0.first_mut().unwrap();
        assert!(identity(decision, &state));
        let MovementDecision::Local { answer, .. } = decision else {
            panic!();
        };
        answer.vertical.speed.0 = answer.vertical.speed.0.wrapping_add(1);
        assert!(!identity(decision, &state));
        let MovementDecision::Local { answer, rule, .. } = decision else {
            panic!();
        };
        *answer = state;
        *rule = LocalRule::NearWall;
        assert!(!identity(decision, &state));
    }
}
