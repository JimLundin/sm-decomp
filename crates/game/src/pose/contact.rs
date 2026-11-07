//! The pose update's collisions: the poses the frame's moves ask for, and the arm-pump step's
//! wall.
//!
//! Bank $91 log (<https://patrickjohnston.org/bank/91>): "Set prospective Samus pose
//! according to solid vertical collision result" (`$91:E8B6`) runs between the animation
//! and the pose update, and the update checks whether the prospective pose runs into a wall
//! (`$91:EADE`). Motion gives the vertical result with its fall and landing kinds.

use data::pose::transition::Control;
use data::pose::{MovementType, Shot, ShotDirection};

use super::{Collisions, Content, PoseCommand, PoseRequests, controls};
use crate::collision::Touches;
use crate::collision::world::World;
use crate::model::samus::motion::{FallKind, LandKind, Locomotion, PoseRequest, VerticalResult};
use crate::state::Controller;
use crate::state::samus::{Facing, PoseId, Samus};

/// The falling pose facing right and facing left a fall of `kind` starts (`$91:E921`).
const fn fall_poses(kind: FallKind) -> Option<[PoseId; 2]> {
    match kind {
        FallKind::Airborne => Some([PoseId(0x29), PoseId(0x2A)]),
        FallKind::MorphBall => Some([PoseId(0x31), PoseId(0x32)]),
        FallKind::SpringBall => Some([PoseId(0x7D), PoseId(0x7E)]),
        FallKind::NoChange => None,
    }
}

/// Which of a pair of poses, right then left, `facing` takes: the original tests for
/// facing left, so every other facing takes the right one.
const fn by_facing(facing: Facing, [right, left]: [PoseId; 2]) -> PoseId {
    match facing {
        Facing::Left => left,
        Facing::Right | Facing::Forward | Facing::Unknown(_) => right,
    }
}

/// The request the frame's vertical result makes of the pose update (`$91:E8B6`).
///
/// A landing on her feet, a fall or a wall jump replaces the input's request with its pose; a
/// ceiling keeps the pose, and a ball's landing the input's request. Each passes the result
/// back as the command.
#[must_use]
pub fn request(
    samus: &Samus,
    vertical: VerticalResult,
    controller: &Controller,
    content: &Content,
    requests: PoseRequests,
) -> PoseRequests {
    let pose = match vertical {
        VerticalResult::None
        | VerticalResult::Falling(FallKind::NoChange)
        | VerticalResult::Landed(LandKind::NoChange) => return requests,
        VerticalResult::HitCeiling => Some(samus.pose),
        // The wall jump's pose by her facing (`$91:EABE`).
        VerticalResult::WallJump => Some(by_facing(samus.facing, [PoseId(0x83), PoseId(0x84)])),
        VerticalResult::Falling(kind) => {
            fall_poses(kind).map(|poses| by_facing(samus.facing, poses))
        }
        VerticalResult::Landed(LandKind::Grounded) => landing(samus, controller, content),
        VerticalResult::Landed(LandKind::MorphBall | LandKind::SpringBall) => {
            Some(requests.ordinary.unwrap_or(samus.pose))
        }
    };
    pose.map_or(requests, |pose| PoseRequests {
        ordinary: Some(pose),
        command: Some(PoseCommand::Vertical(vertical)),
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
fn landing(samus: &Samus, controller: &Controller, content: &Content) -> Option<PoseId> {
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
/// Motion's arm-pump step decides whether she ran into one: running when a wall stopped her
/// this frame, her pose's ran-into-a-wall pose; a running pose asked for, its own. A scroll
/// trigger the step's move reaches joins `touches` (`$94:906F`).
pub(super) fn runs_into_wall(
    samus: &mut Samus,
    requested: Option<PoseId>,
    collisions: Collisions<'_>,
    touches: &mut Touches,
    content: &Content,
) -> Option<PoseId> {
    let motion = collisions.motion;
    let Some(inputs) = motion.inputs else {
        return requested;
    };
    let request = requested.map_or(PoseRequest::None, |pose| PoseRequest::Pose {
        running: content
            .poses
            .get(pose)
            .is_some_and(|definition| definition.movement == MovementType::Running),
    });
    let wall_stop = motion.contacts.wall_stop;
    let mut world = World::new(collisions.surroundings, touches, samus);
    if !samus
        .motion
        .arm_pump_step(&inputs, wall_stop, request, &motion.rules, &mut world)
    {
        return requested;
    }
    let from = if wall_stop && inputs.locomotion == Locomotion::Running {
        samus.pose
    } else {
        requested?
    };
    ran_into_wall(from, content).or(requested)
}

#[cfg(test)]
mod tests;
