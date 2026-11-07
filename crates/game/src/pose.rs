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
use data::palette::VisorColours;
use data::pose::animation::AnimationDelays;
use data::pose::spritemap::SamusSpritemaps;
use data::pose::tiles::{Graphics, SamusTiles};
use data::pose::transition::{Control, Controls, PoseTransitions};
use data::pose::{MovementType, PoseId, Poses};

use crate::animation;
use crate::model::num::Fixed;
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
            };
        }
    }
    // "Handle transition table lookup failure" (`$91:82D9`): running, a straight jump and the
    // ball's fall decelerate, keeping the pose while she has base speed; every other way of
    // moving asks for the pose's next.
    let decelerates = matches!(
        kind,
        MovementType::Running | MovementType::NormalJumping | MovementType::MorphBallFalling
    ) && samus.motion.base_speed != Fixed::default();
    let unaffected = if decelerates {
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

/// The pose change (`$91:EB88`): the super special prospective pose first, else the pose
/// asked for.
///
/// The pose takes the facing and movement type its definition gives when it differs from
/// the last; then the update records the history. The motion model has made the change's
/// writes to her speeds and placed her: pose fit and the shrink are its.
// open: the special prospective pose between them joins with the
// knockback and the grapple (C4); the super special pose with command 3 gives way to it
// when its command is 9.
pub fn change(samus: &mut Samus, asked: Option<PoseId>, content: &Content) {
    if let Some(pose) = samus.super_special.pose {
        let command = samus.super_special.command;
        if command != SuperSpecialCommand::KnockbackFinished {
            samus.pose = asked.unwrap_or(pose);
            refresh(samus, content);
            reset_animation(samus, content);
        }
        run_super_special(command, samus);
        record_history(samus);
        return;
    }
    let Some(pose) = asked else {
        return;
    };
    let old = samus.previous;
    samus.pose = pose;
    if pose != old.pose {
        refresh(samus, content);
        reset_animation(samus, content);
        samus.animation.new_pose_frame = NewPoseFrame::default();
    }
    record_history(samus);
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

#[cfg(test)]
mod tests;
