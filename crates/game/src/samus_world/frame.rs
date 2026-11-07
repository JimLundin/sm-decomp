//! What one frame brings the samus-motion model (`clean_model_samus_motion::Frame`), read from
//! the engine's State: the pad under the bindings, her items, her animation, the elevator and
//! the frame count.

use clean_model_samus_motion::Frame;
use clean_types::{Button, Equipment, Pad, Parity, Radius};
use data::pose::transition::{Control, Controls};

use crate::animation;
use crate::collision::ScanOrder;
use crate::collision::world::X_RADIUS;
use crate::pose::{Content, PoseRequests, controls};
use crate::state::Controller;
use crate::state::room::{ElevatorRide, ElevatorStatus};
use crate::state::samus::{Item, Samus};

/// The model's button each of the engine's controls works; item cancel moves nothing.
const BUTTONS: [(Control, Button); 9] = [
    (Control::Run, Button::Run),
    (Control::Jump, Button::Jump),
    (Control::Shoot, Button::Shoot),
    (Control::Left, Button::Left),
    (Control::Right, Button::Right),
    (Control::Up, Button::Up),
    (Control::Down, Button::Down),
    (Control::AimUp, Button::AimUp),
    (Control::AimDown, Button::AimDown),
];

/// The model's pad from the controls the buttons work.
fn pad(held: Controls) -> Pad {
    BUTTONS
        .iter()
        .filter(|&&(control, _)| held.contains(control))
        .fold(Pad::default(), |pad, &(_, button)| pad.with(button))
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

impl FrameSource<'_> {
    /// The model's frame.
    #[must_use]
    pub fn frame(&self) -> Frame {
        let samus = self.samus;
        let equipped = samus.equipment.equipped;
        Frame {
            pad: pad(controls(self.controller.held)),
            pressed: pad(controls(self.controller.pressed)),
            equipment: Equipment {
                hi_jump: equipped.contains(Item::HiJumpBoots),
                space_jump: equipped.contains(Item::SpaceJump),
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
            radius: Radius {
                x: X_RADIUS,
                y: samus.y_radius,
            },
        }
    }
}

#[cfg(test)]
mod tests;
