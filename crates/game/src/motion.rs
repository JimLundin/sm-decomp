//! Samus's motion in the Tick: the model's inputs from her pose, the pad, her items and the
//! elevator, and its values from the content and the Session's switches.

use data::movement::{AirMotion, GroundMotion};
use data::pose::animation::wall_jump_frame;
use data::pose::transition::Control;

use crate::collision::world::{World, X_RADIUS};
use crate::model::samus::motion::{
    Contacts, Equipment, Facing, Inputs, JumpButton, Locomotion, MissingSpeeds, MotionRules, Pad,
    Radius, Sideways, Spin,
};
use crate::pose::{Content, FACING_FORWARD, JUMP_TAKE_OFF, controls};
use crate::session::SpeedCap;
use crate::state::room::{ElevatorRide, ElevatorStatus};
use crate::state::samus::{Facing as PoseFacing, Item, MovementType, PoseId, Samus};
use crate::state::{Button, Controller};

/// The standing poses without aim that aren't landings.
const PLAIN_STANDING: [PoseId; 2] = [PoseId(0x01), PoseId(0x02)];

/// The still morph ball, facing right and left.
const MORPH_BALL_AT_REST: [PoseId; 2] = [PoseId(0x1D), PoseId(0x41)];

/// The still spring ball, facing right and left.
const SPRING_BALL_AT_REST: [PoseId; 2] = [PoseId(0x79), PoseId(0x7A)];

/// One frame of motion as the pose change reads it: what the model was given, its values,
/// and what the frame's step met.
#[derive(Clone, Copy, Debug)]
pub struct Motion<'c> {
    /// The inputs, unless her movement type is outside the model.
    pub inputs: Option<Inputs>,
    /// The values.
    pub rules: MotionRules<'c>,
    /// What the step met.
    pub contacts: Contacts,
}

/// The values motion reads, under `speed_cap`.
#[must_use]
pub const fn rules(content: &Content, speed_cap: SpeedCap) -> MotionRules<'_> {
    MotionRules {
        speeds: &content.speeds,
        vertical: content.vertical,
        ground: GroundMotion::VANILLA,
        air: AirMotion::VANILLA,
        speed_cap,
    }
}

/// The way of moving her movement type and pose give; `None` for the types motion doesn't
/// move (knockback, grapple, shinespark and the unused ones), which no step moves.
#[must_use]
pub fn locomotion(movement: MovementType, pose: PoseId) -> Option<Locomotion> {
    use {Locomotion as L, MovementType as M};
    Some(match movement {
        M::Standing if FACING_FORWARD.contains(&pose) => L::FacingForward,
        M::Standing => L::Standing {
            plain: PLAIN_STANDING.contains(&pose),
        },
        M::Running => L::Running,
        M::NormalJumping => L::Jumping {
            take_off: JUMP_TAKE_OFF.contains(&pose),
        },
        M::SpinJumping => L::SpinJumping,
        M::MorphBallOnGround => L::Ball {
            spring: false,
            at_rest: MORPH_BALL_AT_REST.contains(&pose),
        },
        M::Crouching => L::Crouching,
        M::Falling => L::Falling,
        M::MorphBallFalling => L::BallInAir {
            spring: false,
            falling: true,
        },
        M::TurningAroundOnGround => L::TurningOnGround,
        M::CrouchingTransition => L::Transition,
        M::Moonwalking => L::Moonwalking,
        M::SpringBallOnGround => L::Ball {
            spring: true,
            at_rest: SPRING_BALL_AT_REST.contains(&pose),
        },
        M::SpringBallInAir => L::BallInAir {
            spring: true,
            falling: false,
        },
        M::SpringBallFalling => L::BallInAir {
            spring: true,
            falling: true,
        },
        M::WallJumping => L::WallJumping,
        M::RanIntoWall => L::RanIntoWall,
        M::TurningAroundJumping | M::TurningAroundFalling => L::TurningInAir,
        M::KnockbackOrCrystalFlashEnding
        | M::Grappling
        | M::DamageBoost
        | M::GrabbedByDraygon
        | M::Shinespark
        | M::Unused07
        | M::Unused09
        | M::Unused0B
        | M::Unused0C
        | M::Unused0D => return None,
    })
}

/// What motion is given this frame, unless her movement type is outside the model.
#[must_use]
pub fn inputs(samus: &Samus, controller: &Controller, ride: ElevatorRide) -> Option<Inputs> {
    let (held, pressed) = (controls(controller.held), controls(controller.pressed));
    let pad_holds = |button: Button| (controller.held.bits & button.bit()) != 0;
    let equipped = samus.equipment.equipped;
    // The original compares the frame and branches on the difference's sign.
    let threshold = wall_jump_frame(samus.pose);
    let ready = samus
        .animation
        .frame
        .0
        .wrapping_sub(threshold.0)
        .cast_signed()
        >= 0;
    Some(Inputs {
        locomotion: locomotion(samus.movement, samus.pose)?,
        facing: match samus.facing {
            PoseFacing::Left => Facing::Left,
            PoseFacing::Right => Facing::Right,
            PoseFacing::Forward | PoseFacing::Unknown(_) => Facing::Forward,
        },
        radius: Radius {
            x: X_RADIUS,
            y: samus.y_radius,
        },
        pad: Pad {
            run: held.contains(Control::Run),
            shoot: held.contains(Control::Shoot),
            jump: match (
                held.contains(Control::Jump),
                pressed.contains(Control::Jump),
            ) {
                (_, true) => JumpButton::Pressed,
                (true, false) => JumpButton::Held,
                (false, false) => JumpButton::Released,
            },
            sideways: Sideways {
                left: pad_holds(Button::Left),
                right: pad_holds(Button::Right),
            },
        },
        spin: Spin {
            ready,
            was_spinning: matches!(
                samus.last_different.movement,
                MovementType::SpinJumping | MovementType::WallJumping
            ),
        },
        equipment: Equipment {
            hi_jump: equipped.contains(Item::HiJumpBoots),
            space_jump: equipped.contains(Item::SpaceJump),
        },
        elevator_moving: ride.status != ElevatorStatus::Inactive,
        on_slope: samus.slope_adjusted,
    })
}

/// The frame's step, through `world`.
///
/// # Errors
///
/// [`MissingSpeeds`] when her way of moving has no speed table entry.
pub fn step<'c>(
    samus: &mut Samus,
    inputs: Option<Inputs>,
    rules: MotionRules<'c>,
    world: &mut World<'_, '_>,
) -> Result<Motion<'c>, MissingSpeeds> {
    let contacts = match &inputs {
        Some(inputs) => samus.motion.step(inputs, &rules, world)?,
        None => Contacts::default(),
    };
    samus.slope_adjusted = world.slope_adjusted;
    Ok(Motion {
        inputs,
        rules,
        contacts,
    })
}
