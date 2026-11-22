//! Following from actual completed movement and the retained prior centre.

use super::{
    connected::{
        Fault,
        inputs::{CameraInputs, Lead, Screen},
    },
    movement_work::Movement,
};
use core::num::Wrapping;
use types::Parts;

/// The shared clean Camera entry; callers qualify their own semantic or physical opportunity.
pub(super) fn step(
    camera: &mut world_camera::Camera,
    rules: &world_camera::Rules,
    frame: &world_camera::Frame<'_>,
) {
    *camera = camera.step(rules, frame);
}

pub(super) fn follow(
    camera: &mut world_camera::Camera,
    previous: &samus_movement::Samus,
    movement: &Movement,
    input: &CameraInputs,
    rules: &world_camera::Rules,
) -> Result<(), Fault> {
    let (current, outputs) = movement.finished().ok_or(Fault::Movement)?;
    let Movement::Work(work) = movement else {
        return Err(Fault::Movement);
    };
    if camera.seen() != previous.position || &work.model.0.start != previous {
        return Err(Fault::Camera);
    }
    let [width, height] = input.size;
    let [normal, rising] = input.heights;
    let screens = input
        .screens
        .iter()
        .map(|screen| match screen {
            Screen::Open => types::ScreenKind::Open,
            Screen::Closed => types::ScreenKind::Closed,
            Screen::Short => types::ScreenKind::Short,
        })
        .collect();
    let room = world_camera::Room::new(
        screens,
        types::Pair {
            x: usize::from(width),
            y: usize::from(height),
        },
        world_camera::Heights {
            while_not_rising: Wrapping(normal),
            while_rising: Wrapping(rising),
        },
    )
    .map_err(|_| Fault::Camera)?;
    let frame = world_camera::Frame {
        samus: completed_samus(current, outputs),
        room: &room,
        lead: match input.lead {
            Lead::Ahead => world_camera::LeadSetting::Ahead,
            Lead::Left => world_camera::LeadSetting::KeepLeft,
            Lead::Right => world_camera::LeadSetting::KeepRight,
            Lead::Near => world_camera::LeadSetting::Near,
        },
        time_stopped: false,
    };
    step(camera, rules, &frame);
    Ok(())
}

pub(super) fn completed_samus(
    current: &samus_movement::Samus,
    outputs: &samus_movement::Outcome,
) -> world_camera::Samus {
    world_camera::Samus {
        position: current.position,
        placed: outputs
            .placed
            .map(|placed| types::PlacementY(placed.pixel().0)),
        facing: current.pose.facing,
        backwards: outputs.skidding,
        rising: outputs.rising,
    }
}
