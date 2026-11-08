//! What one frame brings the samus-motion model (`clean_model_samus_motion::Frame`).
//!
//! It is read from the engine's State: the pad under the bindings, her items, her animation,
//! the elevator and the frame count, and each pose's body size from the pose table.

use clean_model_samus_motion::Frame;
use clean_types::{Buttons, Equipment, Parity, Pose, Radius, button};
use core::num::Wrapping;
use data::assets::samus::import::map;
use data::assets::samus::key::Dress;
use data::pose::transition::{Control, Controls};

use super::state::X_RADIUS;
use crate::animation;
use crate::collision::ScanOrder;
use crate::pose::{Content, PoseRequests, controls};
use crate::state::Controller;
use crate::state::room::{ElevatorRide, ElevatorStatus};
use crate::state::samus::{Item, Samus};

/// The model's pad from the controls the buttons work; item cancel moves nothing.
fn pad(held: Controls) -> Buttons {
    [
        (Control::Run, button::RUN),
        (Control::Jump, button::JUMP),
        (Control::Shoot, button::SHOOT),
        (Control::Left, button::LEFT),
        (Control::Right, button::RIGHT),
        (Control::Up, button::UP),
        (Control::Down, button::DOWN),
        (Control::AimUp, button::AIM_UP),
        (Control::AimDown, button::AIM_DOWN),
    ]
    .into_iter()
    .filter(|&(control, _)| held.contains(control))
    .fold(Buttons::default(), |pad, (_, set)| Buttons(pad.0 | set.0))
}

/// The engine's frame, as the model reads it at the start of the movement handler.
#[derive(Clone, Copy, Debug)]
pub struct FrameSource<'a> {
    /// Samus as the frame starts.
    pub samus: &'a Samus,
    /// The pad.
    pub controller: &'a Controller,
    /// The elevator under her.
    pub ride: ElevatorRide,
    /// The requested-NMI count, whose parity orders the vertical scans.
    pub completed: u16,
    /// The pose the frame's input asked for, which the auto-jump hack reads.
    pub requests: PoseRequests,
    /// The delay lists.
    pub content: &'a Content,
}

/// §1.3 body size: a pose's half-height is the pose table's for the original's pose that shows
/// it in her dress; a pose the table doesn't hold has none, as the engine's radius has none.
#[must_use]
pub fn body(content: &Content, dress: Dress, pose: Pose) -> Radius {
    Radius {
        x: X_RADIUS,
        y: Wrapping(
            map::id_of(pose, dress)
                .and_then(|id| content.poses.get(id))
                .map_or(0, |definition| u16::from(definition.y_radius)),
        ),
    }
}

impl FrameSource<'_> {
    /// The model's frame, with `body` its body sizes.
    #[must_use]
    pub fn frame<'b>(&self, body: &'b dyn Fn(Pose) -> Radius) -> Frame<'b> {
        let samus = self.samus;
        let equipped = samus.equipment.equipped;
        Frame {
            pad: pad(controls(self.controller.held)),
            pressed: pad(controls(self.controller.pressed)),
            equipment: Equipment {
                hi_jump: equipped.contains(Item::HiJumpBoots),
                space_jump: equipped.contains(Item::SpaceJump),
                spring_ball: equipped.contains(Item::SpringBall),
                screw_attack: equipped.contains(Item::ScrewAttack),
                other_suit: equipped.contains(Item::VariaSuit)
                    || equipped.contains(Item::GravitySuit),
            },
            spin_turned_far: animation::spin_turned_far(samus),
            elevator_moving: self.ride.status != ElevatorStatus::Inactive,
            parity: match ScanOrder::of(self.completed) {
                ScanOrder::LeftToRight => Parity::Even,
                ScanOrder::RightToLeft => Parity::Odd,
            },
            animation_ends: animation::ends_this_frame(
                samus,
                self.controller,
                self.requests,
                self.content,
            ),
            body,
        }
    }
}

#[cfg(test)]
mod tests;
