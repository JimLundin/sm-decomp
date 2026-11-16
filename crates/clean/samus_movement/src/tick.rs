//! One frame's play: what the frame brings from the player and the world (§1.3), and the room,
//! the rules and the cells reached that the frame's moves and looks share (`TouchedCells`).

use crate::rules::Rules;
use crate::travel::TravelledDistances;
use types::{Buttons, ClipEnd, Item, Parity, Pose, Radius, Set, Touch, World};

/// What one frame brings from the player and the world (§1.3), and from what is out of scope
/// and feeds us (the header): the end her animation reaches, from its timing.
pub struct Frame<'a> {
    /// Spec: `TouchdownRequests`. Actual figure selected before movement, supplied by
    /// appearance independently of current items or this frame's newly selected sprite.
    pub sprite_before_movement: types::Sprite,
    /// Spec: `TouchdownRequests`. Current cinematic state supplied by the surrounding mode.
    pub cinematic_active: bool,
    /// The buttons held as the frame starts.
    pub pad: Buttons,
    /// The buttons just pressed this frame.
    pub pressed: Buttons,
    /// §1.3: the items switched on, progress's own output (never the ones merely hers). Her
    /// motion reads `HiJumpBoots`, `SpaceJump`, `MorphBall`, `SpringBall` and `SpeedBooster`, and
    /// her sprite `ScrewAttack`, `VariaSuit` and `GravitySuit` (`HowSheIsDrawn`); the others
    /// change nothing here.
    pub items: Set<Item>,
    /// `WallJump`: the spin's sprite has turned far enough to kick off a wall.
    pub spin_turned_far: bool,
    /// `RidingTheElevator`: the elevator is moving.
    pub elevator_moving: bool,
    /// `BlocksStopFlush`: the frame count's parity.
    pub parity: Parity,
    /// `AnimationEnds`: the end her pose's animation reaches this frame, if any (samus-appearance's
    /// `ClipEnds`): an end, or an end unless a jump.
    pub animation_ends: Option<ClipEnd>,
    /// §1.3 body size: each pose's half-width and half-height, the game's data. The frame's
    /// moves use the body of the pose she starts it in; the chosen pose's is read only where pose
    /// fit sees it grow or a change of shape lowers her. `RidingTheElevator`: while a moving
    /// elevator carries her facing the screen, the caller gives, as the body of the pose she
    /// starts the frame in, the body she had as the ride began, not the body of facing the
    /// screen.
    pub body: &'a dyn Fn(Pose) -> Radius,
    /// Spec: `TransitionFiringAim`. Imported cannon direction for ordinary jump and moonwalk
    /// poses. Missing content publishes nothing. Forced shinespark handovers are not this step.
    pub cannon_heading: &'a dyn Fn(Pose) -> Option<types::FiringDirection>,
}

/// One frame's moves and looks: the frame they are made in, the room and rules they meet, and
/// the cells they reach, in order (`TouchedCells`). §8's methods are in `collide`.
pub struct Tick<'a> {
    /// What the frame brings from the player and the world.
    pub frame: &'a Frame<'a>,
    /// The room her moves and looks meet.
    pub world: &'a dyn World,
    /// The tuning parameters her motion reads.
    pub rules: &'a Rules,
    /// The cells her moves and looks reach, in order.
    pub reached: Vec<Touch>,
    /// Spec: `PerformedCollisionTests`. Only actually evaluated leading-edge tests.
    pub collision_tests: types::CollisionTests,
    /// Purpose of the current movement/look, set by its semantic caller.
    pub purpose: types::CollisionPurpose,
    /// Spec: `AcceptedFloorResponse`. Move-local stops, or accepted contact after fitting.
    pub floor_response: Option<types::FloorResponse>,
    /// Spec: `TouchdownRequests`. A wall kick accepted by fitting, not merely selected.
    pub accepted_wall_kick: bool,
    /// Spec: `TravelledDistances`. Fresh ordinary travel, unaffected by later looks or placements.
    pub travelled: TravelledDistances,
    /// Spec: `TransitionFiringAim`. Fresh publication from the accepted ordinary pose change.
    pub pending_aim: Option<types::FiringDirection>,
}

impl Tick<'_> {
    /// §1.3 body size: a pose's half-width and half-height.
    pub fn body(&self, pose: Pose) -> Radius {
        (self.frame.body)(pose)
    }
}
