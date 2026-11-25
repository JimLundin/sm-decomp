//! Following from actual completed movement and the retained prior centre.

use super::connected::{
    Fault,
    inputs::{CameraInputs, Lead, Screen},
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
    current: &samus_movement::Samus,
    outputs: &samus_movement::Outcome,
    input: &CameraInputs,
    rules: &world_camera::Rules,
) -> Result<(), Fault> {
    if camera.seen() != previous.position {
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
        backwards: backwards(current, outputs.skidding),
        rising: outputs.rising,
    }
}

/// World-camera §1.3's going backwards: she skids or moonwalks, or is in exactly one of a hit's
/// push and the backward leap. The leap keeps her facing as she travels against it, and while the
/// push still moves her the two turn the lead round twice.
fn backwards(current: &samus_movement::Samus, skidding: bool) -> bool {
    skidding
        || current.pose.action == types::Action::Moonwalking
        || (current.knockback.push.is_some() != (current.pose.action == types::Action::Leaping))
}

#[cfg(test)]
mod tests {
    use super::backwards;
    use crate::production::tests::fixtures;
    use samus_movement::Push;
    use types::Action;

    /// Her, not skidding, in `action` with `push` under way.
    fn goes_backwards(action: Action, push: Option<Push>) -> bool {
        let mut samus = fixtures::samus();
        samus.pose.action = action;
        samus.knockback.push = push;
        backwards(&samus, false)
    }

    /// The push and the leap each turn the lead round; together they cancel, so the camera reads
    /// exactly one of them as going backwards (world-camera §1.3, `LeadingHer`).
    #[test]
    fn exactly_one_of_the_push_and_the_leap_goes_backwards() {
        assert!(!goes_backwards(Action::Leaping, Some(Push::Up)));
        assert!(goes_backwards(Action::Leaping, None));
        assert!(goes_backwards(Action::Recoiling, Some(Push::Up)));
        assert!(!goes_backwards(Action::Falling, None));
    }
}
