//! Active arithmetic closure for fully qualified actual paired movement owners.

use super::movement_caller::PairOwner;
use crate::production::timing::WorkId;
use canonical::Canonical;

/// Incompatible actual paired owner, geometry or live arithmetic closure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Actual current dependency or physical identity changed.
    Dependency,
    /// Missing, reordered or unequal live arithmetic samples.
    Samples,
    /// Actual paired movement opportunity rejected before publication.
    Movement(super::movement_caller::Fault),
}

/// Independently admitted current source predicates; none supplies a selected result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceAdmission {
    /// Complete unchanged independently decoded current room and permissions.
    pub room: super::movement_caller::Room,
    /// Source plain-air classification, distinct from other decoded air behaviours.
    pub plain_air: Vec<bool>,
    /// Actual completed raw interactive identity mapped by the accepted source owner.
    pub raw_actor: super::borrowed_preparation::ActorSlotIdentity,
    /// Separately proved current support-alignment admission.
    pub support_alignment: bool,
}

/// One whole physical paired owner qualified by actual current geometry and answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalContract {
    owner: PairOwner,
    admission: SourceAdmission,
    template: data::timing::Template,
    fractional_carry: Option<data::timing::Template>,
    two_candidate: Option<data::timing::Template>,
    two_candidate_carry: Option<data::timing::Template>,
}

pub(super) fn ordered_misses(search: &types::EnemySearch, boxes: &[types::Hitbox]) -> bool {
    matches!(boxes.len(), 1 | 2)
        && search.supplied_count.0 == boxes.len()
        && search.candidates.len() == boxes.len()
        && search
            .candidates
            .iter()
            .zip(boxes)
            .enumerate()
            .all(|(ordinal, (candidate, actor))| {
                candidate.candidate.0 == ordinal
                    && candidate.result == types::CandidateResult::Miss
                    && candidate.centre == actor.centre
                    && candidate.radius == actor.radius
            })
}

pub(super) fn horizontal_misses(
    search: &types::EnemySearch,
    boxes: &[types::Hitbox],
    tested_x: u16,
    radius_x: u16,
) -> bool {
    ordered_misses(search, boxes)
        && boxes.iter().all(|actor| {
            let distance = if boxes.len() == 1 {
                actor.centre.x.0.checked_sub(tested_x)
            } else {
                Some(actor.centre.x.0.abs_diff(tested_x))
            };
            distance.is_some_and(|distance| {
                distance < 32768 && distance.saturating_sub(actor.radius.x.0) >= radius_x
            })
        })
}

impl PhysicalContract {
    /// Retain separately gated current source predicates and the complete owner envelope.
    #[must_use]
    pub const fn new(
        owner: PairOwner,
        admission: SourceAdmission,
        template: data::timing::Template,
    ) -> Self {
        Self {
            owner,
            admission,
            template,
            fractional_carry: None,
            two_candidate: None,
            two_candidate_carry: None,
        }
    }

    /// Admit the qualified ordinary-right physical envelope when the X fraction carries.
    pub fn set_fractional_carry(&mut self, template: data::timing::Template) {
        self.fractional_carry = Some(template);
    }

    /// Admit separately qualified traversal of both tangible slots.
    pub fn set_two_candidate_profiles(
        &mut self,
        no_carry: data::timing::Template,
        carry: Option<data::timing::Template>,
    ) {
        self.two_candidate = Some(no_carry);
        self.two_candidate_carry = carry;
    }

    pub(super) fn plain_air_room(&self) -> (&super::movement_caller::Room, &[bool]) {
        (&self.admission.room, &self.admission.plain_air)
    }

    /// Qualify the actual request/answer and bind its complete live arithmetic work.
    /// # Errors
    /// Replaced current geometry/owner, different query arms or incompatible physical effects.
    pub fn physical(
        &self,
        receipt: &super::movement_caller::PairReceipt,
        movement: &super::movement::Movement,
        room: &super::movement_caller::Room,
        actors: &super::borrowed_player_preparation::ActorUpdateCompletion,
        content: &crate::production::MovementContent,
    ) -> Result<(data::timing::Profile, PairRequests), Fault> {
        use samus_movement::suspended::MovementDecision;
        let boxes = actors.solid_boxes().map_err(|_| Fault::Dependency)?;
        let world = crate::production::world::MovementWorld::new(
            room.width,
            room.cells.clone(),
            room.permissions.clone(),
            boxes.clone(),
            room.boundary,
        )
        .map_err(|_| Fault::Dependency)?;
        if receipt.owner != self.owner
            || room != &self.admission.room
            || self.admission.plain_air.len() != room.cells.len()
            || movement != &receipt.retained
            || world != receipt.world.0
            || receipt.before.state() != movement.state()
            || !actors
                .matches_collision_roster(self.admission.raw_actor, self.two_candidate.is_some())
            || actors.dependency.clocks.simulation
                != super::borrowed_player_preparation::ClockDisposition::Running
            || actors.dependency.clocks.enemy
                != super::borrowed_player_preparation::ClockDisposition::Running
        {
            return Err(Fault::Dependency);
        }
        let body = content
            .radius(receipt.before.state().pose)
            .ok_or(Fault::Dependency)?;
        let (row, template) = match receipt.decisions.0.as_slice() {
            [
                MovementDecision::MotionRequest(request),
                returned_answer @ MovementDecision::MotionAnswer {
                    request: returned, ..
                },
            ] if matches!(self.owner, PairOwner::OrdinaryRight | PairOwner::AimRight)
                && request == returned =>
            {
                self.right(receipt, request, returned_answer, &boxes, body)?
            }
            [
                request @ MovementDecision::SupportRequest { .. },
                answer @ MovementDecision::SupportAnswer { .. },
            ] if matches!(
                self.owner,
                PairOwner::FloorSupport | PairOwner::CeilingSupport
            ) =>
            {
                (
                    self.support(receipt, request, answer, body)?,
                    &self.template,
                )
            }
            _ => return Err(Fault::Dependency),
        };
        let requests = PairRequests {
            owner: self.owner,
            row: u8::try_from(row).map_err(|_| Fault::Dependency)?,
            width: u8::try_from(room.width).map_err(|_| Fault::Dependency)?,
        };
        Ok((requests.bind(template)?, requests))
    }

    fn index(&self, cell: types::Cell) -> Result<usize, Fault> {
        if cell.x >= self.admission.room.width {
            return Err(Fault::Dependency);
        }
        let index = usize::from(cell.y)
            .checked_mul(usize::from(self.admission.room.width))
            .and_then(|base| base.checked_add(usize::from(cell.x)))
            .ok_or(Fault::Dependency)?;
        if index >= 32768 || index >= self.admission.room.cells.len() {
            return Err(Fault::Dependency);
        }
        Ok(index)
    }

    fn support(
        &self,
        receipt: &super::movement_caller::PairReceipt,
        request: &samus_movement::suspended::MovementDecision,
        answer: &samus_movement::suspended::MovementDecision,
        body: types::Radius,
    ) -> Result<u16, Fault> {
        use samus_movement::suspended::MovementDecision;
        use types::Parts;
        let MovementDecision::SupportRequest { bounds, surface } = request else {
            return Err(Fault::Dependency);
        };
        let MovementDecision::SupportAnswer { check } = answer else {
            return Err(Fault::Dependency);
        };
        let expected_surface = match self.owner {
            PairOwner::FloorSupport => types::SupportSurface::Floor,
            PairOwner::CeilingSupport => types::SupportSurface::Ceiling,
            PairOwner::OrdinaryRight | PairOwner::GroundFollowingDown | PairOwner::AimRight => {
                return Err(Fault::Dependency);
            }
        };
        let y = bounds.at.y.pixel().0;
        let edge = match surface {
            types::SupportSurface::Floor => y
                .checked_add(body.y.0)
                .and_then(|pixel| pixel.checked_sub(1)),
            types::SupportSurface::Ceiling => y.checked_sub(body.y.0),
        }
        .ok_or(Fault::Dependency)?;
        let cell = types::Cell {
            x: bounds.at.x.pixel().0 >> 4,
            y: edge >> 4,
        };
        let index = self.index(cell)?;
        if !self.admission.support_alignment
            || bounds.at != receipt.before.state().position
            || bounds.r != body
            || *surface != expected_surface
            || check.surface != *surface
            || check.centre != bounds.at
            || check.radius != body
            || check.cell != cell
            || edge >= 4096
            || cell.y == 0
            || check.result != types::SupportResult::NoSlope
            || check.placement.0.0 != 0
            || !matches!(
                self.admission.room.cells.get(index),
                Some(types::Block::Air | types::Block::Solid)
            )
        {
            return Err(Fault::Dependency);
        }
        Ok(cell.y)
    }

    fn right(
        &self,
        receipt: &super::movement_caller::PairReceipt,
        request: &samus_movement::suspended::MotionRequest,
        decision: &samus_movement::suspended::MovementDecision,
        boxes: &[types::Hitbox],
        body: types::Radius,
    ) -> Result<(u16, &data::timing::Template), Fault> {
        use types::Parts;
        let samus_movement::suspended::MovementDecision::MotionAnswer {
            answer, outputs, ..
        } = decision
        else {
            return Err(Fault::Dependency);
        };
        let [types::MovementWork::Motion(attempt)] = outputs.movement_work.as_slice() else {
            return Err(Fault::Dependency);
        };
        let search = attempt.enemy_search.as_ref().ok_or(Fault::Dependency)?;

        let purpose = match self.owner {
            PairOwner::OrdinaryRight => types::CollisionPurpose::OrdinaryMove,
            PairOwner::AimRight
                if request.distance.pixel().0 == 1 && request.distance.sub() == 0 =>
            {
                types::CollisionPurpose::AimChange
            }
            PairOwner::FloorSupport
            | PairOwner::CeilingSupport
            | PairOwner::GroundFollowingDown
            | PairOwner::AimRight => return Err(Fault::Dependency),
        };
        let (remainder, carry) = request
            .bounds
            .at
            .x
            .sub()
            .overflowing_add(request.distance.sub());
        let template = match (boxes, carry) {
            ([_], false) => &self.template,
            ([_], true) if self.owner == PairOwner::OrdinaryRight => {
                self.fractional_carry.as_ref().ok_or(Fault::Dependency)?
            }
            ([_, _], false) => self.two_candidate.as_ref().ok_or(Fault::Dependency)?,
            ([_, _], true) => self.two_candidate_carry.as_ref().ok_or(Fault::Dependency)?,
            _ => return Err(Fault::Dependency),
        };
        let integer_target = request
            .bounds
            .at
            .x
            .pixel()
            .0
            .checked_add(request.distance.pixel().0)
            .and_then(|pixel| pixel.checked_add(u16::from(carry)))
            .ok_or(Fault::Dependency)?;
        let tested_x = integer_target
            .checked_add(u16::from(remainder != 0))
            .ok_or(Fault::Dependency)?;
        if request.bounds.at != receipt.before.state().position
            || request.bounds.r != body
            || request.direction != types::Direction::Right
            || request.purpose != purpose
            || request.distance.negative()
            || request.distance.0 == 0
            || request.distance.pixel().0 >= 16
            || remainder == 0
            || !horizontal_misses(search, boxes, tested_x, body.x.0)
            || attempt.start != request.bounds.at
            || attempt.radius != body
            || attempt.direction != request.direction
            || attempt.purpose != request.purpose
            || attempt.world != types::RequestedWorld::EnemiesThenBlocks
            || attempt.intended.0 != request.distance
            || attempt.result != types::MotionResult::Unobstructed
            || attempt.effect
                != types::RequestEffect::Travel(types::TravelDistance(request.distance))
            || answer.stop.is_some()
            || answer.accepted.0 != request.distance
            || answer.to.x.0 != request.bounds.at.x.0.wrapping_add(request.distance.0)
            || answer.to.y != request.bounds.at.y
            || outputs.floor_response.is_some()
            || outputs.pending_aim.is_some()
        {
            return Err(Fault::Dependency);
        }
        Ok((
            self.right_terrain(receipt, request, outputs, body, integer_target)?,
            template,
        ))
    }

    fn right_terrain(
        &self,
        receipt: &super::movement_caller::PairReceipt,
        request: &samus_movement::suspended::MotionRequest,
        outputs: &samus_movement::suspended::DecisionOutputs,
        body: types::Radius,
        integer_target: u16,
    ) -> Result<u16, Fault> {
        use types::Parts;
        let top = request
            .bounds
            .at
            .y
            .pixel()
            .0
            .checked_sub(body.y.0)
            .ok_or(Fault::Dependency)?;
        let bottom = request
            .bounds
            .at
            .y
            .pixel()
            .0
            .checked_add(body.y.0)
            .and_then(|pixel| pixel.checked_sub(1))
            .ok_or(Fault::Dependency)?;
        let leading = integer_target
            .checked_add(body.x.0)
            .and_then(|pixel| pixel.checked_sub(1))
            .ok_or(Fault::Dependency)?;
        if top > bottom
            || bottom >= 4096
            || leading >= 4096
            || (bottom >> 4).checked_sub(top >> 4) != Some(2)
            || outputs.collision_tests.len() != 3
            || outputs.reached.len() != 3
        {
            return Err(Fault::Dependency);
        }
        for (ordinal, (test, reached)) in outputs
            .collision_tests
            .iter()
            .zip(&outputs.reached)
            .enumerate()
        {
            let cell = types::Cell {
                x: leading >> 4,
                y: (top >> 4)
                    .checked_add(u16::try_from(ordinal).map_err(|_| Fault::Dependency)?)
                    .ok_or(Fault::Dependency)?,
            };
            let index = self.index(cell)?;
            if test.cell != cell
                || test.direction != request.direction
                || test.purpose != request.purpose
                || test.vertical_motion != Some(types::VerticalMotionAtTest::ExactlyZero)
                || receipt.before.state().vertical.speed.0 != 0
                || reached.cell != cell
                || reached.way != types::Way::Move(types::Direction::Right)
                || self.admission.room.cells.get(index) != Some(&types::Block::Air)
                || self.admission.plain_air.get(index) != Some(&true)
            {
                return Err(Fault::Dependency);
            }
        }
        Ok(top >> 4)
    }
}

/// Actual geometric multiply request for one complete paired physical owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PairRequests {
    /// Exact paired semantic owner.
    owner: PairOwner,
    /// Actual first queried cell row.
    row: u8,
    /// Actual admitted complete room width.
    width: u8,
}

impl PairRequests {
    /// Bind the complete owner with its exact publication and sample sequence.
    /// # Errors
    /// Missing, extra, reordered or incompatible arithmetic effects.
    pub fn bind(self, template: &data::timing::Template) -> Result<data::timing::Profile, Fault> {
        use data::timing::{DeviceWrite, Publication, Sample, TemplateEffect};
        let values = match self.owner {
            PairOwner::OrdinaryRight | PairOwner::AimRight | PairOwner::GroundFollowingDown => {
                vec![self.row, self.width]
            }
            PairOwner::FloorSupport | PairOwner::CeilingSupport => vec![self.width, self.row, 0],
        };
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
pub struct PairPending {
    work: WorkId,
    requests: PairRequests,
    count: u8,
    bytes: [u8; 2],
}

impl PairPending {
    /// Retain actual derived request operands; source products never initialize these bytes.
    /// # Errors
    /// The requests belong to another actual owner or have an invalid room width.
    pub fn new(work: WorkId, requests: PairRequests) -> Result<Self, Fault> {
        if work != requests.owner.identity() || requests.width == 0 {
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

/// A paired owner with its mandatory live arithmetic closure.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Actual request and returned answer await full source qualification.
    Offered {
        /// Actual once-queried movement opportunity.
        pair: super::movement_caller::Pair,
        /// Immutable actual rules used at this opportunity.
        rules: canonical::model::Model<super::movement_support_contract::MovementRulesPin>,
    },
    /// Exact physical work and its active arithmetic closure remain pending.
    Charging {
        /// Actual once-queried movement opportunity.
        pair: super::movement_caller::Pair,
        /// Live samples for this exact whole operation.
        pending: PairPending,
        /// Immutable actual rules used at this opportunity.
        rules: canonical::model::Model<super::movement_support_contract::MovementRulesPin>,
    },
    /// Both the hardware product and retained movement answer completed once.
    Complete(super::movement_caller::PairReceipt),
}

/// Fresh actual providers and immutable qualification borrowed at physical completion.
pub struct CompletionInputs<'a> {
    /// Actual current completed actor and control owners.
    pub current: &'a super::movement_caller::Current<'a>,
    /// Actual unchanged admitted room.
    pub room: &'a super::movement_caller::Room,
    /// Actual checked body/cannon content.
    pub content: &'a crate::production::MovementContent,
    /// Actual current rule profile.
    pub rules: &'a samus_movement::Rules,
    /// Fixed immutable full physical predicate contract.
    pub contract: &'a PhysicalContract,
}

impl Stage {
    /// Retain only an actual offered paired movement opportunity.
    /// # Errors
    /// The supplied paired operation is absent, already bound or completed.
    pub fn offered(
        pair: super::movement_caller::Pair,
        rules: &samus_movement::Rules,
    ) -> Result<Self, Fault> {
        if !matches!(&pair, super::movement_caller::Pair::Offered(_)) {
            return Err(Fault::Dependency);
        }
        Ok(Self::Offered {
            pair,
            rules: canonical::model::Model(rules.clone()),
        })
    }

    /// Full actual current request/answer evidence for immutable contract revalidation.
    #[must_use]
    pub const fn receipt(&self) -> Option<&super::movement_caller::PairReceipt> {
        match self {
            Self::Offered { pair, .. } | Self::Charging { pair, .. } => pair.receipt(),
            Self::Complete(_) => None,
        }
    }

    /// Bind a fully qualified actual operation and its derived math request together.
    /// # Errors
    /// Missing offered operation, different work identity or replaced derived owner.
    pub fn bind(&mut self, work: WorkId, requests: PairRequests) -> Result<(), Fault> {
        let Self::Offered { pair, rules } = self else {
            return Err(Fault::Dependency);
        };
        if pair
            .receipt()
            .is_none_or(|receipt| receipt.owner != requests.owner)
        {
            return Err(Fault::Dependency);
        }
        let pending = PairPending::new(work, requests)?;
        let mut candidate = pair.clone();
        candidate.bind(work).map_err(Fault::Movement)?;
        *self = Self::Charging {
            pair: candidate,
            pending,
            rules: rules.clone(),
        };
        Ok(())
    }

    /// Reject replacement of the actual rule profile before each physical advance.
    /// # Errors
    /// Current rules differ from the immutable actual opportunity snapshot.
    pub fn validate_rules(&self, current: &samus_movement::Rules) -> Result<(), Fault> {
        match self {
            Self::Offered { rules, .. } | Self::Charging { rules, .. } if &rules.0 != current => {
                Err(Fault::Dependency)
            }
            Self::Offered { .. } | Self::Charging { .. } | Self::Complete(_) => Ok(()),
        }
    }

    /// Deliver only the active owner's actual hardware result bytes.
    /// # Errors
    /// No charging owner or incompatible exact identity/device/order.
    pub fn sample(
        &mut self,
        work: WorkId,
        sample: data::timing::Sample,
        value: u8,
    ) -> Result<(), Fault> {
        let Self::Charging { pending, .. } = self else {
            return Err(Fault::Dependency);
        };
        pending.sample(work, sample, value)
    }

    /// Publish the retained answer only after this owner's complete actual product.
    /// # Errors
    /// Wrong work, missing/unequal samples or changed current movement/provider premises.
    pub fn complete(
        &mut self,
        work: WorkId,
        movement: &mut super::movement::Movement,
        inputs: &CompletionInputs<'_>,
    ) -> Result<super::movement_caller::PairReceipt, Fault> {
        self.validate_rules(inputs.rules)?;
        let Self::Charging { pair, pending, .. } = self else {
            return Err(Fault::Dependency);
        };
        let receipt = pair.receipt().ok_or(Fault::Dependency)?;
        let (_, requests) = inputs.contract.physical(
            receipt,
            movement,
            inputs.room,
            inputs.current.actors,
            inputs.content,
        )?;
        if requests != pending.requests {
            return Err(Fault::Dependency);
        }
        pending.complete(work)?;
        let mut candidate_pair = pair.clone();
        let mut candidate_movement = movement.clone();
        let receipt = candidate_pair
            .complete(
                work,
                &mut candidate_movement,
                inputs.current,
                inputs.room,
                inputs.content,
                inputs.rules,
            )
            .map_err(Fault::Movement)?;
        *movement = candidate_movement;
        *self = Self::Complete(receipt.clone());
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data::timing::Sample;

    #[test]
    fn held_product_requires_both_actual_bytes_and_exact_work_after_restore() {
        let work = PairOwner::OrdinaryRight.identity();
        let mut pending = PairPending::new(
            work,
            PairRequests {
                owner: PairOwner::OrdinaryRight,
                row: 3,
                width: 7,
            },
        )
        .unwrap();
        assert_eq!(pending.complete(work), Err(Fault::Samples));
        let untouched = pending.clone();
        assert_eq!(
            pending.sample(WorkId(1001), Sample::ProductLow, 21),
            Err(Fault::Samples)
        );
        assert_eq!(pending, untouched);
        pending.sample(work, Sample::ProductLow, 21).unwrap();
        let bytes = canonical::to_bytes(&pending);
        let mut pending: PairPending = canonical::from_bytes(&bytes).unwrap();
        assert_eq!(pending.complete(work), Err(Fault::Samples));
        pending.sample(work, Sample::ProductHigh, 1).unwrap();
        assert_eq!(pending.complete(work), Err(Fault::Samples));
        assert_eq!(pending.complete(WorkId(1001)), Err(Fault::Samples));
    }

    #[test]
    fn actual_product_checks_full_width_and_rejects_extra_samples() {
        let work = PairOwner::CeilingSupport.identity();
        let mut pending = PairPending::new(
            work,
            PairRequests {
                owner: PairOwner::CeilingSupport,
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
