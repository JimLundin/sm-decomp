//! Movement adaptation uses imported content and preserves every clean output.

use super::{MovementContent, ProductionFault, Unsupported};
use canonical::model::Pinned;
use core::cell::Cell;
use serde::{Deserialize, Serialize};
use types::{Buttons, ClipEnd, FiringDirection, Item, Parity, Pose, Radius, Set, Sprite};

/// Complete owned movement inputs at the next actual opportunity.
/// The caller updates inputs only from their admitted current writers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovementInputs {
    /// Appearance figure captured before movement preparation.
    pub sprite_before_movement: Sprite,
    /// Current mode's cinematic activity.
    pub cinematic_active: bool,
    /// Current committed held controls.
    pub pad: Buttons,
    /// Current committed newly pressed controls.
    pub pressed: Buttons,
    /// Current progress equipment output.
    pub items: Set<Item>,
    /// Current completed spin-animation query.
    pub spin_turned_far: bool,
    /// Current elevator producer output.
    pub elevator_moving: bool,
    /// Current frame parity.
    pub parity: Parity,
    /// Actual completed frame-start animation query.
    pub animation_ends: Option<ClipEnd>,
}

/// Owned current movement input schema, including independently captured appearance facts.
pub struct MovementInputsPin;
impl Pinned for MovementInputsPin {
    type State = MovementInputs;
    const NAME: &'static str = "production-frame-movement-inputs";
    const SHAPE: u64 = 0x1073_CEEA_7B2B_8D9B;
}

impl MovementInputs {
    /// Borrow the current inputs with actual imported content adapters.
    /// No gameplay decision or query is performed by borrowing them.
    #[must_use]
    pub const fn frame<'a>(
        &self,
        body: &'a dyn Fn(Pose) -> Radius,
        cannon_heading: &'a dyn Fn(Pose) -> Option<FiringDirection>,
    ) -> samus_movement::Frame<'a> {
        samus_movement::Frame {
            sprite_before_movement: self.sprite_before_movement,
            cinematic_active: self.cinematic_active,
            pad: self.pad,
            pressed: self.pressed,
            items: self.items,
            spin_turned_far: self.spin_turned_far,
            elevator_moving: self.elevator_moving,
            parity: self.parity,
            animation_ends: self.animation_ends,
            body,
            cannon_heading,
        }
    }
}

/// Actual playing pose and sprite, independently retained before later pose selection.
pub struct FigurePin;
impl Pinned for FigurePin {
    type State = samus_appearance::Figure;
    const NAME: &'static str = "production-frame-playing-figure";
    const SHAPE: u64 = 0x4301_5F68_4F3E_58BA;
}

pub(super) fn with_frame<R>(
    current_pose: types::Pose,
    content: &MovementContent,
    frame: &samus_movement::Frame<'_>,
    operation: impl FnOnce(&samus_movement::Frame<'_>) -> Result<R, ProductionFault>,
) -> Result<R, ProductionFault> {
    check_elevator(current_pose, frame.elevator_moving)?;
    with_pose_content(current_pose, content, |body, cannon_heading| {
        operation(&samus_movement::Frame {
            body,
            cannon_heading,
            ..*frame
        })
    })
}

fn check_elevator(pose: Pose, elevator_moving: bool) -> Result<(), ProductionFault> {
    if elevator_moving && pose.facing == types::Facing::Screen {
        return Err(ProductionFault::Unsupported(Unsupported::ElevatorBody));
    }
    Ok(())
}

fn with_pose_content<R>(
    current_pose: Pose,
    content: &MovementContent,
    operation: impl FnOnce(
        &dyn Fn(Pose) -> Radius,
        &dyn Fn(Pose) -> Option<FiringDirection>,
    ) -> Result<R, ProductionFault>,
) -> Result<R, ProductionFault> {
    let initial = content
        .pose(current_pose)
        .ok_or(ProductionFault::Unsupported(Unsupported::PoseContent(
            current_pose,
        )))?;
    let missing = Cell::new(None);
    let body = |pose| {
        content.pose(pose).map_or_else(
            || {
                missing.set(Some(pose));
                // This private candidate is discarded; unavailable content is never published.
                initial.radius
            },
            |entry| entry.radius,
        )
    };
    let heading = |pose| content.pose(pose).and_then(|entry| entry.cannon_heading);
    let value = operation(&body, &heading)?;
    if let Some(pose) = missing.get() {
        return Err(ProductionFault::Unsupported(Unsupported::PoseContent(pose)));
    }
    Ok(value)
}

pub(super) fn advance(
    samus: &mut samus_movement::Samus,
    content: &MovementContent,
    rules: &samus_movement::Rules,
    frame: &samus_movement::Frame<'_>,
    world: &dyn types::World,
) -> Result<samus_movement::Outcome, ProductionFault> {
    with_frame(samus.pose, content, frame, |admitted| {
        step(samus, admitted, world, rules)
    })
}

pub(super) fn advance_scoped(
    samus: &mut samus_movement::Samus,
    content: &MovementContent,
    rules: &samus_movement::Rules,
    inputs: &MovementInputs,
    world: &dyn types::World,
) -> Result<samus_movement::Outcome, ProductionFault> {
    let pose = samus.pose;
    with_pose_content(pose, content, |body, heading| {
        check_elevator(pose, inputs.elevator_moving)?;
        step(samus, &inputs.frame(body, heading), world, rules)
    })
}

fn step(
    samus: &mut samus_movement::Samus,
    admitted: &samus_movement::Frame<'_>,
    world: &dyn types::World,
    rules: &samus_movement::Rules,
) -> Result<samus_movement::Outcome, ProductionFault> {
    let outcome = samus
        .step(admitted, world, rules)
        .map_err(ProductionFault::Movement)?;
    Ok(outcome)
}
