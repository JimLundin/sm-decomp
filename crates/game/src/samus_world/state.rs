//! The samus-motion model's Samus from the engine's State, and back
//! (`docs/behaviours/samus-motion.decode.md`, "State").
//!
//! The engine keeps her motion as the game keeps it, so the model's Samus is rebuilt from it
//! each frame and written back after the step. What she remembers maps to the game's own
//! words: carrying on and skidding to the speed mode, her dashing to the dashing flag,
//! her signed vertical speed to a magnitude and a direction, the moonfall to no vertical
//! direction in the air, falling at 0 on the ground to direction down at 0, a spin carried on
//! (its frame count) to the pose history, the seat to the slope flag, jump held through to its
//! count and the pose input the frame before left.
//!
//! The original counts jump held through once each gameplay frame at the draw (`$90:EB35` →
//! `$90:EAB3`), whatever her pose; the engine counts it only on frames the model runs, so a pose
//! outside the domain leaves the count where it was (docs/LEGACY.md, "Jump held through outside
//! her motion's poses").

use clean_model_samus_motion::{
    Bounces, Carry, Dashing, FrameBefore, JumpHeld, Moment, Phase, Run, Samus as Model, Seat, Spin,
    Vertical,
};
use core::num::Wrapping;

use clean_types::{Action, Buttons, Fixed, Pose, Position, Px, Set, Side, Turn, ZERO};
use data::assets::samus::import::map;
use data::assets::samus::key::{Dress, SpinSprite, Suit};

use crate::model::num::Fixed as Exact;
use crate::state::samus::{
    self, Bounce, Item, MovementType, PoseId, Samus, SpeedMode, VerticalMotion,
};

/// Her body's half-width, the same in every pose.
pub const X_RADIUS: Px = Wrapping(5);

/// The model's quantity from the engine's.
const fn quantity(value: Exact) -> Fixed {
    Wrapping(value.0)
}

/// The engine's quantity from the model's.
const fn exact(value: Fixed) -> Exact {
    Exact(value.0)
}

/// Whether `pose` is one she is in the air in: a jump, a spin, a fall or a wall jump, the
/// decode notes' airborne movement types. Elsewhere she is on the ground, keeping any vertical
/// speed.
const fn in_the_air(pose: Pose) -> bool {
    matches!(
        pose.action,
        Action::Jumping | Action::Spinning | Action::Falling | Action::WallJumping
    )
}

/// Whether `pose` turns her in a jump or a fall: in the air while she falls. With no vertical
/// direction and a speed she kept, it is a turn in the moonfall (chain-0232-D5A7 38 -> 44),
/// whose handler pulls her as the ground's does.
const fn turning_in_the_air(pose: Pose) -> bool {
    matches!(pose.action, Action::Turning(Turn::Jump | Turn::Fall))
}

/// The model's Samus as the frame starts; `None` when her pose is outside the domain.
#[must_use]
pub fn clean_of(samus: &Samus) -> Option<Model> {
    let pose = map::key_of(samus.pose)?;
    let motion = &samus.motion;
    let (carry, way) = match motion.mode {
        SpeedMode::Accelerating => (Carry::FollowsPad, Side::of(pose.facing)),
        SpeedMode::Decelerating => (Carry::CarriesOn, Side::of(pose.facing)),
        SpeedMode::TurningAround => (Carry::CarriesOn, Side::of(pose.facing.back())),
    };
    // The word is a magnitude, downward with direction down and upward otherwise; a rise the
    // pull took past zero is then downward by what it took past. On the ground, direction down
    // at 0 is falling at 0, as a bomb jump's or a knockback's end leaves her (BG2).
    let magnitude = quantity(motion.vertical_speed);
    let (phase, speed) = match motion.vertical {
        VerticalMotion::None
            if in_the_air(pose) || turning_in_the_air(pose) && magnitude != ZERO =>
        {
            (Phase::Moonfall, -magnitude)
        }
        VerticalMotion::None => (Phase::Ground, -magnitude),
        VerticalMotion::Up => (Phase::Rising, -magnitude),
        VerticalMotion::Down if in_the_air(pose) || turning_in_the_air(pose) => {
            (Phase::Falling, magnitude)
        }
        VerticalMotion::Down if magnitude == ZERO => (Phase::GroundFalling, ZERO),
        VerticalMotion::Down => (Phase::Ground, magnitude),
    };
    // The game's spin is carried on when the pose before the last spin pose was a spin or a
    // wall jump; the model's First and Second both mean it isn't yet.
    let carried = matches!(
        samus.last_different.movement,
        MovementType::SpinJumping | MovementType::WallJumping
    );
    Some(Model {
        position: Position {
            x: quantity(motion.position.x),
            y: quantity(motion.position.y),
        },
        run: Run {
            speed: quantity(motion.base_speed),
            way,
            carry,
            dash: quantity(motion.extra_run),
            dashing: match motion.dashing {
                samus::Dashing::On => Dashing::On,
                samus::Dashing::Off => Dashing::Off,
            },
        },
        vertical: Vertical {
            speed,
            phase,
            bounces: match motion.bounce {
                Bounce::None => Bounces::None,
                Bounce::First => Bounces::Once,
                Bounce::Second => Bounces::Twice,
            },
        },
        pose,
        spin: if pose.action == Action::Spinning && carried {
            Spin::CarriedOn
        } else {
            Spin::Off
        },
        seat: if samus.slope_adjusted {
            Seat::Seated
        } else {
            Seat::Unseated
        },
        jump_held: JumpHeld {
            frames: Wrapping(samus.jump_held.frames),
            before: match samus.jump_held.before {
                samus::FrameBefore::LandedOrEnded => FrameBefore::LandedOrEnded,
                samus::FrameBefore::Other => FrameBefore::Other,
            },
        },
    })
}

/// Whether a situation of the model names `pose`, the pose map's for one of hers.
///
/// Every pose the map holds is named (its test pins it), so the step's
/// `SamusFault::UnnamedPose` can't happen; the harness asks this to count such a frame as a
/// defect. The chart answers for every pose a situation names, whatever the frame, so an empty
/// frame asks it.
#[must_use]
pub fn named(pose: Pose) -> bool {
    let still = Moment {
        pad: Buttons::default(),
        pressed: Buttons::default(),
        items: Set::default(),
        animation_ends: None,
        speed: ZERO,
        airborne: false,
        met: None,
    };
    still.next_pose(pose).is_ok()
}

/// The engine's motion after the model's step, `rising` its `Outcome::rising`: the quantities
/// as the game keeps them.
pub fn write_back(model: &Model, rising: bool, samus: &mut Samus) {
    let motion = &mut samus.motion;
    motion.position.x = exact(model.position.x);
    motion.position.y = exact(model.position.y);
    motion.base_speed = exact(model.run.speed);
    motion.extra_run = exact(model.run.dash);
    motion.dashing = match model.run.dashing {
        Dashing::On => samus::Dashing::On,
        Dashing::Off => samus::Dashing::Off,
    };
    motion.mode = if model.run.carry == Carry::FollowsPad {
        SpeedMode::Accelerating
    } else if model.run.way == Side::of(model.pose.facing) {
        SpeedMode::Decelerating
    } else {
        SpeedMode::TurningAround
    };
    // Her vertical direction is up exactly when the model says she is rising, a rise she keeps
    // on the ground included, so the camera reads the model's rising in it. Otherwise on the
    // ground the game leaves her direction as it was: a speed she keeps with no direction stays
    // so (a turn in a moonfall, chain-0232-D5A7 38 -> 44); another is downward, and none is no
    // direction. Falling at 0 on the ground is downward.
    let Vertical {
        speed,
        phase,
        bounces,
    } = model.vertical;
    let kept = motion.vertical;
    (motion.vertical, motion.vertical_speed) = match phase {
        _ if rising => (VerticalMotion::Up, exact(-speed)),
        Phase::Moonfall => (VerticalMotion::None, exact(-speed)),
        Phase::Ground if speed == ZERO => (VerticalMotion::None, Exact(0)),
        Phase::Ground if kept == VerticalMotion::None => (VerticalMotion::None, exact(-speed)),
        Phase::Rising | Phase::Falling | Phase::GroundFalling | Phase::Ground => {
            (VerticalMotion::Down, exact(speed))
        }
    };
    motion.bounce = match bounces {
        Bounces::None => Bounce::None,
        Bounces::Once => Bounce::First,
        Bounces::Twice => Bounce::Second,
    };
    samus.slope_adjusted = model.seat == Seat::Seated;
    samus.jump_held = samus::JumpHeld {
        frames: model.jump_held.frames.0,
        before: match model.jump_held.before {
            FrameBefore::LandedOrEnded => samus::FrameBefore::LandedOrEnded,
            FrameBefore::Other => samus::FrameBefore::Other,
        },
    };
}

/// What her sprite shows that her pose doesn't decide, from her equipment; never in a liquid,
/// which the engine doesn't have yet.
#[must_use]
pub const fn dress(samus: &Samus) -> Dress {
    let equipped = samus.equipment.equipped;
    Dress {
        spin: if equipped.contains(Item::ScrewAttack) {
            SpinSprite::ScrewAttack
        } else if equipped.contains(Item::SpaceJump) {
            SpinSprite::SpaceJump
        } else {
            SpinSprite::Plain
        },
        suit: if equipped.contains(Item::VariaSuit) || equipped.contains(Item::GravitySuit) {
            Suit::VariaOrGravity
        } else {
            Suit::First
        },
    }
}

/// The engine's pose for the model's: the one she is in or the one asked of her when either
/// shows it, else the map's for her dress.
#[must_use]
pub fn pose_id(pose: Pose, samus: &Samus, asked: Option<PoseId>) -> Option<PoseId> {
    let shows = |id: PoseId| map::key_of(id) == Some(pose);
    [Some(samus.pose), asked]
        .into_iter()
        .flatten()
        .find(|&id| shows(id))
        .or_else(|| map::id_of(pose, dress(samus)))
}

#[cfg(test)]
mod tests;
