//! Verification-only access to the actual private collision helper.
use super::{effects::Host, operations::MotionResult};
pub(in crate::game) fn solid_contact(h: &mut impl Host, amount: i32) -> MotionResult {
    super::geometry::solid_contact(h, amount)
}
// Verification-only forwarding of preserved predecessor entry semantics.
// This file is appended inside player_collision::verification under sm_verify_dll.
use super::operations as collision;
use crate::{game::session_runtime::Runtime, game_state::Game};

pub(crate) fn move_left(amount: i32, g: &mut Game) {
    super::motion::horizontal_contact(&mut Runtime::new(g), amount, true);
}
pub(crate) fn move_right(amount: i32, g: &mut Game) {
    super::motion::horizontal_contact(&mut Runtime::new(g), amount, false);
}
pub(crate) fn move_x(base: i32, g: &mut Game) {
    let request = collision::HorizontalRequest {
        base,
        direction: collision::HorizontalDirection::FromFacingAndAcceleration,
    };
    collision::horizontal(&mut Runtime::new(g), request);
}
macro_rules! displacement_entry {
    ($name:ident, $variant:ident, $kind:ident, $direction:ident) => {
        pub(crate) fn $name(amount: i32, g: &mut Game) {
            let request = collision::DisplacementRequest::$variant {
                direction: collision::$kind::$direction,
                amount,
            };
            collision::displace(&mut Runtime::new(g), request);
        }
    };
}
displacement_entry!(move_up, Vertical, VerticalDirection, Up);
displacement_entry!(move_down, Vertical, VerticalDirection, Down);
displacement_entry!(unchecked_left, Unchecked, Direction, Left);
displacement_entry!(unchecked_right, Unchecked, Direction, Right);
displacement_entry!(unchecked_up, Unchecked, Direction, Up);
displacement_entry!(unchecked_down, Unchecked, Direction, Down);

pub(crate) fn pose_clearance(amount: i32, g: &mut Game) -> i32 {
    let request = collision::DisplacementRequest::PoseClearanceProbe { amount };
    collision::displace(&mut Runtime::new(g), request).displacement
}
pub(crate) fn wall_jump(amount: i32, g: &mut Game) -> u8 {
    let request = collision::JumpRequest {
        kind: collision::JumpKind::Wall,
        amount,
    };
    u8::from(collision::jump_admission(&mut Runtime::new(g), request))
}
pub(crate) fn grapple_jump(amount: i32, g: &mut Game) -> u8 {
    let request = collision::JumpRequest {
        kind: collision::JumpKind::Grapple,
        amount,
    };
    u8::from(collision::jump_admission(&mut Runtime::new(g), request))
}
pub(crate) fn fit_pose(g: &mut Game) {
    collision::fit_pose(&mut Runtime::new(g));
}
pub(crate) fn walked_obstacle(g: &mut Game) {
    collision::walked_obstacle(&mut Runtime::new(g));
}
pub(crate) fn shinespark_x(g: &mut Game) {
    collision::shinespark(&mut Runtime::new(g), collision::Axis::X);
}
pub(crate) fn shinespark_y(g: &mut Game) {
    collision::shinespark(&mut Runtime::new(g), collision::Axis::Y);
}
pub(crate) fn align_slopes(g: &mut Game) {
    super::geometry::align_slopes(&mut Runtime::new(g));
}
