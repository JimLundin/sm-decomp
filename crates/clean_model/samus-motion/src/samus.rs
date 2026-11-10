//! Samus as the player can tell her and what she remembers (§1, §1.2), and what a frame shows
//! beyond her quantities: its outputs, for the caller to act on.

use crate::collide::{Bounds, Moved, Stop};
use crate::jump_held::JumpHeld;
use crate::motion::Run;
use crate::rules::Bounces;
use clean_types::{Cell, Fixed, Parts, Pose, Position, Radius, Touch, ZERO};

/// Her vertical motion's phase; her vertical speed itself is `Samus::vy`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Vertical {
    /// On the ground, keeping the vertical speed she brought with her, downward or upward
    /// (`FollowingTheGround`); an upward one is a rise she keeps there (`RisingAndFalling`).
    Ground,
    /// On the ground, falling at a vertical speed of 0 (§1.2): only a push beyond this spec
    /// hands her back so. As the ball, or while her body changes shape, it is a vertical motion
    /// she moves by; in any other pose on the ground she keeps it and moves as with none, and
    /// only leaving the ground ends it (`FollowingTheGround`).
    GroundFalling,
    /// Rising (`RisingAndFalling`); at the start of a frame after the pull took her upward
    /// speed past zero, the rise ends.
    Rising,
    /// Falling.
    Falling,
    /// `Moonfall`: falling with no falling limit and no jump, until she lands.
    Moonfall,
}

impl Vertical {
    /// `FollowingTheGround`: whether she is on the ground, falling at 0 there included.
    #[must_use]
    pub const fn on_ground(self) -> bool {
        matches!(self, Self::Ground | Self::GroundFalling)
    }
}

/// `WallJump` (§1.2 a spin carried on): how long she has been in this spin, counted at each
/// frame's start up to the spin carried on, where it stays.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Seat {
    /// Not seated.
    Unseated,
    /// Seated on a slope: her ground pull is 1 px.
    Seated,
}

/// `WallJump`'s output: the near-wall test's outcome this frame.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
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

/// `AimingAndShooting`: the standing animation.
pub enum Standing {
    /// It plays on.
    Plays,
    /// It restarts.
    Restarts,
}

/// `HowSheIsDrawn`: what her sprite shows beyond her pose's parts.
pub enum Drawn {
    /// Plain, in her first suit.
    Plain,
    /// The spin drawn as the screw attack.
    ScrewAttack,
    /// The spin drawn as the space jump.
    SpaceJump,
    /// Facing the screen in the other suit's sprite.
    OtherSuit,
}

/// What a frame shows beyond her quantities: its outputs, for the caller to act on.
pub struct Outcome {
    /// `WallJump`: the near-wall test's outcome.
    pub wall_test: WallTest,
    /// `AimingAndShooting`: the standing animation.
    pub standing: Standing,
    /// `HowSheIsDrawn`: what her sprite shows beyond her pose.
    pub drawn: Drawn,
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
}

/// Samus as the player can tell her, and what she remembers.
#[derive(Clone, Copy)]
pub struct Samus {
    /// Her body's centre.
    pub position: Position,
    /// Her horizontal motion: speed, the way it points, carrying on, her dash.
    pub run: Run,
    /// Her vertical speed, signed, downward positive.
    pub vy: Fixed,
    /// Her vertical motion's phase.
    pub vertical: Vertical,
    /// The ball's bounces.
    pub bounces: Bounces,
    /// Her pose.
    pub pose: Pose,
    /// Her body's half-width and half-height.
    pub radius: Radius,
    /// How long she has been in a spin.
    pub spin: Spin,
    /// `SlopesCarry`: seated on a slope.
    pub seat: Seat,
    /// `Jumping`: how long jump has stayed held, and whether she landed or an animation ended
    /// the frame before.
    pub jump_held: JumpHeld,
}

impl Samus {
    /// `BlocksStopFlush`: her box, the one her moves test.
    pub(crate) const fn bounds(&self) -> Bounds {
        Bounds {
            at: self.position,
            r: self.radius,
        }
    }

    /// `RisingAndFalling`, `Jumping`, `WallJump`: she starts rising at this speed.
    pub(crate) fn rise(&mut self, v: Fixed) {
        self.vertical = Vertical::Rising;
        self.vy = -v;
    }

    /// §1, `RisingAndFalling`: whether she is rising (`Outcome::rising`).
    pub(crate) fn rising(&self) -> bool {
        match self.vertical {
            Vertical::Rising => true,
            Vertical::Ground => self.vy.negative(),
            Vertical::GroundFalling | Vertical::Falling | Vertical::Moonfall => false,
        }
    }

    /// `RisingAndFalling`, `FollowingTheGround`, `WallsAndCeilings`: she falls from rest.
    pub(crate) const fn fall_from_rest(&mut self) {
        self.vertical = Vertical::Falling;
        self.vy = ZERO;
    }

    /// `Landing`: her vertical motion stops on the floor: on the ground, vertical speed 0.
    pub(crate) const fn settle(&mut self) {
        self.vertical = Vertical::Ground;
        self.vy = ZERO;
    }

    /// `FollowingTheGround` (§1.2): a vertical motion, on the ground: a vertical speed other
    /// than 0, or falling at 0.
    pub(crate) fn vertical_motion(&self) -> bool {
        self.vy != ZERO || self.vertical == Vertical::GroundFalling
    }

    /// `SlopesCarry`: a downward move or look that a stepped block stops seats her, and either
    /// leaves her where it got to (a look where it wrote to her).
    pub(crate) fn reach(&mut self, moved: &Moved) {
        self.position = moved.to;
        if moved.stop == Some(Stop::Seats) {
            self.seat = Seat::Seated;
        }
    }
}
