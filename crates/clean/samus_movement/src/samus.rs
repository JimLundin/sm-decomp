//! Samus as the player can tell her and what she remembers (§1, §1.2), and what a frame shows
//! beyond her quantities: its outputs, for the caller to act on.

use crate::collide::{Bounds, Moved, Stop};
use crate::jump_held::JumpHeld;
use crate::run::Dashing;
use crate::run::Run;
use crate::tick::{Frame, Tick};
use crate::vertical::Vertical;
use types::{Action, Aim, Body, Buttons, Cell, Facing, Fixed, Item, Pose, Position};
use types::{Set, Sprite, Touch, button::SHOOT};

/// `WallJump` (§1.2 a spin carried on): how long she has been in this spin, counted at each
/// frame's start up to the spin carried on, where it stays.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Spin {
    /// Not spinning.
    Off,
    /// In the spin's first frame.
    First,
    /// In its second frame.
    Second,
    /// A spin carried on: in it two frames earlier (this its third frame or later), or one that
    /// followed a wall jump. It can ready a wall jump.
    CarriedOn,
}

impl Spin {
    /// `WallJump`: the frame's start counts one frame more of a spin, up to the spin carried on,
    /// or none.
    pub(crate) const fn then(self, spinning: bool) -> Self {
        match (spinning, self) {
            (false, _) => Self::Off,
            (true, Self::Off) => Self::First,
            (true, Self::First) => Self::Second,
            (true, Self::Second | Self::CarriedOn) => Self::CarriedOn,
        }
    }
}

/// `SlopesCarry`: whether she is seated on a slope, until her next vertical move of more than 0
/// starts.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Seat {
    /// She is not on a slope.
    Unseated,
    /// Seated on a slope: her ground pull is 1 px.
    Seated,
}

/// `WallJump`'s output: the near-wall test's outcome this frame.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum WallTest {
    /// No test, no wall found, or a wall near her once the spin has turned far, without jump
    /// pressed.
    #[default]
    Neither,
    /// A wall near her while the spin hasn't turned far: the spin's animation skips ahead.
    Readied,
    /// She kicked off the wall.
    Kicked,
}

/// `AimingAndShooting`: what the standing animation does this frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StandingAnimation {
    /// It plays on.
    PlaysOn,
    /// It restarts.
    Restarts,
}

impl StandingAnimation {
    /// `AimingAndShooting`: standing, unaimed and not landing (upright `Still`, aim `Forward`,
    /// her pose at the frame's start), holding shoot restarts the standing animation.
    pub(crate) const fn of(p: Pose, pad: Buttons) -> Self {
        let unaimed = matches!(
            (p.body, p.action, p.aim),
            (Body::Upright, Action::Still, Aim::Forward)
        );
        if unaimed && pad.any(SHOOT) {
            Self::Restarts
        } else {
            Self::PlaysOn
        }
    }
}

/// `HowSheIsDrawn`: which sprite shows her, beyond her pose's parts: the spin (and a wall
/// jump's) drawn as the screw attack with screw attack, else as the space jump with space jump;
/// facing the screen, the other suit's sprite with varia or gravity; else plain.
#[must_use]
pub fn sprite(pose: Pose, items: &Set<Item>) -> Sprite {
    let spin = matches!(pose.action, Action::Spinning | Action::WallJumping);
    let suit = items[Item::VariaSuit] || items[Item::GravitySuit];
    if spin && items[Item::ScrewAttack] {
        Sprite::ScrewAttack
    } else if spin && items[Item::SpaceJump] {
        Sprite::SpaceJump
    } else if pose.facing == Facing::Screen && suit {
        Sprite::OtherSuit
    } else {
        Sprite::Plain
    }
}

/// What a frame shows beyond her quantities: its outputs, for the caller to act on.
///
/// The camera reads `skidding`, `placed` and `rising`; samus-appearance the outputs §1.4 gives for
/// her animation, `dashing_after_moves`, `dash_began`, `dash_lost`, `handed_over`,
/// `heading_down` and `vertical_speed_after_moves`, with her pose and vertical speed at the
/// frame's end, `standing_animation` and `wall_test`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each is one of the spec's yes-or-no outputs (§1, §1.4), read apart by other domains"
)]
pub struct Outcome {
    /// Spec: `TravelledDistances`. Fresh complete signed ordinary travels after collision.
    /// Save this set for the next firing opportunity; aim steps and placements are excluded.
    pub travelled: types::DirectionalTravels,
    /// `WallJump`: the near-wall test's outcome.
    pub wall_test: WallTest,
    /// `AimingAndShooting`: what the standing animation does.
    pub standing_animation: StandingAnimation,
    /// `HowSheIsDrawn`: which sprite shows her.
    pub sprite: Sprite,
    /// §1.2, `TurningRound`: she skids, moving against her facing, a skid at speed 0 included
    /// (the camera leads her backwards while she does).
    pub skidding: bool,
    /// `TouchedCells`: the cells her moves and looks reached, in the order they happen.
    pub reached: Vec<Touch>,
    /// `TouchedCells`: the cells she was inside at the frame's start.
    pub inside: Vec<Cell>,
    /// §1, `PoseFitAfterMoves`, `ShrinkingLowersHer`: the y a placement put her at this frame,
    /// if any: pose fit moved her, or a change of shape started, a placement of 0 px included.
    /// The camera's scroll measures her vertical move from it.
    pub placed: Option<Fixed>,
    /// §1, `RisingAndFalling`: she is rising at the frame's end: a rise in the air (a rise at 0
    /// and the frame whose pull spends it included), or an upward speed she keeps on the ground.
    pub rising: bool,
    /// §1.4, `Dashing`: whether she is dashing as her moves leave it, before the pose change.
    pub dashing_after_moves: Dashing,
    /// §1.4, `Dashing`: she wasn't dashing at the frame's start, and her moves were a run with
    /// run held, which starts it.
    pub dash_began: bool,
    /// §1.4, `Dashing`: she was dashing at the frame's start or from a run in her moves, and
    /// isn't at the frame's end. A run into a wall that starts her dashing reports both.
    pub dash_lost: bool,
    /// §1.4, `AnimationEnds`: her animation's end handed over her pose this frame (an end's pose
    /// is always the one she ends the frame in).
    pub handed_over: bool,
    /// §1.4, `RisingAndFalling`: she is heading down as her moves leave her, before what they
    /// met acts and before the pose change: falling (in the air, or on the ground at 0), or
    /// keeping a downward speed on the ground.
    pub heading_down: bool,
    /// §1.4, `RisingAndFalling`: her vertical speed, signed, downward positive, at the point
    /// `heading_down` is read: her moves' pull included, before what they met acts and before
    /// the pose change.
    pub vertical_speed_after_moves: Fixed,
}

/// Samus as the player can tell her, and what she remembers.
///
/// Caller-held State: every public field and nested value is serialized and deserialized by
/// serde's derives, in declaration order. Restore the complete value before stepping a frame;
/// the caller separately preserves `TravelledDistances` for the firing interval.
///
/// Her body's size is not hers to remember: it is the game's data for her pose (§1.3,
/// `Frame::body`), and a frame's moves use the body of the pose she starts it in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Samus {
    /// Where she is: her body's centre.
    pub position: Position,
    /// Her horizontal motion: speed, the way it points, carrying on, her dash and whether she is
    /// dashing.
    pub run: Run,
    /// Her vertical motion: speed, its phase, the ball's bounces.
    pub vertical: Vertical,
    /// What the player sees her doing (§2): body, action, facing, aim, gun and legs.
    pub pose: Pose,
    /// `WallJump`: how long she has been in a spin.
    pub spin: Spin,
    /// `SlopesCarry`: whether she is seated on a slope.
    pub seat: Seat,
    /// `Jumping`: how long jump has stayed held, and whether she landed or an animation ended
    /// the frame before.
    pub jump_held: JumpHeld,
}

impl Samus {
    /// `RidingTheElevator`: she faces the screen on a moving elevator, which carries her down;
    /// nothing else of hers changes that frame.
    pub(crate) fn rides_the_elevator(&self, f: &Frame) -> bool {
        self.pose.facing == Facing::Screen && f.elevator_moving
    }

    /// `BlocksStopFlush`, §1.3 body size: her box, the one her moves and looks test: her centre,
    /// and the body of her pose (during the frame's moves, the pose she started it in; after the
    /// pose change, the one she takes).
    pub(crate) fn bounds(&self, t: &Tick) -> Bounds {
        Bounds {
            at: self.position,
            r: t.body(self.pose),
        }
    }

    /// She takes what a move or a look did to her: a move puts her where it left her; a look
    /// doesn't move her, but what stopped it writes her subpixel as a stop would (a block FFFF or
    /// 0 on the look's axis, a solid enemy she touches her y subpixel 0). `SlopesCarry`: a
    /// downward one that a stepped block stopped seats her.
    pub(crate) fn take_result(&mut self, moved: &Moved) {
        self.position = moved.to;
        if moved.stop == Some(Stop::Seats) {
            self.seat = Seat::Seated;
        }
    }
}
