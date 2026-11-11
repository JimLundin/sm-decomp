//! Samus's animation: each frame's delay counts down, and the delay list's next item shows
//! the next frame or runs an instruction that loops, goes back or ends the animation in a
//! new pose.
//!
//! Bank $90 log (<https://patrickjohnston.org/bank/90>): "Animate Samus" (`$90:8000`) runs
//! between the movement handler and the pose update (`$90:E725`). It counts the timer down
//! and, when the frame is done, advances it and runs "Handle Samus animation delay"
//! (`$90:82DC`): a delay byte sets the timer (`$90:84E3`), an instruction byte first passes
//! the running momentum check (`$90:852C`) and then runs, its table at `$90:8324`.

use clean_types::ClipEnd;
use data::pose::PoseId;
use data::pose::animation::{
    AnimationFrame, BySpeed, DelayItem, DelayList, Instruction, ItemBits, wall_jump_frame,
};
use data::pose::transition::Control;

use crate::pose::{Content, PoseRequests, controls};
use crate::state::Controller;
use crate::state::samus::{
    AnimationTimer, Dashing, Item, Items, MovementType, NewPoseFrame, Samus, SuperSpecial,
    SuperSpecialCommand,
};

/// The poses the auto-jump transition gives way to: the jumps from the ground.
const JUMPS: [PoseId; 4] = [PoseId(0x4B), PoseId(0x4C), PoseId(0x19), PoseId(0x1A)];

/// The frame a spin jump from a spin or wall jump starts on, past the jump from the floor
/// (`$91:F634`).
const SPIN_FROM_SPIN: AnimationFrame = AnimationFrame(1);

/// The health at or above which the healthy loop goes back to the start (`$90:8346`).
const HEALTHY: u16 = 30;

/// How far on from the wall jump's instruction its sequences start: the spin's, the space
/// jump's and the screw attack's (`$90:845C`, `$90:846E`, `$90:8482`).
const WALL_JUMP_SPIN: u16 = 1;
/// The space jump's.
const WALL_JUMP_SPACE_JUMP: u16 = 0x0B;
/// The screw attack's.
const WALL_JUMP_SCREW_ATTACK: u16 = 0x15;

/// What the frame buffer holds: the X speed divisor, which FX "none" copies in every frame
/// (`$90:8000`).
// open: the X speed divisor (`SAMUS_X_SPEED_DIVISOR`) is 0 outside liquids and the low
// gravity rooms; it joins the State with the liquids (C10), the buffer's other writers with
// the FX.
pub(crate) const fn frame_buffer(_samus: &Samus) -> u16 {
    0
}

/// Whether her running momentum runs the running list instead of her pose's: she is
/// dashing and runs (`$90:84E3`).
fn running_momentum(samus: &Samus) -> bool {
    matches!(samus.movement, MovementType::Running) && samus.motion.dashing == Dashing::On
}

/// The falling poses whose animation jumps to its terminal velocity frame (`$90:A58D`).
const FALLING: [PoseId; 4] = [PoseId(0x29), PoseId(0x2A), PoseId(0x67), PoseId(0x68)];

/// The fall's speed in pixels from which the falling animation shows its terminal velocity
/// frame (`$90:A5BC`).
const TERMINAL_PIXELS: i16 = 5;

/// The falling animation's terminal velocity frame and its delay (`$90:A5BC`).
const TERMINAL_FRAME: AnimationFrame = AnimationFrame(5);
/// The terminal velocity frame's delay.
const TERMINAL_DELAY: AnimationTimer = AnimationTimer(8);

/// The animation's part of the movement handler, after motion's step.
///
/// An `armed` wall jump holds the spin on the frame before the wall jump's (`$90:9D35`), and a
/// fast fall shows the terminal velocity frame (`$90:A58D`).
pub fn after_motion(samus: &mut Samus, armed: bool) {
    let animation = &mut samus.animation;
    if armed {
        animation.timer = AnimationTimer(1);
        animation.frame = AnimationFrame(wall_jump_frame(samus.pose).0.wrapping_sub(1));
    }
    let terminal = samus.motion.vertical_speed.pixel().cast_signed() >= TERMINAL_PIXELS;
    if samus.movement == MovementType::Falling
        && FALLING.contains(&samus.pose)
        && terminal
        && animation.frame.0.cast_signed() < TERMINAL_FRAME.0.cast_signed()
    {
        animation.frame = TERMINAL_FRAME;
        animation.timer = TERMINAL_DELAY;
    }
}

/// "Animate Samus" (`$90:8000`): count the frame's timer down and, when it is done, advance
/// to the next item of her pose's delay list.
// open: poses $4D/$4E (the neutral jump) hold the timer at 4 on frame 1 unless she moves
// down; it joins with the jumps' vertical direction (C3b).
pub fn animate(
    samus: &mut Samus,
    controller: &Controller,
    requests: PoseRequests,
    content: &Content,
) {
    let animation = &mut samus.animation;
    animation.timer = AnimationTimer(animation.timer.0.wrapping_sub(1));
    if animation.timer.running() {
        return;
    }
    animation.frame = AnimationFrame(animation.frame.0.wrapping_add(1));
    handle_delay(samus, controller, requests, content);
}

/// Whether her spin's sprite has turned far enough to kick off a wall (`WallJump`): its frame
/// is at or past the wall jump's. The original compares the frame and branches on the
/// difference's sign.
#[must_use]
pub fn spin_turned_far(samus: &Samus) -> bool {
    samus
        .animation
        .frame
        .0
        .wrapping_sub(wall_jump_frame(samus.pose).0)
        .cast_signed()
        >= 0
}

/// The end `animate` brings her pose's animation to this frame, if any (`AnimationEnds`).
///
/// The timer runs out and the next item is an instruction that hands over a pose, which the
/// running list doesn't take over. The auto-jump transition is the end unless a jump: whether
/// the pad's jump takes its place is samus-motion's to decide. Read before the moves, so a
/// frame the moves change (an armed wall jump, a fast fall: `after_motion`) is read as it
/// stood.
#[must_use]
pub fn ends_this_frame(
    samus: &Samus,
    controller: &Controller,
    content: &Content,
) -> Option<ClipEnd> {
    let animation = &samus.animation;
    if AnimationTimer(animation.timer.0.wrapping_sub(1)).running() {
        return None;
    }
    let next = AnimationFrame(animation.frame.0.wrapping_add(1));
    let Some(DelayItem::Instruction(instruction)) =
        content.delays.of(samus.pose).and_then(|list| list.at(next))
    else {
        return None;
    };
    if running_momentum(samus) && controls(controller.held).contains(Control::Run) {
        return None;
    }
    match instruction {
        Instruction::Transition(_)
        | Instruction::BySpeed(_)
        | Instruction::ByItemsAndSpeed { .. } => Some(ClipEnd::End),
        Instruction::AutoJumpTransition(_) => Some(ClipEnd::EndUnlessJump),
        Instruction::NoOp(_)
        | Instruction::LoopIfHealthy
        | Instruction::DrainedFalling
        | Instruction::WallJump
        | Instruction::ByItems { .. }
        | Instruction::GoBack(_)
        | Instruction::Loop => None,
    }
}

/// "Handle Samus animation delay" (`$90:82DC`): the item at the frame.
fn handle_delay(
    samus: &mut Samus,
    controller: &Controller,
    requests: PoseRequests,
    content: &Content,
) {
    let Some(list) = content.delays.of(samus.pose) else {
        return;
    };
    let buffer = frame_buffer(samus);
    // A frame past the decoded list reads bytes the original reads past it; no list in the
    // slice reaches one.
    let Some(item) = list.at(samus.animation.frame) else {
        return;
    };
    let instruction = match item {
        DelayItem::Delay(_) => {
            let list = if running_momentum(samus) {
                content.delays.running()
            } else {
                list
            };
            set_timer(samus, list, buffer);
            return;
        }
        DelayItem::Instruction(instruction) => instruction,
    };
    // `$90:852C`: running with momentum and the run button held, the running list takes over
    // from its start and the instruction doesn't run.
    // open: with the speed booster the boost's stages (`$91:B5DE`) take over instead and
    // count the boost up; it joins with the speed booster (C3b).
    if running_momentum(samus) && controls(controller.held).contains(Control::Run) {
        samus.animation.frame = AnimationFrame::default();
        set_timer(samus, content.delays.running(), buffer);
        return;
    }
    if run(instruction, samus, requests) {
        set_timer(samus, list, buffer);
    }
}

/// The timer from `list`'s byte at the frame, plus the buffer (`$90:84E3`, `$90:8315`).
fn set_timer(samus: &mut Samus, list: &DelayList, buffer: u16) {
    if let Some(byte) = list.byte(samus.animation.frame) {
        samus.animation.timer = AnimationTimer(buffer.wrapping_add(u16::from(byte)));
    }
}

/// Run an instruction; whether the timer then comes from the byte at the new frame (the
/// original's carry).
fn run(instruction: Instruction, samus: &mut Samus, requests: PoseRequests) -> bool {
    let frame = &mut samus.animation.frame;
    match instruction {
        Instruction::LoopIfHealthy => {
            *frame = if samus.equipment.energy.current >= HEALTHY {
                AnimationFrame::default()
            } else {
                AnimationFrame(frame.0.wrapping_add(1))
            };
            true
        }
        Instruction::GoBack(back) => {
            *frame = AnimationFrame(frame.0.wrapping_sub(u16::from(back)));
            true
        }
        Instruction::Loop => {
            *frame = AnimationFrame::default();
            true
        }
        Instruction::Transition(pose) => {
            finish(samus, pose);
            false
        }
        // `$90:8370`: unless the frame's prospective pose is a jump from the ground, the
        // auto-jump hack takes the next frame's input, then the transition.
        // open: the demos skip the hack; the hack's input handler (it presses jump for a
        // jump the auto-jump timer still allows) joins with the jumps (C3b). Until then
        // the next frame's input runs as normal, as it does with the timer at 0.
        Instruction::AutoJumpTransition(pose) => {
            if !requests
                .ordinary
                .is_some_and(|asked| JUMPS.contains(&asked))
            {
                finish(samus, pose);
            }
            false
        }
        Instruction::BySpeed(poses) => {
            finish(samus, by_speed(samus, poses));
            false
        }
        Instruction::WallJump => {
            *frame = AnimationFrame(
                frame
                    .0
                    .wrapping_add(wall_jump_sequence(samus.equipment.equipped)),
            );
            true
        }
        // `$90:839A`: holding any of the items picks the pair of poses by her Y speed.
        Instruction::ByItemsAndSpeed {
            items,
            without,
            with,
        } => {
            let poses = if holds_any(samus.equipment.equipped, items) {
                with
            } else {
                without
            };
            finish(samus, by_speed(samus, poses));
            false
        }
        // open: the drained falling movement handler (`$90:8360`) joins with the metroid
        // drain (C4); the item-dependent transition (`$90:848B`) with the equipment's item
        // words (C3b). No list in the slice reaches them; they leave the frame and the
        // timer as they are, as the no-ops do.
        Instruction::NoOp(_) | Instruction::DrainedFalling | Instruction::ByItems { .. } => false,
    }
}

/// "Animation delay instruction Bh - select animation delay sequence for wall-jump"
/// (`$90:841D`): out of liquids or in the gravity suit, the screw attack's sequence, else
/// the space jump's, else the spin's; submerged without the gravity suit, the spin's.
// open: the submerged check (her top boundary below the FX or lava/acid surface, with
// liquid physics on, `$90:8425`) joins with the liquids (C10), and the sound each sequence
// queues (spin $31, space jump $3E, screw attack $33, `$80:9049`) with the sound queues.
// Out of liquids she is never submerged.
const fn wall_jump_sequence(equipped: Items) -> u16 {
    if equipped.contains(Item::ScrewAttack) {
        WALL_JUMP_SCREW_ATTACK
    } else if equipped.contains(Item::SpaceJump) {
        WALL_JUMP_SPACE_JUMP
    } else {
        WALL_JUMP_SPIN
    }
}

/// The items' bits in the original's equipped items word, the form the delay lists' operands
/// are in (bank $84 log: the item PLMs' equipment instruction).
const ITEM_WORD_BITS: [(Item, u16); 11] = [
    (Item::VariaSuit, 0x0001),
    (Item::SpringBall, 0x0002),
    (Item::MorphBall, 0x0004),
    (Item::ScrewAttack, 0x0008),
    (Item::GravitySuit, 0x0020),
    (Item::HiJumpBoots, 0x0100),
    (Item::SpaceJump, 0x0200),
    (Item::Bombs, 0x1000),
    (Item::SpeedBooster, 0x2000),
    (Item::GrappleBeam, 0x4000),
    (Item::XRayScope, 0x8000),
];

/// Whether `equipped` holds any of the items an operand names.
fn holds_any(equipped: Items, named: ItemBits) -> bool {
    ITEM_WORD_BITS
        .iter()
        .any(|&(item, bit)| (named.0 & bit) != 0 && equipped.contains(item))
}

/// The pose a transition by speed takes: still when her Y speed is zero.
const fn by_speed(samus: &Samus, poses: BySpeed) -> PoseId {
    if samus.motion.vertical_speed.0 == 0 {
        poses.still
    } else {
        poses.moving
    }
}

/// The animation finished: ask for `pose` as the super special prospective pose
/// (`$90:84B6`).
const fn finish(samus: &mut Samus, pose: PoseId) {
    samus.super_special = SuperSpecial {
        pose: Some(pose),
        command: SuperSpecialCommand::TransitionAnimationFinished,
    };
}

/// The frame a new pose's animation starts on when it is not the first: running from
/// running, the morph ball from the morph ball and the spring ball from the spring ball keep
/// the animation going (`$91:F50C`, `$91:F9F4`), and a spin jump from a spin or wall jump
/// skips the jump from the floor (`$91:F624`).
// open: $15/$16 from $55/$56 (`$91:F5AF`) and standing aiming up (`$91:F4EE`) start on
// frame 1; they join with C3b.
pub(crate) const fn new_pose_frame(from: MovementType, to: MovementType) -> Option<NewPoseFrame> {
    match (from, to) {
        (MovementType::Running, MovementType::Running)
        | (
            MovementType::MorphBallOnGround | MovementType::MorphBallFalling,
            MovementType::MorphBallOnGround | MovementType::MorphBallFalling,
        )
        | (
            MovementType::SpringBallOnGround
            | MovementType::SpringBallInAir
            | MovementType::SpringBallFalling,
            MovementType::SpringBallOnGround
            | MovementType::SpringBallInAir
            | MovementType::SpringBallFalling,
        ) => Some(NewPoseFrame::Preserve),
        (MovementType::SpinJumping | MovementType::WallJumping, MovementType::SpinJumping) => {
            Some(NewPoseFrame::From(SPIN_FROM_SPIN))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;
