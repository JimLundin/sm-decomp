//! Samus's pose from input: the pose input handler picks a new pose from the pad through
//! the transition table, and the pose change applies it after movement.
//!
//! Bank $91 log: the normal input handler (`$91:8000`) skips the movement types that take
//! no input, then "determine new pose from input" (`$91:81A9`) walks the pose's transition
//! list (`$91:9EE2`) and, when no entry matches or nothing is held, asks for the pose the
//! pose definitions give "if not affected by buttons" (`$91:82EC`). Bank $91's pose
//! transition handler (`$91:EB88`) then applies the request after the movement handler and
//! records the pose history (<https://patrickjohnston.org/bank/91>).

use data::movement::HorizontalSpeeds;
use data::pose::animation::AnimationDelays;
use data::pose::transition::{Control, Controls, PoseTransitions};
use data::pose::{MovementType, PoseId, Poses};

use crate::animation;
use crate::movement::cancel_speed_boosting;
use crate::state::samus::{
    Acceleration, AnimationTimer, Horizontal, Movement, NewPoseFrame, PoseRecord, Samus, Speed,
    SuperSpecialCommand, Turn,
};
use crate::state::{Button, Buttons, Controller};

/// The content the Session reads: the decoded tables its rules look up.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Content {
    /// Samus's pose definitions.
    pub poses: Poses,
    /// Samus's pose transitions from input.
    pub transitions: PoseTransitions,
    /// Samus's horizontal speed tables.
    pub speeds: HorizontalSpeeds,
    /// Samus's animation delay lists.
    pub delays: AnimationDelays,
}

/// The pose a frame asks for, by the channel that asked.
///
/// The original keeps the three requests in work RAM between its input handler and its
/// pose transition handler, which consumes them in the same frame, so a Tick holds them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoseRequests {
    /// From input or the pose's own next pose.
    pub ordinary: Option<PoseId>,
    /// What the update does to her speeds after it (`SAMUS_MOMENTUM_ROUTINE_INDEX`, reset at
    /// the start of every frame by `$90:E695`).
    pub command: Option<PoseCommand>,
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
pub(crate) fn controls(buttons: Buttons) -> Controls {
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

/// What the pose update does to Samus's speeds when no transition matched her controls.
///
/// Bank $91 log: `$91:8304` "Set prospective pose change command" and its table at
/// `$91:8332`, one command a movement type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoseCommand {
    /// Slow down and keep the pose while she still has base speed (`$91:EC50`).
    Decelerate,
    /// Stop accelerating (`$91:ECD0`).
    Stop,
    /// Lose the base speed, then the run speed (`$91:EC85`, which runs on into `$91:EC8E`).
    KillXSpeed,
    /// Lose the extra run speed (`$91:EC8E`).
    KillRunSpeed,
}

/// The command a movement type's lookup failure gives (`$91:8332`); spin jumping has none.
const fn failed_lookup_command(movement: MovementType) -> Option<PoseCommand> {
    match movement {
        MovementType::SpinJumping => None,
        MovementType::Running | MovementType::NormalJumping | MovementType::MorphBallFalling => {
            Some(PoseCommand::Decelerate)
        }
        MovementType::MorphBallOnGround
        | MovementType::Unused09
        | MovementType::Unused0D
        | MovementType::SpringBallOnGround
        | MovementType::SpringBallInAir
        | MovementType::SpringBallFalling
        | MovementType::WallJumping
        | MovementType::Grappling => Some(PoseCommand::KillXSpeed),
        MovementType::Falling => Some(PoseCommand::KillRunSpeed),
        MovementType::Standing
        | MovementType::Crouching
        | MovementType::Unused07
        | MovementType::KnockbackOrCrystalFlashEnding
        | MovementType::Unused0B
        | MovementType::Unused0C
        | MovementType::TurningAroundOnGround
        | MovementType::CrouchingTransition
        | MovementType::Moonwalking
        | MovementType::RanIntoWall
        | MovementType::TurningAroundJumping
        | MovementType::TurningAroundFalling
        | MovementType::DamageBoost
        | MovementType::GrabbedByDraygon
        | MovementType::Shinespark => Some(PoseCommand::Stop),
    }
}

/// Whether Samus has base speed, which decides whether decelerating keeps her pose.
const fn has_base_speed(movement: Movement) -> bool {
    match movement {
        Movement::MorphBall { speed, .. } => speed.pixels != 0 || speed.subpixels != 0,
        Movement::Standing { speed }
        | Movement::Running { speed }
        | Movement::TurningAround { speed, .. } => speed.base.0 != 0,
        // open: the jumps' and falls' horizontal speed joins with C3b.
        Movement::Crouching
        | Movement::Jumping { .. }
        | Movement::Falling { .. }
        | Movement::SpringBall { .. }
        | Movement::KnockbackOrCrystalFlashEnding
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
                command: None,
            };
        }
    }
    // "Handle transition table lookup failure" (`$91:82D9`): decelerating keeps the pose,
    // and with no base speed left it is a stop.
    let command = match failed_lookup_command(kind) {
        Some(PoseCommand::Decelerate) if !has_base_speed(samus.movement) => Some(PoseCommand::Stop),
        other => other,
    };
    let unaffected = if command == Some(PoseCommand::Decelerate) {
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
        command,
    }
}

/// The pose change (`$91:EB88`): the super special prospective pose first, else the
/// requested pose.
///
/// The pose takes the facing and movement type its definition gives when it differs from
/// the last; then the update records the history.
// open: the special prospective pose between them joins with the
// knockback and the grapple (C4); the super special pose with command 3 gives way to it
// when its command is 9.
pub fn change(samus: &mut Samus, requests: PoseRequests, content: &Content) {
    if let Some(pose) = samus.super_special.pose {
        let command = samus.super_special.command;
        if command != SuperSpecialCommand::KnockbackFinished {
            samus.pose = pose;
            refresh(samus, content);
            reset_animation(samus, content);
        }
        run_super_special(command, samus);
        record_history(samus);
        return;
    }
    let Some(pose) = requests.ordinary else {
        return;
    };
    samus.pose = pose;
    if pose != samus.previous.pose {
        refresh(samus, content);
        reset_animation(samus, content);
        samus.animation.new_pose_frame = NewPoseFrame::default();
    }
    if let (Some(command), Some(speed)) = (requests.command, samus.movement.horizontal_mut()) {
        apply(command, speed);
    }
    record_history(samus);
}

/// Copy the pose's facing and movement type from its definition and initialise the pose
/// (`$91:F433`).
fn refresh(samus: &mut Samus, content: &Content) {
    let Some(definition) = content.poses.get(samus.pose) else {
        return;
    };
    samus.facing = definition.facing;
    if animation::preserves(samus.previous.movement, definition.movement) {
        samus.animation.new_pose_frame = NewPoseFrame::Preserve;
    }
    if samus.movement.kind() != definition.movement {
        // The original keeps her speeds in work RAM whatever the movement: a movement that
        // keeps them takes them over.
        let carried = samus.movement.horizontal();
        samus.movement = Movement::from(definition.movement);
        if let (Some(carried), Some(speed)) = (carried, samus.movement.horizontal_mut()) {
            *speed = carried;
        }
    }
    initialise(&mut samus.movement);
}

/// Start the new pose's animation on the new pose frame (`$91:FB08`), unless the pose is
/// unchanged or the frame is preserved.
// open: with the gravity suit off, water gives a delay of 3 and lava or acid 2 in place of
// the X speed divisor; they join with the liquids (C10).
fn reset_animation(samus: &mut Samus, content: &Content) {
    let NewPoseFrame::From(frame) = samus.animation.new_pose_frame else {
        return;
    };
    if samus.pose == samus.previous.pose {
        return;
    }
    samus.animation.frame = frame;
    if let Some(byte) = content
        .delays
        .of(samus.pose)
        .and_then(|list| list.byte(frame))
    {
        let delay = animation::frame_buffer(samus);
        samus.animation.timer = AnimationTimer(delay.wrapping_add(u16::from(byte)));
    }
}

/// The super special prospective pose change command, after the pose (bank $91 log, its
/// table at `$91:EBEB`).
// open: the knockback's, the shinespark's, the grapple's and the screen-centring commands
// join with C4 and C7; "transition animation finished" also clears the landing bounce,
// which joins with C3b.
const fn run_super_special(command: SuperSpecialCommand, samus: &mut Samus) {
    match command {
        SuperSpecialCommand::TransitionAnimationFinished => {
            let buffer = animation::frame_buffer(samus);
            let timer = &mut samus.animation.timer;
            *timer = AnimationTimer(timer.0.wrapping_add(buffer));
        }
        SuperSpecialCommand::None
        | SuperSpecialCommand::KnockbackFinished
        | SuperSpecialCommand::ShinesparkFinished
        | SuperSpecialCommand::CentreOnScreen
        | SuperSpecialCommand::MoveDown
        | SuperSpecialCommand::StartGrappleWallJump
        | SuperSpecialCommand::StartReleaseFromGrappleSwing
        | SuperSpecialCommand::KnockbackAndTransitionAnimationFinished => {}
    }
}

/// Record the pose history after a change.
const fn record_history(samus: &mut Samus) {
    samus.last_different = samus.previous;
    samus.previous = PoseRecord {
        pose: samus.pose,
        facing: samus.facing,
        movement: samus.movement.kind(),
    };
}

/// "Initialise Samus pose" (`$91:F433`) for a new pose, the part that sets her speeds:
/// turning on the ground starts the turn's momentum, her run speed joining her base speed
/// (`$91:F8D3`).
// open: the other movement types' initialisations (jumps, the morph ball, spin jumps'
// momentum, the wall jump) join with C3b; the turning poses chosen from the old pose's shot
// direction (`$91:F9C2`) match the transition table's in slice 0.
const fn initialise(movement: &mut Movement) {
    if let Movement::TurningAround {
        place: Turn::OnGround,
        speed,
    } = movement
    {
        speed.base = Speed(speed.base.0.wrapping_add(speed.extra_run.0));
        speed.extra_run = Speed::ZERO;
        speed.acceleration = Acceleration::TurningAround;
    }
}

/// A prospective pose change command's effect on her speeds (`$91:EC16`'s table).
// open: "Kill Y speed if started falling" (`$91:FB8E`) after decelerating or losing the run
// speed joins with C3b; the movements without `Horizontal` yet ignore the commands.
const fn apply(command: PoseCommand, speed: &mut Horizontal) {
    match command {
        PoseCommand::Decelerate => {
            if speed.base.0 == 0 {
                stop(speed);
            } else {
                speed.base = Speed(speed.base.0.wrapping_add(speed.extra_run.0));
                speed.acceleration = Acceleration::Decelerating;
                cancel_speed_boosting(speed);
                speed.extra_run = Speed::ZERO;
            }
        }
        PoseCommand::Stop => stop(speed),
        PoseCommand::KillXSpeed => {
            speed.acceleration = Acceleration::Accelerating;
            speed.base = Speed::ZERO;
            kill_run_speed(speed);
        }
        PoseCommand::KillRunSpeed => kill_run_speed(speed),
    }
}

/// "Prospective pose change command 2 - stop" (`$91:ECD0`).
const fn stop(speed: &mut Horizontal) {
    speed.acceleration = Acceleration::Accelerating;
    cancel_speed_boosting(speed);
}

/// "Prospective pose change command 8 - kill run speed" (`$91:EC8E`).
const fn kill_run_speed(speed: &mut Horizontal) {
    cancel_speed_boosting(speed);
    speed.extra_run = Speed::ZERO;
}

#[cfg(test)]
mod tests;
