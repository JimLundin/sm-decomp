//! The pose update's collisions: the poses the frame's moves ask for, and what they do to
//! Samus's speeds after the change.
//!
//! Bank $91 log (<https://patrickjohnston.org/bank/91>): "Set prospective Samus pose
//! according to solid vertical collision result" (`$91:E8B6`) runs between the animation
//! and the pose update; the update checks whether the prospective pose runs into a wall
//! (`$91:EADE`) and, after the change, runs the solid vertical collision command
//! (`$91:EFC4`). The movement types' downwards results come from bank $90's tables
//! (`$90:E65A` falling, `$90:E676` landed), read by the movement type the move saw.

use data::pose::transition::Control;
use data::pose::{Shot, ShotDirection};

use super::{Content, PoseCommand, PoseRequests, controls};
use crate::collision::{self, Side, Surroundings, VerticalContact};
use crate::state::Controller;
use crate::state::samus::{
    Acceleration, Facing, MovementType, PoseId, Samus, Speed, Vertical, YDirection,
};

/// What a downwards move that met no floor makes of a movement type (`$90:E65A`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fall {
    /// Off the ground: she falls (0).
    Airborne,
    /// The morph ball rolls off (1).
    MorphBall,
    /// Unused movement type 7 (2).
    Unused,
    /// The spring ball rolls off (3).
    SpringBall,
}

impl Fall {
    /// The result for `movement`; `None` is the table's 4, no change: she was already in
    /// the air.
    #[must_use]
    pub const fn of(movement: MovementType) -> Option<Self> {
        match movement {
            MovementType::Standing
            | MovementType::Running
            | MovementType::Crouching
            | MovementType::KnockbackOrCrystalFlashEnding
            | MovementType::Unused0B
            | MovementType::Unused0C
            | MovementType::Unused0D
            | MovementType::Moonwalking
            | MovementType::RanIntoWall => Some(Self::Airborne),
            MovementType::MorphBallOnGround => Some(Self::MorphBall),
            MovementType::Unused07 => Some(Self::Unused),
            MovementType::SpringBallOnGround => Some(Self::SpringBall),
            MovementType::NormalJumping
            | MovementType::SpinJumping
            | MovementType::Falling
            | MovementType::MorphBallFalling
            | MovementType::Unused09
            | MovementType::TurningAroundOnGround
            | MovementType::CrouchingTransition
            | MovementType::SpringBallInAir
            | MovementType::SpringBallFalling
            | MovementType::WallJumping
            | MovementType::Grappling
            | MovementType::TurningAroundJumping
            | MovementType::TurningAroundFalling
            | MovementType::DamageBoost
            | MovementType::GrabbedByDraygon
            | MovementType::Shinespark => None,
        }
    }

    /// The falling pose facing right and facing left (`$91:E921`).
    const fn poses(self) -> [PoseId; 2] {
        match self {
            Self::Airborne => [PoseId(0x29), PoseId(0x2A)],
            Self::MorphBall => [PoseId(0x31), PoseId(0x32)],
            Self::Unused => [PoseId(0x33), PoseId(0x34)],
            Self::SpringBall => [PoseId(0x7D), PoseId(0x7E)],
        }
    }
}

/// What a downwards move stopped by a floor makes of a movement type (`$90:E676`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grounding {
    /// She lands on her feet (0).
    Grounded,
    /// The morph ball lands (1).
    MorphBall,
    /// Unused movement type 9 (2).
    Unused,
    /// The spring ball lands (3).
    SpringBall,
}

impl Grounding {
    /// The result for `movement`; `None` is the table's 4, no change: she was already on
    /// the ground.
    #[must_use]
    pub const fn of(movement: MovementType) -> Option<Self> {
        match movement {
            MovementType::NormalJumping
            | MovementType::SpinJumping
            | MovementType::Falling
            | MovementType::KnockbackOrCrystalFlashEnding
            | MovementType::Unused0D
            | MovementType::WallJumping
            | MovementType::DamageBoost => Some(Self::Grounded),
            MovementType::MorphBallFalling => Some(Self::MorphBall),
            MovementType::Unused09 => Some(Self::Unused),
            MovementType::SpringBallInAir | MovementType::SpringBallFalling => {
                Some(Self::SpringBall)
            }
            MovementType::Standing
            | MovementType::Running
            | MovementType::MorphBallOnGround
            | MovementType::Crouching
            | MovementType::Unused07
            | MovementType::Unused0B
            | MovementType::Unused0C
            | MovementType::TurningAroundOnGround
            | MovementType::CrouchingTransition
            | MovementType::Moonwalking
            | MovementType::SpringBallOnGround
            | MovementType::RanIntoWall
            | MovementType::Grappling
            | MovementType::TurningAroundJumping
            | MovementType::TurningAroundFalling
            | MovementType::GrabbedByDraygon
            | MovementType::Shinespark => None,
        }
    }
}

/// The solid vertical collision a pose change command carries to after the change
/// (`$91:EFC4`'s table at `$91:EFD0`).
///
/// The downwards result is the one the move stored, by the movement type before the change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolidVertical {
    /// An upwards move was stopped.
    HitCeiling,
    /// A downwards move met no floor.
    Falling,
    /// A downwards move was stopped.
    Landed(Grounding),
}

/// Which of a pair of poses, right then left, `facing` takes: the original tests for
/// facing left, so every other facing takes the right one.
const fn by_facing(facing: Facing, [right, left]: [PoseId; 2]) -> PoseId {
    match facing {
        Facing::Left => left,
        Facing::Right | Facing::Forward | Facing::Unknown(_) => right,
    }
}

/// The request the frame's vertical collision makes of the pose update (`$91:E8B6`).
///
/// A landing or a fall replaces the input's request with its pose and the solid vertical
/// collision command; a ceiling keeps the pose with the command.
// open: the morph ball's and the spring ball's landings (`$91:EA07`, `$91:EA63`) read the
// bounce state, and the wall jump's result 5 (`$91:EABE`) joins with the wall jump check;
// until then they leave the input's request.
#[must_use]
pub fn request(
    samus: &Samus,
    vertical: VerticalContact,
    controller: &Controller,
    content: &Content,
    requests: PoseRequests,
) -> PoseRequests {
    let movement = samus.movement.kind();
    let (pose, collision) = match vertical {
        VerticalContact::NoChange => return requests,
        VerticalContact::HitCeiling => (Some(samus.pose), SolidVertical::HitCeiling),
        VerticalContact::Falling => match Fall::of(movement) {
            Some(fall) => (
                Some(by_facing(samus.facing, fall.poses())),
                SolidVertical::Falling,
            ),
            None => return requests,
        },
        VerticalContact::Landed => match Grounding::of(movement) {
            Some(grounding) => (
                landing(samus, grounding, controller, content),
                SolidVertical::Landed(grounding),
            ),
            None => return requests,
        },
    };
    pose.map_or(requests, |pose| PoseRequests {
        ordinary: Some(pose),
        command: Some(PoseCommand::SolidVertical(collision)),
    })
}

/// The landing poses a normal jump's pose lands in, by the direction it shoots
/// (`$91:E9F3`).
const fn normal_landing(direction: ShotDirection) -> PoseId {
    match direction {
        ShotDirection::UpFacingRight => PoseId(0xE0),
        ShotDirection::UpRight => PoseId(0xE2),
        ShotDirection::Right | ShotDirection::DownFacingRight => PoseId(0xA4),
        ShotDirection::DownRight => PoseId(0xE4),
        ShotDirection::DownFacingLeft | ShotDirection::Left => PoseId(0xA5),
        ShotDirection::DownLeft => PoseId(0xE5),
        ShotDirection::UpLeft => PoseId(0xE3),
        ShotDirection::UpFacingLeft => PoseId(0xE1),
    }
}

/// The pose a landing asks for (`$91:E931`'s table at `$91:E951`).
///
/// On her feet (`$91:E95D`): from a spin or wall jump the spin jump's landing; else by the
/// pose's shot direction, firing sideways with shoot held, the plain landing when the pose
/// can't shoot.
fn landing(
    samus: &Samus,
    grounding: Grounding,
    controller: &Controller,
    content: &Content,
) -> Option<PoseId> {
    match grounding {
        Grounding::Grounded => {
            if matches!(
                samus.previous.movement,
                MovementType::SpinJumping | MovementType::WallJumping
            ) {
                return Some(by_facing(samus.facing, [PoseId(0xA6), PoseId(0xA7)]));
            }
            let shot = content.poses.get(samus.pose)?.shot;
            let shooting = controls(controller.held).contains(Control::Shoot);
            match shot {
                Shot::Fires(ShotDirection::Right | ShotDirection::Left) if shooting => {
                    Some(by_facing(samus.facing, [PoseId(0xE6), PoseId(0xE7)]))
                }
                Shot::Fires(direction) => Some(normal_landing(direction)),
                Shot::Withheld(0xFF) => Some(by_facing(samus.facing, [PoseId(0xA4), PoseId(0xA5)])),
                // The other withheld bytes ($FA-$FC) index past the table; no pose that
                // lands on her feet holds one.
                Shot::Withheld(_) => None,
            }
        }
        Grounding::Unused => Some(by_facing(samus.facing, [PoseId(0x20), PoseId(0x42)])),
        Grounding::MorphBall | Grounding::SpringBall => None,
    }
}

/// The ran-into-a-wall pose for a pose's shot direction (`$91:EB74`); aiming straight up,
/// she keeps to the standing pose aiming up.
const fn ran_into_wall_pose(direction: ShotDirection) -> PoseId {
    match direction {
        ShotDirection::UpFacingRight => PoseId(0x03),
        ShotDirection::UpRight => PoseId(0xCF),
        ShotDirection::Right | ShotDirection::DownFacingRight => PoseId(0x89),
        ShotDirection::DownRight => PoseId(0xD1),
        ShotDirection::DownFacingLeft | ShotDirection::Left => PoseId(0x8A),
        ShotDirection::DownLeft => PoseId(0xD2),
        ShotDirection::UpLeft => PoseId(0xD0),
        ShotDirection::UpFacingLeft => PoseId(0x04),
    }
}

/// The ran-into-a-wall pose for `pose`, if its shot has a direction; every running pose's
/// has.
fn ran_into_wall(pose: PoseId, content: &Content) -> Option<PoseId> {
    match content.poses.get(pose)?.shot {
        Shot::Fires(direction) => Some(ran_into_wall_pose(direction)),
        Shot::Withheld(_) => None,
    }
}

/// "Check if prospective pose runs into a wall" (`$91:EADE`): the pose she runs into a
/// wall in, or the request as it was.
///
/// Running when a wall stopped her this frame, her pose's ran-into-a-wall pose. Otherwise a
/// running pose asked for moves her a pixel forward through the blocks, and a block there
/// turns it into its ran-into-a-wall pose. The move happens whether or not the pose changes:
/// the arm pump bug, a pure horizontal move that can lift her off a downwards slope.
// open: the solid enemy check before the pixel's move (`$A0:A8F0`) joins with solid enemies.
pub(super) fn runs_into_wall(
    samus: &mut Samus,
    requested: Option<PoseId>,
    wall: Option<Side>,
    surroundings: Surroundings<'_>,
    content: &Content,
) -> Option<PoseId> {
    if wall.is_some() && samus.movement.kind() == MovementType::Running {
        return ran_into_wall(samus.pose, content).or(requested);
    }
    let pose = requested?;
    let running = content
        .poses
        .get(pose)
        .is_some_and(|definition| definition.movement == MovementType::Running);
    if !running {
        return requested;
    }
    let pixel = match samus.facing {
        Facing::Left => Speed::new(0xFFFF, 0),
        Facing::Right | Facing::Forward | Facing::Unknown(_) => Speed::new(1, 0),
    };
    let body = crate::movement::body(samus);
    let moved = collision::move_x(surroundings.room, body, &mut samus.position, pixel);
    if moved.collided {
        ran_into_wall(pose, content).or(requested)
    } else {
        requested
    }
}

/// "Prospective pose change command 5 - solid vertical collision" (`$91:EFC4`), after the
/// change.
///
/// A ceiling stops her rise; a fall from the ground starts with no Y speed unless she was
/// rising; a landing on her feet stops her base speed and her vertical motion.
// open: the landing's sounds and dust (`$91:F046`), the auto-jump hack the landing arms
// (`$91:F1EC`, its input handler joins with the jumps), the falling flag, the unused type's
// horizontal slope detection (`$91:F253`) and the bounces join with the sounds, the
// atmospheric graphics, the slopes' flags and the morph ball.
pub(super) const fn solid_vertical(collision: SolidVertical, samus: &mut Samus) {
    match collision {
        SolidVertical::HitCeiling => stop_vertically(samus, YDirection::Down),
        SolidVertical::Falling => {
            let rising = matches!(
                samus.movement.vertical(),
                Some(Vertical {
                    direction: YDirection::Up,
                    ..
                })
            );
            if !rising {
                stop_vertically(samus, YDirection::Down);
            }
        }
        SolidVertical::Landed(Grounding::Grounded | Grounding::Unused) => {
            if let Some(speed) = samus.movement.horizontal_mut() {
                speed.acceleration = Acceleration::Accelerating;
                speed.base = Speed::ZERO;
            }
            stop_vertically(samus, YDirection::None);
        }
        SolidVertical::Landed(Grounding::MorphBall | Grounding::SpringBall) => {}
    }
}

/// Her Y speed stops, and her Y direction becomes `direction`.
const fn stop_vertically(samus: &mut Samus, direction: YDirection) {
    if let Some(vertical) = samus.movement.vertical_mut() {
        *vertical = Vertical {
            speed: Speed::ZERO,
            direction,
        };
    }
}

#[cfg(test)]
mod tests;
