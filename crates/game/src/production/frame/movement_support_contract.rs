//! Complete support owner and read-only current rule provenance.

use super::{
    TravelWork,
    movement::Movement,
    movement_caller::{Current, Room},
    movement_pair_contract::SourceAdmission,
};
use crate::production::{MovementContent, timing::WorkId};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};

/// Read-only actual completed actor owners held through the support opportunity.
pub struct ActorCompletionPin;
impl Pinned for ActorCompletionPin {
    type State = super::borrowed_player_preparation::ActorUpdateCompletion;
    const NAME: &'static str = "production-current-actor-completion";
    const SHAPE: u64 = 0x1DEB_5F0B_D081_2315;
}

/// Read-only exact actual rule profile held by a physical movement operation.
pub struct MovementRulesPin;
impl Pinned for MovementRulesPin {
    type State = samus_movement::Rules;
    const NAME: &'static str = "production-current-movement-rules";
    const SHAPE: u64 = 0x6E13_8C3D_9A84_4003;
}

/// Invalid current support owner, query sequence or live hardware closure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The complete actual current dependency differs from its retained snapshot.
    Dependency,
    /// Unexpected query, unsupported support geometry or omitted/reordered answer.
    Query,
    /// Missing, reordered or unequal active-owner arithmetic samples.
    Samples,
    /// Actual movement admission or completion rejected.
    Movement(super::movement::WorkFault),
}

/// Immutable evidence for both actual support queries, with a single chosen continuation.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    /// Actual external movement owner retained unchanged until full support completion.
    pub before: Movement,
    /// Read-only chosen continuation with Floor consumed and Ceiling answer retained.
    pub chosen: Movement,
    /// Actual Floor request/answer followed by actual Ceiling request/answer.
    pub decisions: Model<super::movement::ReceiptsPin>,
    /// Actual current completed controls and neighbouring query outputs.
    pub inputs: Model<super::inputs::MovementInputsPin>,
    /// Actual current rule profile, never a synthetic local decision.
    pub rules: Model<MovementRulesPin>,
    /// Actual completed actor identity and full owners, retained read-only.
    pub actors: Model<ActorCompletionPin>,
    /// Actual complete admitted room and ordered actor geometry.
    pub world: Model<super::inputs::WorldPin>,
}

/// Actual complete support publication at its sole physical completion.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    /// Exact compound physical owner, shared admission and return included once.
    pub work: WorkId,
    /// Both actual queried and consumed support answers.
    pub receipt: Box<Receipt>,
}

/// One whole support owner, with no independent Floor or Ceiling hardware completion.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// No actual compound support opportunity has been chosen.
    #[default]
    Idle,
    /// Both actual queries await full current-source/physical qualification.
    Offered(Box<Receipt>),
    /// Whole support physical work and both live products are pending.
    Charging {
        /// Actual immutable chosen continuation and current dependencies.
        receipt: Box<Receipt>,
        /// Both active-owner arithmetic tasks.
        pending: Pending,
    },
    /// Both actual answers were published once at the outer physical return.
    Complete(Completion),
}

#[derive(Default)]
struct Queries<'a> {
    room: Option<&'a Room>,
    fault: core::cell::Cell<bool>,
    cells: core::cell::Cell<Vec<types::Cell>>,
}

impl types::World for Queries<'_> {
    fn block(&self, cell: types::Cell) -> types::Block {
        let mut cells = self.cells.take();
        cells.push(cell);
        self.cells.set(cells);
        let block = self.room.and_then(|room| {
            if cell.x >= room.width {
                return None;
            }
            usize::from(cell.y)
                .checked_mul(usize::from(room.width))
                .and_then(|base| base.checked_add(usize::from(cell.x)))
                .and_then(|index| room.cells.get(index))
                .copied()
        });
        if let Some(block @ (types::Block::Air | types::Block::Solid)) = block {
            block
        } else {
            self.fault.set(true);
            types::Block::Solid
        }
    }
    fn lets_through(&self, _: types::Cell, _: types::Way) -> bool {
        self.fault.set(true);
        false
    }
    fn solid_enemies(&self) -> &[types::Hitbox] {
        self.fault.set(true);
        &[]
    }
}

fn world(
    room: &Room,
    current: &Current<'_>,
) -> Result<crate::production::world::MovementWorld, Fault> {
    crate::production::world::MovementWorld::new(
        room.width,
        room.cells.clone(),
        room.permissions.clone(),
        current
            .actors
            .solid_boxes()
            .map_err(|_| Fault::Dependency)?,
        room.boundary,
    )
    .map_err(|_| Fault::Dependency)
}

fn stationary_support(
    decision: &samus_movement::suspended::MovementDecision,
    expected: types::SupportSurface,
) -> Result<&types::SupportCheck, Fault> {
    let samus_movement::suspended::MovementDecision::SupportAnswer { check } = decision else {
        return Err(Fault::Query);
    };
    if check.surface != expected
        || check.result != types::SupportResult::NoSlope
        || check.placement.0.0 != 0
    {
        return Err(Fault::Query);
    }
    Ok(check)
}

impl Stage {
    /// Query both actual supports once on a candidate without publishing any Movement effect.
    /// # Errors
    /// Wrong order, unexpected queries, unavailable current content or nonidentity Floor.
    pub fn choose(
        &mut self,
        movement: &Movement,
        current: &Current<'_>,
        room: &Room,
        content: &MovementContent,
        rules: &samus_movement::Rules,
    ) -> Result<(), Fault> {
        use samus_movement::suspended::MovementDecision;
        if !matches!(self, Self::Idle) {
            return Err(Fault::Dependency);
        }
        let geometry = world(room, current)?;
        let queries = Queries {
            room: Some(room),
            ..Queries::default()
        };
        let radius = content
            .radius(movement.state().pose)
            .ok_or(Fault::Dependency)?;
        let body = |_| radius;
        let cannon = |pose| content.cannon_heading(pose);
        let frame = current.inputs.frame(&body, &cannon);
        let mut candidate = movement.clone();
        let floor = candidate
            .offer(content, rules, &frame, &queries)
            .map_err(Fault::Movement)?
            .ok_or(Fault::Query)?;
        if !matches!(
            floor,
            MovementDecision::SupportRequest {
                surface: types::SupportSurface::Floor,
                ..
            }
        ) {
            return Err(Fault::Query);
        }
        candidate
            .complete(content, rules, &frame, &queries)
            .map_err(Fault::Movement)?;
        let floor_answer = candidate
            .offer(content, rules, &frame, &queries)
            .map_err(Fault::Movement)?
            .ok_or(Fault::Query)?;
        let floor_check = stationary_support(&floor_answer, types::SupportSurface::Floor)?;
        let mut expected_outputs = candidate.outputs().cloned().ok_or(Fault::Dependency)?;
        expected_outputs
            .movement_work
            .push(types::MovementWork::Support(floor_check.clone()));
        candidate
            .complete(content, rules, &frame, &queries)
            .map_err(Fault::Movement)?;
        if candidate.state() != movement.state() || candidate.outputs() != Some(&expected_outputs) {
            return Err(Fault::Query);
        }
        let ceiling = candidate
            .offer(content, rules, &frame, &queries)
            .map_err(Fault::Movement)?
            .ok_or(Fault::Query)?;
        if !matches!(
            ceiling,
            MovementDecision::SupportRequest {
                surface: types::SupportSurface::Ceiling,
                ..
            }
        ) {
            return Err(Fault::Query);
        }
        candidate
            .complete(content, rules, &frame, &queries)
            .map_err(Fault::Movement)?;
        let ceiling_answer = candidate
            .offer(content, rules, &frame, &queries)
            .map_err(Fault::Movement)?
            .ok_or(Fault::Query)?;
        let ceiling_check = stationary_support(&ceiling_answer, types::SupportSurface::Ceiling)?;
        if queries.fault.get()
            || queries.cells.take() != [floor_check.cell, ceiling_check.cell]
            || candidate.state() != movement.state()
        {
            return Err(Fault::Query);
        }
        *self = Self::Offered(Box::new(Receipt {
            before: movement.clone(),
            chosen: candidate,
            decisions: Model(vec![floor, floor_answer, ceiling, ceiling_answer]),
            inputs: Model(*current.inputs),
            rules: Model(rules.clone()),
            actors: Model(current.actors.clone()),
            world: Model(geometry),
        }));
        Ok(())
    }

    /// Actual current support evidence for full source qualification.
    #[must_use]
    pub const fn receipt(&self) -> Option<&Receipt> {
        match self {
            Self::Offered(receipt) | Self::Charging { receipt, .. } => Some(receipt),
            Self::Idle | Self::Complete(_) => None,
        }
    }

    /// Reject a replaced actual rule profile before every physical advance.
    /// # Errors
    /// The actual selected current profile differs from the immutable opportunity snapshot.
    pub fn validate_rules(&self, current: &samus_movement::Rules) -> Result<(), Fault> {
        if self
            .receipt()
            .is_some_and(|receipt| &receipt.rules.0 != current)
        {
            return Err(Fault::Dependency);
        }
        Ok(())
    }
}

/// Derived operands for Floor and Ceiling, with one complete physical owner identity.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requests {
    width: u8,
    rows: [u8; 2],
}

impl Requests {
    /// Bind Width/Row/Zero and Low/High twice, preserving both task boundaries.
    /// # Errors
    /// Omitted, extra, reordered or incompatible whole-owner arithmetic effects.
    pub fn bind(self, template: &data::timing::Template) -> Result<data::timing::Profile, Fault> {
        use data::timing::{DeviceWrite, Publication, Sample, TemplateEffect};
        let mut values = self.rows.into_iter().flat_map(|row| [self.width, row, 0]);
        let mut writes = Vec::new();
        let mut samples = 0_usize;
        for run in &template.runs {
            for _ in 0..run.count {
                match run.effect {
                    TemplateEffect::None => {}
                    TemplateEffect::Publish(port) => {
                        let value = values.next().ok_or(Fault::Samples)?;
                        let write = DeviceWrite::from_cpu_port(port.0, value)
                            .map_err(|_| Fault::Samples)?;
                        let expected_samples = writes
                            .len()
                            .checked_div(3)
                            .and_then(|task| task.checked_mul(2))
                            .ok_or(Fault::Samples)?;
                        if samples != expected_samples
                            || !matches!(
                                (writes.len() % 3, &write.effect),
                                (0, Publication::Multiplicand(_))
                                    | (1, Publication::Multiplier(_))
                                    | (2, Publication::DividendLow(0))
                            )
                        {
                            return Err(Fault::Samples);
                        }
                        writes.push(write);
                    }
                    TemplateEffect::Sample(sample) => {
                        let expected = if samples.is_multiple_of(2) {
                            Sample::ProductLow
                        } else {
                            Sample::ProductHigh
                        };
                        let expected_writes = samples
                            .checked_div(2)
                            .and_then(|task| task.checked_add(1))
                            .and_then(|task| task.checked_mul(3))
                            .ok_or(Fault::Samples)?;
                        if sample != expected || writes.len() != expected_writes || samples >= 4 {
                            return Err(Fault::Samples);
                        }
                        samples = samples.checked_add(1).ok_or(Fault::Samples)?;
                    }
                }
            }
        }
        if values.next().is_some() || writes.len() != 6 || samples != 4 {
            return Err(Fault::Samples);
        }
        template.bind(&writes).map_err(|_| Fault::Samples)
    }
}

/// Two actual live math closures billed and completed only by compound `WorkId` 1001.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Pending {
    requests: Requests,
    count: u8,
    bytes: [u8; 4],
}

impl Pending {
    /// Retain both derived actual requests without a native expected product.
    const fn new(requests: Requests) -> Self {
        Self {
            requests,
            count: 0,
            bytes: [0; 4],
        }
    }

    fn sample(
        &mut self,
        work: WorkId,
        device: data::timing::Sample,
        value: u8,
    ) -> Result<(), Fault> {
        use data::timing::Sample;
        let expected = if self.count.is_multiple_of(2) {
            Sample::ProductLow
        } else {
            Sample::ProductHigh
        };
        if work != TravelWork::Support.identity() || device != expected {
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

    fn complete(&self, work: WorkId) -> Result<(), Fault> {
        let [floor_low, floor_high, ceiling_low, ceiling_high] = self.bytes;
        if work != TravelWork::Support.identity()
            || self.count != 4
            || [
                u16::from_le_bytes([floor_low, floor_high]),
                u16::from_le_bytes([ceiling_low, ceiling_high]),
            ] != self
                .requests
                .rows
                .map(|row| u16::from(row).wrapping_mul(u16::from(self.requests.width)))
        {
            return Err(Fault::Samples);
        }
        Ok(())
    }
}

/// Complete source-qualified support routine, never two separately billed native returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalContract {
    admission: SourceAdmission,
    template: data::timing::Template,
}

/// Fresh actual providers for source revalidation and full outer completion.
pub struct Inputs<'a> {
    /// Actual current completed actor and control owners.
    pub current: &'a Current<'a>,
    /// Actual current unchanged room.
    pub room: &'a Room,
    /// Actual checked current body/cannon content.
    pub content: &'a MovementContent,
    /// Actual current rule profile.
    pub rules: &'a samus_movement::Rules,
}

impl PhysicalContract {
    /// Retain independently admitted current source predicates and the whole envelope.
    #[must_use]
    pub const fn new(admission: SourceAdmission, template: data::timing::Template) -> Self {
        Self {
            admission,
            template,
        }
    }

    /// Qualify both actual requests/answers and derive both math operands.
    /// # Errors
    /// Different current owners, wrong support order, substituted queries or geometry.
    pub fn physical(
        &self,
        stage: &Stage,
        movement: &Movement,
        inputs: &Inputs<'_>,
    ) -> Result<(data::timing::Profile, Requests), Fault> {
        let receipt = stage.receipt().ok_or(Fault::Dependency)?;
        stage.validate_rules(inputs.rules)?;
        if &receipt.before != movement
            || receipt.chosen.state() != movement.state()
            || &receipt.actors.0 != inputs.current.actors
            || inputs.current.actors.interactive.as_slice() != [self.admission.raw_actor]
            || inputs.current.actors.dependency.clocks.simulation
                != super::borrowed_player_preparation::ClockDisposition::Running
            || inputs.current.actors.dependency.clocks.enemy
                != super::borrowed_player_preparation::ClockDisposition::Running
            || receipt.inputs.0 != *inputs.current.inputs
            || inputs.room != &self.admission.room
            || receipt.world.0 != world(inputs.room, inputs.current)?
            || !self.admission.support_alignment
        {
            return Err(Fault::Dependency);
        }
        let [floor_request, floor_answer, ceiling_request, ceiling_answer] =
            receipt.decisions.0.as_slice()
        else {
            return Err(Fault::Query);
        };
        let body = inputs
            .content
            .radius(movement.state().pose)
            .ok_or(Fault::Dependency)?;
        let centre = movement.state().position;
        let floor_row = Self::support_row(
            floor_request,
            floor_answer,
            types::SupportSurface::Floor,
            centre,
            body,
            inputs.room,
        )?;
        let ceiling_row = Self::support_row(
            ceiling_request,
            ceiling_answer,
            types::SupportSurface::Ceiling,
            centre,
            body,
            inputs.room,
        )?;
        let Movement::Work(chosen) = &receipt.chosen else {
            return Err(Fault::Query);
        };
        if chosen.model.0.offered.as_ref() != Some(ceiling_answer) {
            return Err(Fault::Query);
        }
        let requests = Requests {
            width: u8::try_from(inputs.room.width).map_err(|_| Fault::Query)?,
            rows: [floor_row, ceiling_row],
        };
        if requests.width == 0 {
            return Err(Fault::Query);
        }
        Ok((requests.bind(&self.template)?, requests))
    }

    fn support_row(
        request: &samus_movement::suspended::MovementDecision,
        answer: &samus_movement::suspended::MovementDecision,
        surface: types::SupportSurface,
        centre: types::Position,
        body: types::Radius,
        room: &Room,
    ) -> Result<u8, Fault> {
        use samus_movement::suspended::MovementDecision;
        use types::Parts;
        let MovementDecision::SupportRequest {
            bounds,
            surface: requested,
        } = request
        else {
            return Err(Fault::Query);
        };
        let MovementDecision::SupportAnswer { check } = answer else {
            return Err(Fault::Query);
        };
        let edge = match surface {
            types::SupportSurface::Floor => bounds
                .at
                .y
                .pixel()
                .0
                .checked_add(body.y.0)
                .and_then(|pixel| pixel.checked_sub(1)),
            types::SupportSurface::Ceiling => bounds.at.y.pixel().0.checked_sub(body.y.0),
        }
        .ok_or(Fault::Query)?;
        let cell = types::Cell {
            x: bounds.at.x.pixel().0 >> 4,
            y: edge >> 4,
        };
        let index = usize::from(cell.y)
            .checked_mul(usize::from(room.width))
            .and_then(|base| base.checked_add(usize::from(cell.x)))
            .ok_or(Fault::Query)?;
        if *requested != surface
            || check.surface != surface
            || bounds.at != centre
            || bounds.r != body
            || check.centre != bounds.at
            || check.radius != body
            || check.cell != cell
            || bounds.at.x.pixel().0 >= 4096
            || edge >= 4096
            || cell.y == 0
            || cell.x >= room.width
            || index >= 32768
            || check.result != types::SupportResult::NoSlope
            || check.placement.0.0 != 0
            || !matches!(
                room.cells.get(index),
                Some(types::Block::Air | types::Block::Solid)
            )
        {
            return Err(Fault::Query);
        }
        u8::try_from(cell.y).map_err(|_| Fault::Query)
    }
}

impl Stage {
    /// Bind both actual requests to the sole begun compound physical work.
    /// # Errors
    /// Wrong actual work, absent offer or repeated physical binding.
    pub fn bind(&mut self, work: WorkId, requests: Requests) -> Result<(), Fault> {
        let Self::Offered(receipt) = self else {
            return Err(Fault::Dependency);
        };
        if work != TravelWork::Support.identity() {
            return Err(Fault::Dependency);
        }
        *self = Self::Charging {
            receipt: receipt.clone(),
            pending: Pending::new(requests),
        };
        Ok(())
    }

    /// Deliver only this compound owner's actual product bytes in Floor/Ceiling order.
    /// # Errors
    /// Wrong active work, no pending owner, reordered or extra hardware sample.
    pub fn sample(
        &mut self,
        work: WorkId,
        device: data::timing::Sample,
        value: u8,
    ) -> Result<(), Fault> {
        let Self::Charging { pending, .. } = self else {
            return Err(Fault::Dependency);
        };
        pending.sample(work, device, value)
    }

    /// Publish both actual answers atomically only at the complete outer physical return.
    /// # Errors
    /// Replaced current owners, incomplete products or an unexpected completion query/effect.
    pub fn complete(
        &mut self,
        work: WorkId,
        movement: &mut Movement,
        inputs: &Inputs<'_>,
        contract: &PhysicalContract,
    ) -> Result<Completion, Fault> {
        let (_, requests) = contract.physical(self, movement, inputs)?;
        let Self::Charging { receipt, pending } = self else {
            return Err(Fault::Dependency);
        };
        if requests != pending.requests {
            return Err(Fault::Dependency);
        }
        pending.complete(work)?;
        let mut candidate = receipt.chosen.clone();
        let queries = Queries::default();
        let radius = inputs
            .content
            .radius(candidate.state().pose)
            .ok_or(Fault::Dependency)?;
        let body = |_| radius;
        let cannon = |pose| inputs.content.cannon_heading(pose);
        let frame = receipt.inputs.0.frame(&body, &cannon);
        let mut expected_outputs = candidate.outputs().cloned().ok_or(Fault::Dependency)?;
        let Some(samus_movement::suspended::MovementDecision::SupportAnswer { check }) =
            receipt.decisions.0.last()
        else {
            return Err(Fault::Query);
        };
        expected_outputs
            .movement_work
            .push(types::MovementWork::Support(check.clone()));
        candidate
            .complete(inputs.content, inputs.rules, &frame, &queries)
            .map_err(Fault::Movement)?;
        if queries.fault.get()
            || candidate.state() != movement.state()
            || candidate.outputs() != Some(&expected_outputs)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actual_movement_rules_shape_stays_pinned() {
        assert_eq!(canonical::model::pin_holds::<MovementRulesPin>(), Ok(()));
    }

    #[test]
    fn actual_actor_completion_shape_stays_pinned() {
        assert_eq!(canonical::model::pin_holds::<ActorCompletionPin>(), Ok(()));
    }

    #[test]
    fn whole_support_requires_two_distinct_live_tasks_after_restore() {
        use data::timing::Sample;
        let work = WorkId(1001);
        let mut pending = Pending::new(Requests {
            width: 7,
            rows: [5, 3],
        });
        assert_eq!(
            pending.sample(WorkId(1002), Sample::ProductLow, 35),
            Err(Fault::Samples)
        );
        pending.sample(work, Sample::ProductLow, 35).unwrap();
        pending.sample(work, Sample::ProductHigh, 0).unwrap();
        assert_eq!(pending.complete(work), Err(Fault::Samples));
        let bytes = canonical::to_bytes(&pending);
        let mut pending: Pending = canonical::from_bytes(&bytes).unwrap();
        assert_eq!(
            pending.sample(work, Sample::ProductHigh, 0),
            Err(Fault::Samples)
        );
        pending.sample(work, Sample::ProductLow, 35).unwrap();
        pending.sample(work, Sample::ProductHigh, 0).unwrap();
        assert_eq!(pending.complete(work), Err(Fault::Samples));
    }

    #[test]
    fn complete_outer_work_accepts_both_products_and_refuses_a_third_task() {
        use data::timing::Sample;
        let work = WorkId(1001);
        let mut pending = Pending::new(Requests {
            width: 7,
            rows: [5, 3],
        });
        for product in [35_u16, 21] {
            let [low, high] = product.to_le_bytes();
            pending.sample(work, Sample::ProductLow, low).unwrap();
            pending.sample(work, Sample::ProductHigh, high).unwrap();
        }
        assert_eq!(pending.complete(work), Ok(()));
        assert_eq!(pending.complete(WorkId(1002)), Err(Fault::Samples));
        let complete = pending.clone();
        assert_eq!(
            pending.sample(work, Sample::ProductLow, 21),
            Err(Fault::Samples)
        );
        assert_eq!(pending, complete);
    }
}
