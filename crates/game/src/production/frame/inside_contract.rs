//! Whole Inside physical admission from actual current geometry and retained decisions.

use super::{
    inside_world::InsideWorld,
    movement::Movement,
    movement_stage::{Group, Receipt},
};
use crate::production::{MovementContent, timing::WorkId};
use canonical::Canonical;
use data::timing::{DeviceWrite, Publication, Sample, Template, TemplateEffect};
use samus_movement::suspended::MovementDecision;
use types::Parts;

#[cfg(test)]
mod tests;

/// Validation-only current geometry required by the accepted whole physical owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequiredDependencies {
    /// Required actual current centre pixels, horizontal then vertical.
    pub pixels: [u16; 2],
    /// Required actual current complete pose.
    pub pose: types::Pose,
    /// Required immutable current body's half sizes.
    pub radius: [u16; 2],
    /// Required complete unchanged current room blocks, without an actor roster.
    pub room: Box<InsideWorld>,
}

/// Immutable checked whole-owner content; selected quantities cannot seed model state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalContract {
    dependencies: RequiredDependencies,
    template: Template,
}

/// An unsupported dependency never authorizes any portion of the whole owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Actual current movement/body/room or retained Inside decisions changed.
    Dependencies,
    /// This actual Inside geometry does not reach the admitted three-request shape.
    Geometry,
    /// Missing, extra or reordered physical publications or samples.
    Profile,
    /// Actual pending whole operation identity changed.
    Work,
    /// Missing, duplicate, reordered or unequal live products.
    Samples,
}

impl PhysicalContract {
    /// Retain independently checked source predicates and the complete physical template.
    #[must_use]
    pub const fn new(dependencies: RequiredDependencies, template: Template) -> Self {
        Self {
            dependencies,
            template,
        }
    }

    /// Qualify complete actual current operands before binding one whole Inside operation.
    /// # Errors
    /// Stale or incomplete producers, a different query shape or incompatible physical content.
    pub fn physical(
        &self,
        receipt: &Receipt,
        movement: &Movement,
        content: &MovementContent,
        room: &InsideWorld,
        pose_input: &super::borrowed_player_preparation::PoseInputCompletion,
    ) -> Result<data::timing::Profile, Fault> {
        let Movement::Work(work) = movement else {
            return Err(Fault::Dependencies);
        };
        let [
            MovementDecision::InsideRequest(bounds),
            MovementDecision::InsideAnswer(_),
        ] = receipt.decisions.0.as_slice()
        else {
            return Err(Fault::Dependencies);
        };
        let current = &work.model.0.state;
        let body = content
            .pose(current.pose)
            .ok_or(Fault::Dependencies)?
            .radius;
        if receipt.group != Group::Inside
            || (work.completed.0.len() != 1
                || work.completed.0.first() != receipt.decisions.0.first())
            || work.model.0.offered.as_ref() != receipt.decisions.0.last()
            || current != &work.model.0.start
            || bounds.at != current.position
            || bounds.r != body
            || pose_input.dependency.pose != current.pose
            || pose_input.terminal.radius != body
            || [current.position.x.pixel().0, current.position.y.pixel().0]
                != self.dependencies.pixels
            || current.pose != self.dependencies.pose
            || [body.x.0, body.y.0] != self.dependencies.radius
            || room != self.dependencies.room.as_ref()
        {
            return Err(Fault::Dependencies);
        }
        let requests = Requests::from_receipt(receipt, room.width())?;
        let column = bounds.at.x.pixel().0 >> 4;
        if requests.rows.iter().any(|row| {
            room.block_type(types::Cell {
                x: column,
                y: u16::from(*row),
            }) != types::Block::Air
        }) {
            return Err(Fault::Dependencies);
        }
        requests.bind(&self.template)
    }
}

/// The three actual current room-row requests, independent of source fixture quantities.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requests {
    /// Actual bottom, centre and top rows, in reached query order.
    pub rows: [u8; 3],
    /// Actual current room width in blocks.
    pub width: u8,
}

impl Requests {
    /// Derive exactly the rows reached by the clean Inside request's bounds.
    /// # Errors
    /// Incomplete receipt, a different reached query count or unrepresentable current operands.
    pub fn from_receipt(receipt: &Receipt, width: u16) -> Result<Self, Fault> {
        let [
            MovementDecision::InsideRequest(bounds),
            MovementDecision::InsideAnswer(_),
        ] = receipt.decisions.0.as_slice()
        else {
            return Err(Fault::Geometry);
        };
        let y = bounds.at.y.pixel().0;
        let radius = bounds.r.y.0;
        let bottom_pixel = y
            .checked_add(radius)
            .and_then(|edge| edge.checked_sub(1))
            .ok_or(Fault::Geometry)?;
        let top_pixel = y.checked_sub(radius).ok_or(Fault::Geometry)?;
        if bounds.at.x.pixel().0 >= 4096
            || bottom_pixel >= 4096
            || top_pixel > y
            || y > bottom_pixel
        {
            return Err(Fault::Geometry);
        }
        let bottom = bottom_pixel >> 4;
        let centre = y >> 4;
        let top = top_pixel >> 4;
        if bottom == centre
            || top == bottom
            || top == centre
            || bottom ^ centre ^ top == 0
            || top == 0
            || width == 0
        {
            return Err(Fault::Geometry);
        }
        Ok(Self {
            rows: [
                u8::try_from(bottom).map_err(|_| Fault::Geometry)?,
                u8::try_from(centre).map_err(|_| Fault::Geometry)?,
                u8::try_from(top).map_err(|_| Fault::Geometry)?,
            ],
            width: u8::try_from(width).map_err(|_| Fault::Geometry)?,
        })
    }

    /// Required products are used only to check actual hardware completion.
    #[must_use]
    pub fn products(self) -> [u16; 3] {
        self.rows
            .map(|row| u16::from(row).wrapping_mul(u16::from(self.width)))
    }

    /// Bind all three width/row/zero publication triples and their complete samples once.
    /// # Errors
    /// Any missing, extra, reordered or differently addressed physical effect.
    pub fn bind(self, template: &Template) -> Result<data::timing::Profile, Fault> {
        let mut values = self.rows.into_iter().flat_map(|row| [self.width, row, 0]);
        let mut writes = Vec::new();
        let mut sample_count = 0_usize;
        for run in &template.runs {
            for _ in 0..run.count {
                match run.effect {
                    TemplateEffect::None => {}
                    TemplateEffect::Publish(port) => {
                        let value = values.next().ok_or(Fault::Profile)?;
                        let write = DeviceWrite::from_cpu_port(port.0, value)
                            .map_err(|_| Fault::Profile)?;
                        let slot = writes.len() % 3;
                        if sample_count
                            != writes
                                .len()
                                .checked_div(3)
                                .and_then(|count| count.checked_mul(2))
                                .ok_or(Fault::Profile)?
                            || !matches!(
                                (slot, &write.effect),
                                (0, Publication::Multiplicand(_))
                                    | (1, Publication::Multiplier(_))
                                    | (2, Publication::DividendLow(0))
                            )
                        {
                            return Err(Fault::Profile);
                        }
                        writes.push(write);
                    }
                    TemplateEffect::Sample(sample) => {
                        let expected = if sample_count.is_multiple_of(2) {
                            Sample::ProductLow
                        } else {
                            Sample::ProductHigh
                        };
                        if sample != expected
                            || writes.len()
                                != sample_count
                                    .checked_div(2)
                                    .and_then(|count| count.checked_add(1))
                                    .and_then(|count| count.checked_mul(3))
                                    .ok_or(Fault::Profile)?
                            || sample_count >= 6
                        {
                            return Err(Fault::Profile);
                        }
                        sample_count = sample_count.checked_add(1).ok_or(Fault::Profile)?;
                    }
                }
            }
        }
        if values.next().is_some() || writes.len() != 9 || sample_count != 6 {
            return Err(Fault::Profile);
        }
        template.bind(&writes).map_err(|_| Fault::Profile)
    }
}

/// Live hardware closure retained by the whole Inside operation across suspension.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Pending {
    work: WorkId,
    requests: Requests,
    count: u8,
    bytes: [u8; 6],
}

impl Pending {
    /// Retain actual request operands and the complete ordinary operation identity.
    #[must_use]
    pub const fn new(work: WorkId, requests: Requests) -> Self {
        Self {
            work,
            requests,
            count: 0,
            bytes: [0; 6],
        }
    }

    /// Retain only this whole operation's actual live low/high samples, in order.
    /// # Errors
    /// Wrong work, wrong device or a missing/extra/reordered byte.
    pub fn sample(&mut self, work: WorkId, device: Sample, value: u8) -> Result<(), Fault> {
        if work != self.work {
            return Err(Fault::Work);
        }
        let expected = if self.count.is_multiple_of(2) {
            Sample::ProductLow
        } else {
            Sample::ProductHigh
        };
        if device != expected {
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

    /// Require all three actual products before consuming the retained Inside answer.
    /// # Errors
    /// Wrong whole work, incomplete samples or unequal actual products.
    pub fn complete(&self, work: WorkId) -> Result<(), Fault> {
        if work != self.work {
            return Err(Fault::Work);
        }
        if self.count != 6 {
            return Err(Fault::Samples);
        }
        let [
            first_low,
            first_high,
            second_low,
            second_high,
            third_low,
            third_high,
        ] = self.bytes;
        if [
            u16::from_le_bytes([first_low, first_high]),
            u16::from_le_bytes([second_low, second_high]),
            u16::from_le_bytes([third_low, third_high]),
        ] != self.requests.products()
        {
            return Err(Fault::Samples);
        }
        Ok(())
    }
}
