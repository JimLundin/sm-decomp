//! Movement adaptation uses imported content and preserves every clean output.

use super::{MovementContent, MovementInputs, ProductionFault, Unsupported};
use core::cell::Cell;
use types::{FiringDirection, Pose, Radius};

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
