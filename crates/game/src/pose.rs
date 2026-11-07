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
use data::enemy::face_block::{FaceBlockLists, GlowColours};
use data::enemy::program::EnemyPrograms;
use data::level::slope::Slopes;
use data::movement::{HorizontalSpeeds, VerticalSpeeds};
use data::palette::VisorColours;
use data::pose::animation::AnimationDelays;
use data::pose::spritemap::SamusSpritemaps;
use data::pose::tiles::{Graphics, SamusTiles};
use data::pose::transition::{Control, Controls, PoseTransitions};
use data::pose::{MovementType, PoseId, Poses};

use crate::animation;
use crate::collision::{Surroundings, Touches};
use crate::model::num::Fixed;
use crate::model::samus::motion::{self, PoseChange, PoseStart, VerticalResult};
use crate::motion::Motion;
use crate::plm::RoomPlms;
use crate::state::room::{ElevatorRide, ElevatorStatus};
use crate::state::samus::{AnimationTimer, NewPoseFrame, PoseRecord, Samus, SuperSpecialCommand};
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
    /// The Blue Brinstar face block's lists in them.
    pub face_block_lists: FaceBlockLists,
    /// Samus's spritemaps by pose and frame, which the importer reads.
    pub samus_spritemaps: SamusSpritemaps,
    /// Samus's sprite sheet and animation clips, which she is drawn from.
    pub samus_assets: data::assets::samus::SamusAssets,
    /// Samus's tile uploads by pose and frame.
    pub samus_tiles: SamusTiles,
    /// The ROM graphics uploads read.
    pub graphics: Graphics,
    /// Her visor's colour cycle.
    pub visor_colours: VisorColours,
    /// The face block's glow colour sets.
    pub face_block_glow: GlowColours,
    /// The rooms' PLM populations, with their scroll PLMs' change lists.
    pub room_plms: RoomPlms,
    /// The area maps the mini-map draws.
    pub area_maps: data::map::AreaMaps,
    /// The tilesets' tile tables, which draw the level's blocks.
    pub tile_tables: data::tileset::TileTables,
    /// The message boxes' borders, letters and messages.
    pub message_boxes: data::message::MessageBoxes,
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
    /// The frame's vertical result, passed back (`$91:EFC4`).
    Vertical(VerticalResult),
    /// The start of a crouching or morphing transition animation (`$91:ECDA`), which the pose
    /// sets itself when it is initialised.
    StartTransitionAnimation,
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

/// The pose input handler: the request this frame's controls make (`$91:8000`).
#[must_use]
pub fn input(samus: &Samus, controller: &Controller, content: &Content) -> PoseRequests {
    let kind = samus.movement;
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
        Some(PoseCommand::Decelerate) if samus.motion.base_speed == Fixed::default() => {
            Some(PoseCommand::Stop)
        }
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

/// What the pose update reads of the frame: motion's step, and where the arm-pump step and
/// pose fit move her.
#[derive(Clone, Copy, Debug)]
pub struct Collisions<'a> {
    /// The frame's motion.
    pub motion: Motion<'a>,
    /// Where the moves happen.
    pub surroundings: Surroundings<'a>,
}

/// The poses that face the screen: standing in them is the elevator's, which takes no
/// horizontal motion (`$90:A383`), and a change to them checks nothing (`$91:FDAE`).
pub(crate) const FACING_FORWARD: [PoseId; 2] = [PoseId(0), PoseId(0x9B)];

/// "Set Samus radius" (`$90:EC22`), which the normal current state handler runs first: her
/// Y radius is her pose's (`$91:B62F`).
pub fn set_radius(samus: &mut Samus, content: &Content) {
    samus.y_radius = content
        .poses
        .get(samus.pose)
        .map_or(0, |pose| u16::from(pose.y_radius));
}

/// Whether the elevator carries her: standing facing the screen while a ride is on. The pose
/// input handler takes no input then (`$91:805C`).
#[must_use]
pub fn carried(samus: &Samus, ride: ElevatorRide) -> bool {
    ride.status != ElevatorStatus::Inactive
        && FACING_FORWARD.contains(&samus.pose)
        && samus.movement == MovementType::Standing
}

/// The pose change (`$91:EB88`): the super special prospective pose first, else the
/// requested pose, once the check for a wall has had its say.
///
/// The pose takes the facing and movement type its definition gives when it differs from
/// the last; motion takes the new pose's start and the command; then the update records the
/// history. The scroll triggers the wall check's move reaches join `touches`.
// open: the special prospective pose between them joins with the
// knockback and the grapple (C4); the super special pose with command 3 gives way to it
// when its command is 9.
pub fn change(
    samus: &mut Samus,
    requests: PoseRequests,
    collisions: Collisions<'_>,
    touches: &mut Touches,
    content: &Content,
) {
    if let Some(pose) = samus.super_special.pose {
        let command = samus.super_special.command;
        if command == SuperSpecialCommand::TransitionAnimationFinished {
            contact::forced_arm_pump(samus, collisions, touches);
        }
        if command != SuperSpecialCommand::KnockbackFinished {
            let old = samus.previous;
            samus.pose = pose;
            refresh(samus, content);
            let start = if samus.pose == old.pose {
                None
            } else {
                pose_start(old, samus)
            };
            reset_animation(samus, content);
            if command == SuperSpecialCommand::TransitionAnimationFinished {
                let command = motion::PoseCommand::AnimationEnd;
                pose_writes(samus, old, pose, start, command, collisions.motion);
            }
        }
        run_super_special(command, samus);
        record_history(samus);
        return;
    }
    let requested = runs_into_wall(samus, requests.ordinary, collisions, touches, content);
    let Some(pose) = requested else {
        return;
    };
    let old = samus.previous;
    samus.pose = pose;
    let mut command = requests.command;
    let mut start = None;
    if pose != old.pose {
        // "Handle Samus pose change" (`$91:F404`): fit the new pose's height, then initialise
        // the pose, whichever it is now.
        fit::fit(samus, collisions.surroundings, touches, content);
        refresh(samus, content);
        if samus.pose != old.pose {
            start = pose_start(old, samus);
        }
        reset_animation(samus, content);
        samus.animation.new_pose_frame = NewPoseFrame::default();
        command = start_transition_command(samus).or(command);
    }
    // The commands run only while the pose stands as it was asked for: a pose the handler
    // changed, by crouching her or refusing the change, skips them (`$91:EBEE`).
    let motion_command = match command {
        Some(PoseCommand::Decelerate) => motion::PoseCommand::Decelerate,
        Some(PoseCommand::Stop) => motion::PoseCommand::Stop,
        Some(PoseCommand::KillXSpeed) => motion::PoseCommand::KillXSpeed,
        Some(PoseCommand::KillRunSpeed) => motion::PoseCommand::KillRunSpeed,
        Some(PoseCommand::Vertical(result)) => motion::PoseCommand::Vertical(result),
        Some(PoseCommand::StartTransitionAnimation) | None => motion::PoseCommand::None,
    };
    pose_writes(samus, old, pose, start, motion_command, collisions.motion);
    if samus.pose == pose && command == Some(PoseCommand::StartTransitionAnimation) {
        fit::start_transition(samus, collisions.surroundings, touches, content);
    }
    record_history(samus);
}

/// Motion's writes as the pose asked for starts after `old`: `start`, then `command`
/// (spec §7). The pose was replaced when the change left her in another one.
fn pose_writes(
    samus: &mut Samus,
    old: PoseRecord,
    asked: PoseId,
    start: Option<PoseStart>,
    command: motion::PoseCommand,
    frame: Motion<'_>,
) {
    let Some(inputs) = frame.inputs else {
        return;
    };
    let change = PoseChange {
        start,
        replaced: samus.pose != asked,
        command,
        started_falling: samus.movement == MovementType::Falling
            && old.movement != MovementType::Falling,
    };
    let jump_held = !matches!(inputs.pad.jump, motion::JumpButton::Released);
    samus
        .motion
        .pose_writes(change, jump_held, inputs.equipment, &frame.rules);
}

/// What the new pose's start writes to motion, after `old` (spec §7's `PoseStart`).
fn pose_start(old: PoseRecord, samus: &Samus) -> Option<PoseStart> {
    use MovementType as M;
    let turn = |movement| {
        matches!(
            movement,
            M::TurningAroundOnGround | M::TurningAroundJumping | M::TurningAroundFalling
        )
    };
    let ball = |movement| {
        matches!(
            movement,
            M::MorphBallOnGround
                | M::MorphBallFalling
                | M::SpringBallOnGround
                | M::SpringBallInAir
                | M::SpringBallFalling
        )
    };
    let spun = matches!(old.movement, M::SpinJumping | M::WallJumping);
    let reversed = samus.facing != old.facing;
    let take_off = |pose| JUMP_TAKE_OFF.contains(&pose);
    match samus.movement {
        new if turn(new) => (!turn(old.movement)).then_some(PoseStart::Turn),
        new if ball(new) && ball(old.movement) && reversed => Some(PoseStart::BallReverse),
        M::SpinJumping if spun => reversed.then_some(PoseStart::SpinReverse),
        M::SpinJumping => Some(PoseStart::SpinStart),
        M::NormalJumping if take_off(samus.pose) && !take_off(old.pose) => {
            Some(PoseStart::TakeOff {
                from_plain_crouch: CROUCHING.contains(&old.pose),
            })
        }
        M::NormalJumping | M::Falling => Some(PoseStart::JumpOrFall),
        M::SpringBallInAir => {
            (old.movement == M::SpringBallOnGround).then_some(PoseStart::SpringLaunch)
        }
        M::WallJumping => (old.movement != M::WallJumping).then_some(PoseStart::WallJumpStart),
        M::Standing
        | M::Running
        | M::MorphBallOnGround
        | M::Crouching
        | M::Unused07
        | M::MorphBallFalling
        | M::Unused09
        | M::KnockbackOrCrystalFlashEnding
        | M::Unused0B
        | M::Unused0C
        | M::Unused0D
        | M::TurningAroundOnGround
        | M::CrouchingTransition
        | M::Moonwalking
        | M::SpringBallOnGround
        | M::SpringBallFalling
        | M::RanIntoWall
        | M::Grappling
        | M::TurningAroundJumping
        | M::TurningAroundFalling
        | M::DamageBoost
        | M::GrabbedByDraygon
        | M::Shinespark => None,
    }
}

/// The unused poses $DB to $F0, whose initialisation sets no command (`$91:F775`).
pub(crate) const UNUSED_TRANSITIONS: std::ops::Range<u16> = 0xDB..0xF1;

/// The command the transition poses' initialisation sets (`$91:F758`): the start of the
/// animation, for every transition pose but the unused `$DB` to `$F0`.
fn start_transition_command(samus: &Samus) -> Option<PoseCommand> {
    let unused = UNUSED_TRANSITIONS.contains(&samus.pose.0);
    (samus.movement == MovementType::CrouchingTransition && !unused)
        .then_some(PoseCommand::StartTransitionAnimation)
}

/// Copy the pose's facing and movement type from its definition (`$91:F433`).
fn refresh(samus: &mut Samus, content: &Content) {
    let Some(definition) = content.poses.get(samus.pose) else {
        return;
    };
    samus.facing = definition.facing;
    if let Some(frame) = animation::new_pose_frame(samus.previous.movement, definition.movement) {
        samus.animation.new_pose_frame = frame;
    }
    samus.movement = definition.movement;
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
const CROUCHING: [PoseId; 2] = [fit::CROUCH_RIGHT, fit::CROUCH_LEFT];

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
        movement: samus.movement,
    };
}

mod contact;
mod fit;

pub use contact::request as collision_request;
use contact::runs_into_wall;

#[cfg(test)]
mod tests;
