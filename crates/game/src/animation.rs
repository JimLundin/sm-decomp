//! Samus's animation: each frame's delay counts down, and the delay list's next item shows
//! the next frame or runs an instruction that loops, goes back or ends the animation in a
//! new pose.
//!
//! Bank $90 log (<https://patrickjohnston.org/bank/90>): "Animate Samus" (`$90:8000`) runs
//! between the movement handler and the pose update (`$90:E725`). It counts the timer down
//! and, when the frame is done, advances it and runs "Handle Samus animation delay"
//! (`$90:82DC`): a delay byte sets the timer (`$90:84E3`), an instruction byte first passes
//! the running momentum check (`$90:852C`) and then runs, its table at `$90:8324`.

use data::pose::PoseId;
use data::pose::animation::{AnimationFrame, BySpeed, DelayItem, DelayList, Instruction};
use data::pose::transition::Control;

use crate::pose::{Content, PoseRequests, controls};
use crate::state::Controller;
use crate::state::samus::{
    AnimationTimer, Movement, MovementType, Samus, SuperSpecial, SuperSpecialCommand,
};

/// The poses the auto-jump transition gives way to: the jumps from the ground.
const JUMPS: [PoseId; 4] = [PoseId(0x4B), PoseId(0x4C), PoseId(0x19), PoseId(0x1A)];

/// The health at or above which the healthy loop goes back to the start (`$90:8346`).
const HEALTHY: u16 = 30;

/// What the frame buffer holds: the X speed divisor, which FX "none" copies in every frame
/// (`$90:8000`).
// open: the X speed divisor (`SAMUS_X_SPEED_DIVISOR`) is 0 outside liquids and the low
// gravity rooms; it joins the State with the liquids (C10), the buffer's other writers with
// the FX.
pub(crate) const fn frame_buffer(_samus: &Samus) -> u16 {
    0
}

/// Whether her running momentum runs the running list instead of her pose's: she has
/// momentum and runs (`$90:84E3`).
const fn running_momentum(samus: &Samus) -> bool {
    matches!(
        samus.movement,
        Movement::Running { speed } if speed.momentum
    )
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
        // open: the drained falling movement handler (`$90:8360`) joins with the metroid
        // drain (C4); the wall jump's
        // sequence (`$90:841D`) with the wall jump (C3b); the item-dependent transitions
        // (`$90:839A`, `$90:848B`) with the equipment's item words (C3b). No list in the
        // slice reaches them; they leave the frame and the timer as they are, as the no-ops do.
        Instruction::NoOp(_)
        | Instruction::DrainedFalling
        | Instruction::WallJump
        | Instruction::ByItemsAndSpeed { .. }
        | Instruction::ByItems { .. } => false,
    }
}

/// The pose a transition by speed takes: still when her Y speed is zero.
const fn by_speed(samus: &Samus, poses: BySpeed) -> PoseId {
    match samus.movement.vertical() {
        Some(vertical) if vertical.speed.0 != 0 => poses.moving,
        Some(_) | None => poses.still,
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

/// Whether the new pose frame keeps the animation going from the last pose: running from
/// running, the morph ball from the morph ball, the spring ball from the spring ball
/// (`$91:F50C`, `$91:F9F4`).
// open: the spin jump from a spin or wall jump (`$91:F624`), $15/$16 from $55/$56
// (`$91:F5AF`) and standing aiming up (`$91:F4EE`) start on frame 1; they join with C3b.
pub(crate) const fn preserves(from: MovementType, to: MovementType) -> bool {
    matches!(
        (from, to),
        (MovementType::Running, MovementType::Running)
            | (
                MovementType::MorphBallOnGround | MovementType::MorphBallFalling,
                MovementType::MorphBallOnGround | MovementType::MorphBallFalling
            )
            | (
                MovementType::SpringBallOnGround
                    | MovementType::SpringBallInAir
                    | MovementType::SpringBallFalling,
                MovementType::SpringBallOnGround
                    | MovementType::SpringBallInAir
                    | MovementType::SpringBallFalling
            )
    )
}

#[cfg(test)]
mod tests;
