//! Movement adaptation uses imported content and preserves every clean output.

use super::{MovementContent, ProductionFault, Unsupported};
use core::cell::Cell;

pub(super) fn with_frame<R>(
    current_pose: types::Pose,
    content: &MovementContent,
    frame: &samus_movement::Frame<'_>,
    operation: impl FnOnce(&samus_movement::Frame<'_>) -> Result<R, ProductionFault>,
) -> Result<R, ProductionFault> {
    if frame.elevator_moving && current_pose.facing == types::Facing::Screen {
        return Err(ProductionFault::Unsupported(Unsupported::ElevatorBody));
    }
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
    let admitted = samus_movement::Frame {
        body: &body,
        cannon_heading: &heading,
        ..*frame
    };
    let value = operation(&admitted)?;
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
        let outcome = samus
            .step(admitted, world, rules)
            .map_err(ProductionFault::Movement)?;
        Ok(outcome)
    })
}
