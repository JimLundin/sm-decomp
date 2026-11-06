//! Samus in the air: the normal, spin and wall jumps and the fall, their horizontal motion
//! and the vertical motion they share.
//!
//! Bank $90 log (<https://patrickjohnston.org/bank/90>): normal jumping (`$90:A42E`) and wall
//! jumping (`$90:A734`) run "Samus jumping movement" (`$90:8FB3`), spin jumping (`$90:A436`)
//! "Samus spin jumping movement" (`$90:9040`), falling (`$90:A58D`) "Samus falling movement"
//! (`$90:9168`). Each takes the extra run speed (`$90:973E`), the base speed without
//! deceleration (`$90:9B1F`) and the move (`$90:8EA9`), then "Samus Y movement - with speed
//! calculations" (`$90:90E2`). Gravity is "Determine Samus Y acceleration" (`$90:9C5B`), set
//! every frame before the movement handler, so the State keeps none.

use data::movement::{Speed, SpeedEntry, VerticalSpeeds};
use data::pose::animation::wall_jump_frame;
use data::pose::transition::Control;

use super::{Mover, Rules, move_horizontally, over_cap};
use crate::collision::solid_enemy::Direction;
use crate::collision::{Heading, VerticalContact};
use crate::pose::{Content, JUMP_TAKE_OFF, Takeoff, UNUSED_TRANSITIONS, controls};
use crate::state::samus::{
    Acceleration, Animation, AnimationFrame, AnimationTimer, Facing, Horizontal, Item, Jump,
    Movement, MovementType, Point, PoseId, PoseRecord, Samus, Vertical, YDirection,
};
use crate::state::{Button, Controller};

/// The falling poses whose animation jumps to its terminal velocity frame (`$90:A58D`).
const FALLING: [PoseId; 4] = [PoseId(0x29), PoseId(0x2A), PoseId(0x67), PoseId(0x68)];

/// The fall's speed in pixels from which gravity adds no more (`$90:910F`).
const TERMINAL_PIXELS: u16 = 5;

/// The falling animation's terminal velocity frame and its delay (`$90:A5BC`).
const TERMINAL_FRAME: AnimationFrame = AnimationFrame(5);
/// The terminal velocity frame's delay.
const TERMINAL_DELAY: AnimationTimer = AnimationTimer(8);

/// The wall jump check's reach from her side (`$90:9E9F`, "maximum distance from wall for
/// wall-jump").
const WALL_JUMP_REACH: u16 = 8;

/// What the air movements read from the pad: the jump control, held and newly pressed, and
/// the D-pad's left and right, which the original tests on the pad itself (`$90:9012`).
#[derive(Clone, Copy)]
struct Pad {
    jump: JumpControl,
    left: bool,
    right: bool,
}

/// The jump control on the pad.
#[derive(Clone, Copy)]
struct JumpControl {
    held: bool,
    /// Newly pressed this frame.
    pressed: bool,
}

impl Pad {
    fn of(controller: &Controller) -> Self {
        let held = |button: Button| (controller.held.bits & button.bit()) != 0;
        Self {
            jump: JumpControl {
                held: controls(controller.held).contains(Control::Jump),
                pressed: controls(controller.pressed).contains(Control::Jump),
            },
            left: held(Button::Left),
            right: held(Button::Right),
        }
    }

    const fn sideways(self) -> bool {
        self.left || self.right
    }
}

/// The movement handler's air movements for the frame; the other movements are not here.
// open: the extra Y displacement (`EXTRA_SAMUS_Y_DISPLACEMENT`) is zero in slice 0; the space
// jump and screw attack (`$90:A436`) and the contact damage index (`$90:A734`) join with the
// items and C4.
pub(super) fn run(
    samus: &mut Samus,
    controller: &Controller,
    content: &Content,
    mover: &mut Mover<'_>,
) {
    let pad = Pad::of(controller);
    let rules = Rules {
        speeds: &content.speeds,
        movement: samus.movement,
        run_held: controls(controller.held).contains(Control::Run),
        equipped_booster: samus.equipment.equipped.contains(Item::SpeedBooster),
    };
    let entry = content.speeds.normal.of(samus.movement.kind());
    let gravity = content.vertical.gravity;
    let (facing, pose) = (samus.facing, samus.pose);
    let mut position = samus.position;
    match &mut samus.movement {
        Movement::Jumping {
            jump: Jump::Normal,
            speed,
            vertical,
        } => {
            rules.extra_run(speed);
            if JUMP_TAKE_OFF.contains(&pose) {
                // The take-off moves her by her extra run speed alone.
                speed.acceleration = Acceleration::Accelerating;
                move_horizontally(speed, Speed::ZERO, facing, &mut position, mover);
            } else {
                release(vertical, pad);
                air_base_speed(speed, entry);
                drift(speed, pad, facing, &mut position, mover);
                move_vertically(vertical, gravity, &mut position, mover);
            }
        }
        Movement::WallJumping { speed, vertical } => {
            rules.extra_run(speed);
            release(vertical, pad);
            air_base_speed(speed, entry);
            if speed.acceleration == Acceleration::Accelerating {
                speed.acceleration = Acceleration::Decelerating;
            }
            move_horizontally(speed, speed.base, facing, &mut position, mover);
            move_vertically(vertical, gravity, &mut position, mover);
        }
        Movement::Jumping {
            jump: Jump::Spin,
            speed,
            vertical,
        } => {
            rules.extra_run(speed);
            release(vertical, pad);
            let capped = air_base_speed(speed, entry).unwrap_or(false);
            let forwards = if facing == Facing::Left {
                pad.left
            } else {
                pad.right
            };
            if capped || speed.acceleration == Acceleration::TurningAround || forwards {
                if speed.acceleration == Acceleration::Accelerating {
                    speed.acceleration = Acceleration::Decelerating;
                }
            } else {
                speed.base = Speed::ZERO;
                speed.acceleration = Acceleration::Accelerating;
            }
            move_horizontally(speed, speed.base, facing, &mut position, mover);
            let spin = Spin::of(samus.last_different, pose, pad);
            let animation = &mut samus.animation;
            spin_vertically(spin, vertical, gravity, animation, &mut position, mover);
        }
        Movement::Falling { speed, vertical } => {
            rules.extra_run(speed);
            air_base_speed(speed, entry);
            drift(speed, pad, facing, &mut position, mover);
            started_falling(vertical);
            move_vertically(vertical, gravity, &mut position, mover);
            let terminal = vertical.speed.pixels().cast_signed() >= TERMINAL_PIXELS.cast_signed();
            let animation = &mut samus.animation;
            if FALLING.contains(&pose)
                && terminal
                && animation.frame.0.cast_signed() < TERMINAL_FRAME.0.cast_signed()
            {
                animation.frame = TERMINAL_FRAME;
                animation.timer = TERMINAL_DELAY;
            }
        }
        Movement::Transition { vertical } => {
            transition(vertical, gravity, pose, &mut position, mover);
        }
        Movement::Standing { .. }
        | Movement::Running { .. }
        | Movement::Crouching
        | Movement::MorphBall { .. }
        | Movement::SpringBall { .. }
        | Movement::KnockbackOrCrystalFlashEnding
        | Movement::TurningAround { .. }
        | Movement::Moonwalking
        | Movement::RanIntoWall
        | Movement::Grappling
        | Movement::DamageBoost
        | Movement::GrabbedByDraygon
        | Movement::Shinespark
        | Movement::Unused(_) => return,
    }
    samus.position = position;
}

/// The transition movement's Y part (`$90:A61C`): the poses it moves in (all but the unused
/// ones) take the simple Y movement (`$90:92C7`), which moves her only while she has a Y
/// direction; landing then stops her.
// open: the morph ball bounce state (`$0B20`) the landing clears has no reader until the
// morph ball's bounces; the transition's horizontal move with a zero base speed (`$90:9348`)
// and the ground's step down for a direction of none (`$90:923F`) join with her extra run
// speed.
fn transition(
    vertical: &mut Vertical,
    gravity: Speed,
    pose: PoseId,
    position: &mut Point,
    mover: &mut Mover<'_>,
) {
    if UNUSED_TRANSITIONS.contains(&pose.0) || vertical.direction == YDirection::None {
        return;
    }
    started_falling(vertical);
    move_vertically(vertical, gravity, position, mover);
    // The movement type has no grounding of its own, so a landing is only this.
    if mover.contacts.vertical == VerticalContact::Landed {
        *vertical = Vertical::default();
    }
}

/// What the wall jump check reads of a spin jump.
#[derive(Clone, Copy)]
struct Spin {
    /// Whether the last different pose's movement type spun: a spin or wall jump.
    spinning: bool,
    pose: PoseId,
    pad: Pad,
}

impl Spin {
    const fn of(last_different: PoseRecord, pose: PoseId, pad: Pad) -> Self {
        Self {
            spinning: matches!(
                last_different.movement,
                MovementType::SpinJumping | MovementType::WallJumping
            ),
            pose,
            pad,
        }
    }
}

/// The spin jump after its horizontal move (`$90:90B3`): the wall jump check, and unless it
/// triggers a wall jump, her vertical motion.
fn spin_vertically(
    spin: Spin,
    vertical: &mut Vertical,
    gravity: Speed,
    animation: &mut Animation,
    position: &mut Point,
    mover: &mut Mover<'_>,
) {
    if wall_jump_check(spin, animation, position, mover) {
        mover.contacts.vertical = VerticalContact::WallJumpTriggered;
    } else {
        move_vertically(vertical, gravity, position, mover);
    }
}

/// "Wall jump check" (`$90:9D35`), after the spin jump's horizontal move: holding a
/// direction, she looks [`WALL_JUMP_REACH`] pixels the other way for a wall. Still spinning,
/// a wall moves her animation to the frame before the wall jump's; past it, newly pressing
/// jump with the wall nearer than the reach triggers the wall jump, and she doesn't move
/// vertically this frame. Left is checked before right.
///
/// A solid enemy that way (`$A0:A8F0`, `$90:9D80`, `$90:9D92`, `$90:9DF8`, `$90:9E1B`) is the
/// wall, at the distance it cut the look to, and no block is looked for.
///
/// The look reaches blocks her move didn't, so it touches them as well: `item-morph-ball`
/// frame 817 takes the Morph Ball's item 8 pixels to her left.
// open: the enemy to shake (`$0E1C`, `$90:9E64`) has no reader until the enemies' shake.
fn wall_jump_check(
    spin: Spin,
    animation: &mut Animation,
    position: &mut Point,
    mover: &mut Mover<'_>,
) -> bool {
    if !spin.spinning {
        return false;
    }
    let reach = Speed::new(WALL_JUMP_REACH, 0);
    // Holding left, she looks right.
    let (direction, distance) = if spin.pad.left {
        (Direction::Right, reach)
    } else if spin.pad.right {
        (Direction::Left, Speed(reach.0.wrapping_neg()))
    } else {
        return false;
    };
    let threshold = wall_jump_frame(spin.pose);
    // The original compares the frame and branches on the difference's sign.
    let eligible = animation.frame.0.wrapping_sub(threshold.0).cast_signed() >= 0;
    let enemies = mover.surroundings.enemies;
    let enemy = enemies
        .check(direction, mover.body, reach, position)
        .distance();
    let Some(wall) = enemy
        .map(Speed::pixels)
        .or_else(|| mover.check_x(distance, position))
    else {
        return false;
    };
    if eligible {
        spin.pad.jump.pressed && wall.cast_signed() < WALL_JUMP_REACH.cast_signed()
    } else {
        animation.timer = AnimationTimer(1);
        animation.frame = AnimationFrame(threshold.0.wrapping_sub(1));
        false
    }
}

/// Move her by her base speed while she keeps momentum or steers; else she loses it and
/// stays where she is (`$90:900D`, `$90:9174`).
fn drift(
    speed: &mut Horizontal,
    pad: Pad,
    facing: Facing,
    position: &mut Point,
    mover: &mut Mover<'_>,
) {
    if speed.acceleration != Acceleration::Accelerating || pad.sideways() {
        move_horizontally(speed, speed.base, facing, position, mover);
    } else {
        speed.base = Speed::ZERO;
    }
}

/// "Calculate Samus X base speed - deceleration disallowed" (`$90:9B1F`): turning takes the
/// entry's deceleration away until she stops and accelerates again; otherwise she speeds up
/// to the cap, decelerating or not. `Some(true)` when she reached it; `None` for a movement
/// type past the table, whose base speed stays.
// open: the deceleration multiplier (`SAMUS_X_DECEL_MULT`) joins with the blocks that set it.
fn air_base_speed(speed: &mut Horizontal, entry: Option<&SpeedEntry>) -> Option<bool> {
    let entry = entry?;
    if speed.acceleration == Acceleration::TurningAround {
        let base = Speed(speed.base.0.wrapping_sub(entry.deceleration.0));
        if base.pixels().cast_signed() < 0 {
            speed.base = Speed::ZERO;
            speed.acceleration = Acceleration::Accelerating;
        } else {
            speed.base = base;
        }
        return Some(false);
    }
    let base = Speed(speed.base.0.wrapping_add(entry.acceleration.0));
    let capped = over_cap(base, entry.max);
    speed.base = if capped { entry.max } else { base };
    Some(capped)
}

/// The rise's release check (`$90:8FDC`): letting go of jump, or a speed gravity took below
/// zero, ends the rise; she then falls from rest.
fn release(vertical: &mut Vertical, pad: Pad) {
    if vertical.direction == YDirection::Up && (!pad.jump.held || rising_past_apex(vertical.speed))
    {
        fall_from_rest(vertical);
    }
}

/// "Check if Samus has started falling" (`$90:90C4`): the rise ends at its apex.
fn started_falling(vertical: &mut Vertical) {
    if vertical.direction == YDirection::Up && rising_past_apex(vertical.speed) {
        fall_from_rest(vertical);
    }
}

/// Whether gravity took the rise's speed below zero, read as the original reads its pixel
/// word: by its sign.
const fn rising_past_apex(speed: Speed) -> bool {
    speed.pixels().cast_signed() < 0
}

const fn fall_from_rest(vertical: &mut Vertical) {
    *vertical = Vertical {
        speed: Speed::ZERO,
        direction: YDirection::Down,
    };
}

/// "Samus Y movement - with speed calculations" (`$90:90E2`): she moves by the frame's
/// starting speed, up unless falling; gravity then takes from a rise or adds to a fall whose
/// pixel word isn't 5 yet. The move goes through her block collision (`$90:93EC` up,
/// `$90:9440` down), which lands her on floors and stops her at ceilings.
fn move_vertically(
    vertical: &mut Vertical,
    gravity: Speed,
    position: &mut Point,
    mover: &mut Mover<'_>,
) {
    let amount = vertical.speed;
    let (displacement, heading) = match vertical.direction {
        YDirection::Down => {
            if amount.pixels() != TERMINAL_PIXELS {
                vertical.speed = Speed(amount.0.wrapping_add(gravity.0));
            }
            (amount.0, Heading::Down)
        }
        YDirection::Up | YDirection::None => {
            vertical.speed = Speed(amount.0.wrapping_sub(gravity.0));
            (amount.0.wrapping_neg(), Heading::Up)
        }
    };
    mover.move_y(Speed(displacement), heading, position);
}

/// "Make Samus jump" (`$90:98BC`) and "Make Samus wall-jump" (`$90:9949`): she rises.
///
/// The initial speed is higher with the hi-jump boots and raised by half her extra run speed
/// with the speed booster (its pixel word halved, its subpixels added without a carry).
// open: water and lava/acid's initial speeds (the liquids), and the grapple wall jump and
// Ceres elevator fade timers the original clears, join with their features.
pub fn make_jump(samus: &mut Samus, takeoff: Takeoff, speeds: &VerticalSpeeds) {
    let equipped = samus.equipment.equipped;
    let hi_jump = equipped.contains(Item::HiJumpBoots);
    let initial = match (takeoff, hi_jump) {
        (Takeoff::Jump, false) => speeds.jump,
        (Takeoff::Jump, true) => speeds.hi_jump,
        (Takeoff::WallJump, false) => speeds.wall_jump,
        (Takeoff::WallJump, true) => speeds.wall_hi_jump,
    };
    let extra = samus
        .movement
        .horizontal()
        .map_or(Speed::ZERO, |speed| speed.extra_run);
    let speed = if equipped.contains(Item::SpeedBooster) {
        Speed::new(
            initial.pixels().wrapping_add(extra.pixels() >> 1),
            initial.subpixels().wrapping_add(extra.subpixels()),
        )
    } else {
        initial
    };
    if let Some(vertical) = samus.movement.vertical_mut() {
        *vertical = Vertical {
            speed,
            direction: YDirection::Up,
        };
    }
}

#[cfg(test)]
mod tests;
