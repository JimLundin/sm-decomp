//! One frame's play: what the frame brings from the player and the world (§1.3), and the room,
//! the rules and the cells reached that the frame's moves and looks share (`TouchedCells`).

use crate::rules::Rules;
use clean_types::{Buttons, Equipment, Parity, Pose, Radius, Touch, World};

/// What one frame brings from the player and the world (§1.3), and from what is out of scope
/// and feeds us (the header): an animation's end from its timing.
pub struct Frame<'a> {
    /// The buttons held as the frame starts.
    pub pad: Buttons,
    /// The buttons just pressed this frame.
    pub pressed: Buttons,
    /// Her equipment.
    pub equipment: Equipment,
    /// `WallJump`: the spin's sprite has turned far enough to kick off a wall.
    pub spin_turned_far: bool,
    /// `RidingTheElevator`: the elevator is moving.
    pub elevator_moving: bool,
    /// `BlocksStopFlush`: the frame count's parity.
    pub parity: Parity,
    /// `AnimationEnds`: her pose's animation ends this frame.
    pub animation_ends: bool,
    /// §1.3 body size: each pose's half-width and half-height, the game's data.
    pub body: &'a dyn Fn(Pose) -> Radius,
}

/// One frame's moves and looks: the frame they are made in, the room and rules they meet, and
/// the cells they reach, in order (`TouchedCells`). §8's methods are in `collide`.
pub struct Tick<'a> {
    /// The frame.
    pub frame: &'a Frame<'a>,
    /// The room.
    pub world: &'a dyn World,
    /// The rules.
    pub rules: &'a Rules,
    /// The cells her moves and looks reach, in order.
    pub reached: Vec<Touch>,
}

impl Tick<'_> {
    /// `PoseFitAfterMoves`: a pose's half-width and half-height (§1.3 body size).
    pub fn body(&self, pose: Pose) -> Radius {
        (self.frame.body)(pose)
    }
}
