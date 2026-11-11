//! What one frame brings the samus-motion model (`clean_model_samus_motion::Frame`).
//!
//! It is read from the engine's State: the pad under the bindings, her items, her animation,
//! the elevator and the frame count, and each pose's body size from the pose table.

use clean_model_samus_motion::Frame;
use clean_types::{Buttons, Parity, Pose, Radius, button};
use core::num::Wrapping;
use data::assets::samus::import::map;
use data::pose::transition::{Control, Controls};

use super::state::{X_RADIUS, dress};
use crate::animation;
use crate::collision::ScanOrder;
use crate::pose::{Content, controls};
use crate::progress_world;
use crate::state::Controller;
use crate::state::room::{ElevatorRide, ElevatorStatus};
use crate::state::samus::Samus;

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
    /// The delay lists.
    pub content: &'a Content,
}

/// §1.3 body size: a pose's half-height.
///
/// The pose she is in has her radius word, which the current state handler set from the pose
/// table as the frame started, or which the elevator's ride kept from the pose before, since that
/// handler doesn't run while it carries her. Any other pose has the pose table's for the
/// original's pose that shows it in her dress; a pose the table doesn't hold has none, as the
/// engine's radius has none.
#[must_use]
pub fn body(content: &Content, samus: &Samus, pose: Pose) -> Radius {
    let y = if map::key_of(samus.pose) == Some(pose) {
        samus.y_radius
    } else {
        map::id_of(pose, dress(samus))
            .and_then(|id| content.poses.get(id))
            .map_or(0, |definition| u16::from(definition.y_radius))
    };
    Radius {
        x: X_RADIUS,
        y: Wrapping(y),
    }
}

impl FrameSource<'_> {
    /// The model's frame, with `body` its body sizes.
    #[must_use]
    pub fn frame<'b>(&self, body: &'b dyn Fn(Pose) -> Radius) -> Frame<'b> {
        let samus = self.samus;
        Frame {
            pad: pad(controls(self.controller.held)),
            pressed: pad(controls(self.controller.pressed)),
            items: progress_world::switched_on(&samus.equipment),
            spin_turned_far: animation::spin_turned_far(samus),
            elevator_moving: self.ride.status != ElevatorStatus::Inactive,
            parity: match ScanOrder::of(self.completed) {
                ScanOrder::LeftToRight => Parity::Even,
                ScanOrder::RightToLeft => Parity::Odd,
            },
            animation_ends: animation::ends_this_frame(samus, self.controller, self.content),
            body,
        }
    }
}

#[cfg(test)]
mod tests;
