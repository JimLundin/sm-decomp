//! Movement adaptation uses imported content and preserves every clean output.

use super::{MovementContent, ProductionFault, Unsupported};
use core::cell::Cell;

pub(super) fn advance(
    samus: &mut samus_movement::Samus,
    content: &MovementContent,
    rules: &samus_movement::Rules,
    frame: &samus_movement::Frame<'_>,
    world: &dyn types::World,
) -> Result<samus_movement::Outcome, ProductionFault> {
    if frame.elevator_moving && samus.pose.facing == types::Facing::Screen {
        return Err(ProductionFault::Unsupported(Unsupported::ElevatorBody));
    }
    let initial = content
        .pose(samus.pose)
        .ok_or(ProductionFault::Unsupported(Unsupported::PoseContent(
            samus.pose,
        )))?;
    let missing = Cell::new(None);
    let body = |pose| {
        content.pose(pose).map_or_else(
            || {
                missing.set(Some(pose));
                // This candidate is discarded; no unavailable body is published or retained.
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
    let outcome = samus
        .step(&admitted, world, rules)
        .map_err(ProductionFault::Movement)?;
    if let Some(pose) = missing.get() {
        return Err(ProductionFault::Unsupported(Unsupported::PoseContent(pose)));
    }
    Ok(outcome)
}
