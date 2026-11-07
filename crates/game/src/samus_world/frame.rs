//! What one frame brings the samus-motion model (`clean_model_samus_motion::Frame`), read from
//! the engine's State: the pad under the bindings, her items, her animation, the elevator and
//! the frame count.

use clean_model_samus_motion::Frame;
use clean_types::{Equipment, Fixed, Pad, Parity, Radius, Signed};
use data::pose::transition::{Control, Controls};

use super::state::X_RADIUS;
use crate::animation;
use crate::collision::ScanOrder;
use crate::pose::{Content, PoseRequests, controls};
use crate::state::Controller;
use crate::state::room::{ElevatorRide, ElevatorStatus};
use crate::state::samus::{Item, Samus};

/// The model's pad from the controls the buttons work; item cancel moves nothing.
const fn pad(held: Controls) -> Pad {
    Pad {
        run: held.contains(Control::Run),
        jump: held.contains(Control::Jump),
        shoot: held.contains(Control::Shoot),
        left: held.contains(Control::Left),
        right: held.contains(Control::Right),
        up: held.contains(Control::Up),
        down: held.contains(Control::Down),
        aim_up: held.contains(Control::AimUp),
        aim_down: held.contains(Control::AimDown),
    }
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
            radius: Radius {
                x: X_RADIUS,
                y: samus.y_radius,
            },
            // The engine's pose fit runs after the model's step (gaps, item 5).
            push: Signed {
                negative: false,
                size: Fixed(0),
            },
        }
    }
}

#[cfg(test)]
mod tests;
