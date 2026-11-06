//! Samus's pose from input: the pose input handler picks a new pose from the pad through
//! the transition table, and the pose change applies it after movement.
//!
//! Bank $91 log: the normal input handler (`$91:8000`) skips the movement types that take
//! no input, then "determine new pose from input" (`$91:81A9`) walks the pose's transition
//! list (`$91:9EE2`) and, when no entry matches or nothing is held, asks for the pose the
//! pose definitions give "if not affected by buttons" (`$91:82EC`). Bank $91's pose
//! transition handler (`$91:EB88`) then applies the request after the movement handler and
//! records the pose history (<https://patrickjohnston.org/bank/91>).

use data::enemy::eye::EyeLists;
use data::enemy::program::EnemyPrograms;
use data::level::slope::Slopes;
use data::movement::{HorizontalSpeeds, VerticalSpeeds};
use data::pose::animation::AnimationDelays;
use data::pose::transition::{Control, Controls, PoseTransitions};
use data::pose::{MovementType, PoseId, Poses};

use crate::animation;
use crate::collision::{Contacts, Surroundings};
use crate::movement::{cancel_speed_boosting, make_jump};
use crate::state::samus::{
    Acceleration, AnimationTimer, Facing, Horizontal, Jump, Movement, NewPoseFrame, PoseRecord,
    Samus, Speed, SuperSpecialCommand, Turn, Vertical, YDirection,
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
    /// Samus's gravity and jump speeds.
    pub vertical: VerticalSpeeds,
    /// Samus's animation delay lists.
    pub delays: AnimationDelays,
    /// The slope shapes her block collision reads.
    pub slopes: Slopes,
    /// The loaded species' instruction lists.
    pub enemy_programs: EnemyPrograms,
    /// The Morph Ball Room eye's lists in them.
    pub eye_lists: EyeLists,
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
    /// The frame's solid vertical collision (`$91:EFC4`).
    SolidVertical(SolidVertical),
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
        | Movement::TurningAround { speed, .. }
        | Movement::Jumping { speed, .. }
        | Movement::Falling { speed, .. }
        | Movement::WallJumping { speed, .. } => speed.base.0 != 0,
        Movement::Crouching
        | Movement::SpringBall { .. }
        | Movement::KnockbackOrCrystalFlashEnding
        | Movement::Transition
        | Movement::Moonwalking
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

/// What the pose update reads of the frame's moves: what they met, and where a running
/// pose's check for a wall moves her.
#[derive(Clone, Copy, Debug, Default)]
pub struct Collisions<'a> {
    /// What the movement handler's moves met.
    pub contacts: Contacts,
    /// Where they happened.
    pub surroundings: Surroundings<'a>,
}

/// The pose change (`$91:EB88`): the super special prospective pose first, else the
/// requested pose, once the check for a wall has had its say.
///
/// The pose takes the facing and movement type its definition gives when it differs from
/// the last; then the update records the history.
// open: the special prospective pose between them joins with the
// knockback and the grapple (C4); the super special pose with command 3 gives way to it
// when its command is 9.
pub fn change(
    samus: &mut Samus,
    requests: PoseRequests,
    collisions: Collisions<'_>,
    content: &Content,
) {
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
    let requested = runs_into_wall(
        samus,
        requests.ordinary,
        collisions.contacts.wall,
        collisions.surroundings,
        content,
    );
    let Some(pose) = requested else {
        return;
    };
    samus.pose = pose;
    if pose != samus.previous.pose {
        // "Handle Samus pose change" (`$91:F404`): initialise the pose, then the jump.
        refresh(samus, content);
        jump_transition(samus, content);
        reset_animation(samus, content);
        samus.animation.new_pose_frame = NewPoseFrame::default();
    }
    match requests.command {
        Some(PoseCommand::SolidVertical(collision)) => solid_vertical(collision, samus),
        Some(command) => {
            if let Some(speed) = samus.movement.horizontal_mut() {
                apply(command, speed);
                if command != PoseCommand::Stop {
                    kill_y_speed_if_started_falling(samus);
                }
            }
        }
        None => {}
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
        let airborne = samus.movement.vertical();
        samus.movement = Movement::from(definition.movement);
        if let (Some(carried), Some(speed)) = (carried, samus.movement.horizontal_mut()) {
            *speed = carried;
        }
        if let (Some(airborne), Some(vertical)) = (airborne, samus.movement.vertical_mut()) {
            *vertical = airborne;
        }
    }
    initialise(samus.previous, samus.facing, &mut samus.movement);
}

/// The poses of the normal jump's take-off, the only normal jumping poses that start a jump
/// (`$91:FC66`) and that move her without her speeds (`$90:8FB3`).
pub(crate) const JUMP_TAKE_OFF: [PoseId; 8] = [
    PoseId(0x4B),
    PoseId(0x4C),
    PoseId(0x55),
    PoseId(0x56),
    PoseId(0x57),
    PoseId(0x58),
    PoseId(0x59),
    PoseId(0x5A),
];

/// The crouching poses a jump from which starts higher (`$91:FC7D`).
const CROUCHING: [PoseId; 2] = [PoseId(0x27), PoseId(0x28)];

/// The crouch jump's boost in pixels (`$91:FC8A`).
const CROUCH_JUMP_BOOST: u16 = 10;

/// "Handle jump transition" (`$91:FBBB`): a new jumping pose starts the jump.
// open: the spring ball's jump (`$91:FC18`, poses $7F/$80 from the ground) joins with the
// spring ball's vertical motion; type $D's (`$91:FC42`) has only unused poses.
fn jump_transition(samus: &mut Samus, content: &Content) {
    let old = samus.previous;
    let takeoff = match samus.movement {
        Movement::Jumping {
            jump: Jump::Normal, ..
        } => {
            let take_off = JUMP_TAKE_OFF.contains(&samus.pose);
            if take_off && CROUCHING.contains(&old.pose) {
                let y = &mut samus.position.y.pixels;
                *y = y.wrapping_sub(CROUCH_JUMP_BOOST);
            }
            take_off.then_some(Takeoff::Jump)
        }
        Movement::Jumping {
            jump: Jump::Spin, ..
        } => (!matches!(
            old.movement,
            MovementType::SpinJumping | MovementType::WallJumping
        ))
        .then_some(Takeoff::Jump),
        Movement::WallJumping { .. } => {
            (old.movement != MovementType::WallJumping).then_some(Takeoff::WallJump)
        }
        Movement::Standing { .. }
        | Movement::Running { .. }
        | Movement::Crouching
        | Movement::Falling { .. }
        | Movement::MorphBall { .. }
        | Movement::SpringBall { .. }
        | Movement::KnockbackOrCrystalFlashEnding
        | Movement::TurningAround { .. }
        | Movement::Transition
        | Movement::Moonwalking
        | Movement::RanIntoWall
        | Movement::Grappling
        | Movement::DamageBoost
        | Movement::GrabbedByDraygon
        | Movement::Shinespark
        | Movement::Unused(_) => None,
    };
    if let Some(takeoff) = takeoff {
        make_jump(samus, takeoff, &content.vertical);
    }
}

/// Which initial speed a jump takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Takeoff {
    /// From the ground ("Make Samus jump", bank $90 log `$90:98BC`).
    Jump,
    /// From a wall ("Make Samus wall-jump", `$90:9949`).
    WallJump,
}

/// "Kill Y speed if started falling" (`$91:FB8E`): a command that leaves her falling from
/// another movement stops her vertical speed. The bank log notes no command reaches it.
const fn kill_y_speed_if_started_falling(samus: &mut Samus) {
    if matches!(samus.previous.movement, MovementType::Falling) {
        return;
    }
    if let Movement::Falling { vertical, .. } = &mut samus.movement {
        *vertical = Vertical {
            speed: Speed::ZERO,
            direction: YDirection::Down,
        };
    }
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

/// "Initialise Samus pose" (`$91:F433`) for a new pose, the part that sets her speeds.
///
/// Turning on the ground starts the turn's momentum, her run speed joining her base speed
/// (`$91:F8D3`); a normal jump or a fall decelerates while she keeps extra run speed
/// (`$91:F59A`, `$91:F60D`); a spin jump from a spin or wall jump the other way turns her
/// momentum around (`$91:F624`).
// open: the shinespark windup from a normal jump (`$91:F543`, the special palette timer),
// the animation skips (`$91:F5AF`, `$91:F634`), the spin jump's sounds, space jump and screw
// attack poses (`$91:F67F`), the morph ball and the other types' initialisations join with
// the speed booster, C6 and the items; the turning poses chosen from the old pose's shot
// direction (`$91:F9C2`) match the transition table's in slice 0.
const fn initialise(old: PoseRecord, facing: Facing, movement: &mut Movement) {
    match movement {
        Movement::TurningAround {
            place: Turn::OnGround,
            speed,
        } => {
            speed.base = Speed(speed.base.0.wrapping_add(speed.extra_run.0));
            speed.extra_run = Speed::ZERO;
            speed.acceleration = Acceleration::TurningAround;
        }
        Movement::Jumping {
            jump: Jump::Normal,
            speed,
            ..
        }
        | Movement::Falling { speed, .. } => {
            speed.acceleration = if speed.extra_run.0 == 0 {
                Acceleration::Accelerating
            } else {
                Acceleration::Decelerating
            };
        }
        Movement::Jumping {
            jump: Jump::Spin,
            speed,
            ..
        } => {
            let from_spin = matches!(
                old.movement,
                MovementType::SpinJumping | MovementType::WallJumping
            );
            let turned = matches!(
                (old.facing, facing),
                (Facing::Left, Facing::Right) | (Facing::Right, Facing::Left)
            );
            if from_spin && turned {
                speed.base = Speed(speed.base.0.wrapping_add(speed.extra_run.0));
                cancel_speed_boosting(speed);
                speed.extra_run = Speed::ZERO;
                speed.acceleration = Acceleration::TurningAround;
            }
        }
        Movement::Standing { .. }
        | Movement::Running { .. }
        | Movement::Crouching
        | Movement::MorphBall { .. }
        | Movement::SpringBall { .. }
        | Movement::KnockbackOrCrystalFlashEnding
        | Movement::TurningAround { .. }
        | Movement::Transition
        | Movement::Moonwalking
        | Movement::WallJumping { .. }
        | Movement::RanIntoWall
        | Movement::Grappling
        | Movement::DamageBoost
        | Movement::GrabbedByDraygon
        | Movement::Shinespark
        | Movement::Unused(_) => {}
    }
}

/// A prospective pose change command's effect on her speeds (`$91:EC16`'s table).
// open: the movements without `Horizontal` yet (the morph ball, the spring ball, grappling)
// ignore the commands.
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
        // It reads more than her horizontal speeds, so `change` runs it on its own.
        PoseCommand::SolidVertical(_) => {}
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

mod contact;

pub use contact::{Fall, Grounding, SolidVertical, request as collision_request};
use contact::{runs_into_wall, solid_vertical};

#[cfg(test)]
mod tests;
