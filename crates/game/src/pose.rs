//! Samus's pose from input: the pose input handler picks a new pose from the pad through
//! the transition table, and the pose change applies it after movement.
//!
//! Bank $91 log: the normal input handler (`$91:8000`) skips the movement types that take
//! no input, then "determine new pose from input" (`$91:81A9`) walks the pose's transition
//! list (`$91:9EE2`) and, when no entry matches or nothing is held, asks for the pose the
//! pose definitions give "if not affected by buttons" (`$91:82EC`). Bank $91's pose
//! transition handler (`$91:EB88`) then applies the request after the movement handler and
//! records the pose history (<https://patrickjohnston.org/bank/91>).

use data::pose::transition::{Control, Controls, PoseTransitions};
use data::pose::{MovementType, PoseId, Poses};

use crate::state::samus::{Movement, PoseRecord, Samus};
use crate::state::{Button, Buttons, Controller};

/// The content the Session reads: the decoded tables its rules look up.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Content {
    /// Samus's pose definitions.
    pub poses: Poses,
    /// Samus's pose transitions from input.
    pub transitions: PoseTransitions,
}

/// The pose a frame asks for, by the channel that asked.
///
/// The original keeps the three requests in work RAM between its input handler and its
/// pose transition handler, which consumes them in the same frame, so a Tick holds them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoseRequests {
    /// From input or the pose's own next pose.
    pub ordinary: Option<PoseId>,
}

/// The pad buttons each control is on: the controller settings' defaults.
// open: the controller settings are the player's options, kept in work RAM and saved; they
// join the State with the options menu. Until then every Session uses the defaults.
const DEFAULT_BINDINGS: [(Button, Control); 10] = [
    (Button::Up, Control::Up),
    (Button::Down, Control::Down),
    (Button::Left, Control::Left),
    (Button::Right, Control::Right),
    (Button::R, Control::AimUp),
    (Button::L, Control::AimDown),
    (Button::X, Control::Shoot),
    (Button::A, Control::Jump),
    (Button::B, Control::Run),
    (Button::Y, Control::ItemCancel),
];

/// Whether `buttons` holds `button`.
const fn holds(buttons: Buttons, button: Button) -> bool {
    (buttons.bits & button.bit()) != 0
}

/// The controls `buttons` work under the bindings.
fn controls(buttons: Buttons) -> Controls {
    let held: Vec<Control> = DEFAULT_BINDINGS
        .iter()
        .filter(|&&(button, _)| holds(buttons, button))
        .map(|&(_, control)| control)
        .collect();
    Controls::of(&held)
}

/// Whether a movement type ignores input: the unused types 7, 9 and $B, and the crouching
/// transitions, whose poses run on by themselves (`$91:8000`).
// open: standing in pose 0 or $9B on a moving elevator ignores input too; the elevator's
// state joins with its ride (C7).
const fn ignores_input(movement: MovementType) -> bool {
    matches!(
        movement,
        MovementType::Unused07
            | MovementType::Unused09
            | MovementType::Unused0B
            | MovementType::CrouchingTransition
    )
}

/// Whether a movement type keeps its pose while Samus still has horizontal speed when no
/// entry matches: running, normal jumping and the falling morph ball (`$91:82EC`'s action
/// table, the momentum action 1).
const fn keeps_pose_while_moving(movement: MovementType) -> bool {
    matches!(
        movement,
        MovementType::Running | MovementType::NormalJumping | MovementType::MorphBallFalling
    )
}

/// Whether Samus has horizontal speed.
const fn moving(movement: Movement) -> bool {
    match movement {
        Movement::Running { speed } | Movement::MorphBall { speed, .. } => {
            speed.pixels != 0 || speed.subpixels != 0
        }
        // open: the jumps' and falls' horizontal speed joins with C3b.
        Movement::Standing
        | Movement::Crouching
        | Movement::Jumping { .. }
        | Movement::Falling { .. }
        | Movement::SpringBall { .. }
        | Movement::KnockbackOrCrystalFlashEnding
        | Movement::TurningAround { .. }
        | Movement::Transition
        | Movement::Moonwalking
        | Movement::WallJumping
        | Movement::RanIntoWall
        | Movement::Grappling
        | Movement::DamageBoost
        | Movement::GrabbedByDraygon
        | Movement::Shinespark
        | Movement::Unused(_) => false,
    }
}

/// The pose input handler: the request this frame's controls make (`$91:8000`).
#[must_use]
pub fn input(samus: &Samus, controller: &Controller, content: &Content) -> PoseRequests {
    let kind = samus.movement.kind();
    if ignores_input(kind) {
        return PoseRequests::default();
    }
    if !controller.held.is_empty() {
        let list = content.transitions.of(samus.pose);
        // A pose with no entries leaves the request alone.
        if list.is_empty() {
            return PoseRequests::default();
        }
        let pressed = controls(controller.pressed);
        let held = controls(controller.held);
        if let Some(transition) = list.iter().find(|entry| entry.matches(pressed, held)) {
            return PoseRequests {
                ordinary: (transition.pose != samus.pose).then_some(transition.pose),
            };
        }
    }
    let unaffected = if keeps_pose_while_moving(kind) && moving(samus.movement) {
        samus.pose
    } else {
        content
            .poses
            .get(samus.pose)
            .and_then(|definition| definition.next)
            .unwrap_or(samus.pose)
    };
    PoseRequests {
        ordinary: Some(unaffected),
    }
}

/// The pose change: take the requested pose, with the facing and movement type its
/// definition gives when it differs from the last, then record the history (`$91:EB88`).
pub fn change(samus: &mut Samus, requests: PoseRequests, content: &Content) {
    let Some(pose) = requests.ordinary else {
        return;
    };
    samus.pose = pose;
    if pose != samus.previous.pose
        && let Some(definition) = content.poses.get(pose)
    {
        samus.facing = definition.facing;
        if samus.movement.kind() != definition.movement {
            samus.movement = Movement::from(definition.movement);
        }
    }
    samus.last_different = samus.previous;
    samus.previous = PoseRecord {
        pose,
        facing: samus.facing,
        movement: samus.movement.kind(),
    };
}

#[cfg(test)]
mod tests;
