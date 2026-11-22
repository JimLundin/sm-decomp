//! Immutable semantic predicates admitting one complete physical follow.

use super::{Fault, Inputs, Receipt, Screen};
use crate::state::owned::CameraPin;
use canonical::model::Model;
use types::Parts;

/// A selected centre requirement, used only to validate actual movement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequiredCentre {
    /// Required whole pixels, horizontal then vertical.
    pub pixels: [u16; 2],
    /// Required fractions, horizontal then vertical.
    pub fractions: [u16; 2],
}

/// Selected initial viewport requirements, never a Camera seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequiredView {
    /// Initial horizontal pixel.
    pub x: u16,
    /// Initial horizontal fraction.
    pub fraction: u16,
    /// Initial vertical pixel.
    pub y: u16,
}

/// A semantic room screen pair required by the physical follow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequiredScreens {
    /// Linear screen cell within the complete current room run.
    pub cell: usize,
    /// Current cell and the next cell's permissions.
    pub screens: [Screen; 2],
}

/// Complete selected dependencies; these quantities cannot construct model state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequiredDependencies {
    /// Actual completed centre requirements.
    pub current: RequiredCentre,
    /// Pre-movement requirements, using the completed placement's previous Y pixel.
    pub previous: RequiredCentre,
    /// Initial viewport requirements.
    pub view: RequiredView,
    /// Complete unchanged neighbouring inputs required at this opportunity.
    pub inputs: Box<Inputs>,
    /// Selected current screen pairs read by this whole physical follow.
    pub screens: Vec<RequiredScreens>,
}

/// Checked immutable physical content, interpreted entirely by production.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalContract {
    dependencies: RequiredDependencies,
    further: Vec<RequiredDependencies>,
    pub(super) template: data::timing::Template,
}

impl PhysicalContract {
    /// Retain normalized semantic predicates and the checked whole owner template.
    #[must_use]
    pub const fn new(dependencies: RequiredDependencies, template: data::timing::Template) -> Self {
        Self {
            dependencies,
            further: Vec::new(),
            template,
        }
    }

    pub(in crate::production::frame) fn with_dependencies(
        &self,
        dependencies: RequiredDependencies,
    ) -> Self {
        Self::new(dependencies, self.template.clone())
    }

    pub(in crate::production::frame) fn admit(&mut self, dependencies: RequiredDependencies) {
        self.further.push(dependencies);
    }

    pub(super) fn check(&self, receipt: &Receipt, result: &Model<CameraPin>) -> Result<(), Fault> {
        let first = self.dependencies.check(receipt, result);
        if first.is_err()
            && self
                .further
                .iter()
                .any(|current| current.check(receipt, result).is_ok())
        {
            return Ok(());
        }
        first
    }
}

impl RequiredDependencies {
    pub(super) fn check(&self, receipt: &Receipt, result: &Model<CameraPin>) -> Result<(), Fault> {
        let (samus, outputs) = receipt.movement.finished().ok_or(Fault::Movement)?;
        let previous = receipt.initial.0.seen();
        let view = receipt.initial.0.position();
        let current = RequiredCentre {
            pixels: [samus.position.x.pixel().0, samus.position.y.pixel().0],
            fractions: [samus.position.x.sub(), samus.position.y.sub()],
        };
        let previous_required = RequiredCentre {
            pixels: [
                previous.x.pixel().0,
                outputs
                    .placed
                    .map_or_else(|| previous.y.pixel().0, |placed| placed.pixel().0),
            ],
            fractions: [previous.x.sub(), previous.y.sub()],
        };
        if current != self.current {
            return Err(Fault::CurrentCentre);
        }
        if previous_required != self.previous {
            return Err(Fault::PreviousCentre);
        }
        if (RequiredView {
            x: view.x.pixel().0,
            fraction: view.x.sub(),
            y: view.y.pixel().0,
        }) != self.view
            || receipt.inputs != *self.inputs
            || samus.pose.action != types::Action::Running
            || samus.pose.facing != types::Facing::Right
            || samus.run.carry != samus_movement::Carry::FollowsPad
            || outputs.skidding
            || samus.position.y.pixel() != previous.y.pixel()
            || result.0.position().y != view.y
            || result.0.seen() != samus.position
        {
            return Err(Fault::Dependencies);
        }
        let [width, _] = receipt.inputs.room.size;
        let [_, row] = view.y.pixel().0.to_le_bytes();
        let [_, column] = result.0.position().x.pixel().0.to_le_bytes();
        let cell = usize::from(row)
            .checked_mul(usize::from(width))
            .and_then(|row| row.checked_add(usize::from(column)))
            .ok_or(Fault::Dependencies)?;
        for required in &self.screens {
            if required.cell != cell && Some(required.cell) != cell.checked_add(usize::from(width))
            {
                return Err(Fault::Dependencies);
            }
            let end = required.cell.checked_add(2).ok_or(Fault::Dependencies)?;
            if receipt.inputs.room.screens.get(required.cell..end)
                != Some(required.screens.as_slice())
            {
                return Err(Fault::Dependencies);
            }
        }
        Ok(())
    }
}
