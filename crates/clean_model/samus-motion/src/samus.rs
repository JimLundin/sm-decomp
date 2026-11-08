//! Samus as the player can tell her and what she remembers (§1, §1.2), and what a frame shows
//! beyond her quantities: its outputs, for the caller to act on.

use crate::collide::{Bounds, Moved, Stop};
use crate::motion::Run;
use clean_types::{Cell, Fixed, Parts, Pose, Position, Radius, Touch, ZERO};

/// Her vertical motion's phase; her vertical speed itself is `Samus::vy`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Vertical {
    /// On the ground, keeping the vertical speed she brought with her (`FollowingTheGround`).
    Ground,
    /// On the ground, falling at a vertical speed of 0 (§1.2): only a push beyond this spec
    /// hands her back so. As the ball, or while her body changes shape, it is a vertical motion
    /// she moves by (`FollowingTheGround`).
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

/// `Landing`: how many times the ball has bounced since it last rested.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Bounces {
    /// None.
    None,
    /// One.
    Once,
    /// Two.
    Twice,
}

/// `WallJump` (§1.2 a spin carried on): how many frames she has been in this spin, counted up to
/// `Spin::CARRIED_ON`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Spin(pub u8);

impl Spin {
    /// Not spinning.
    pub const OFF: Self = Self(0);
    /// A spin carried on: in it two frames earlier (this its third frame or later), or one that
    /// followed a wall jump.
    pub const CARRIED_ON: Self = Self(3);
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
    /// §1, `PoseFitAfterMoves`, `ShrinkingLowersHer`: where the camera follows her from, a y.
    pub camera_from: Fixed,
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

    /// `RisingAndFalling`, `FollowingTheGround`, `WallsAndCeilings`: she falls from rest.
    pub(crate) const fn fall_from_rest(&mut self) {
        self.vertical = Vertical::Falling;
        self.vy = ZERO;
    }

    /// `FollowingTheGround` (§1.2): a vertical motion, on the ground: a vertical speed other
    /// than 0, or falling at 0.
    pub(crate) fn vertical_motion(&self) -> bool {
        self.vy != ZERO || self.vertical == Vertical::GroundFalling
    }

    /// `SlopesCarry`: a downward move or look that a stepped block stops seats her.
    pub(crate) fn seat_on(&mut self, stop: Option<Stop>) {
        if stop == Some(Stop::Seats) {
            self.seat = Seat::Seated;
        }
    }

    /// `PoseFitAfterMoves`, `WallJump`: a look writes to her as a stop would, without moving
    /// her: the subpixels it left.
    pub(crate) fn wrote(&mut self, seen: &Moved) {
        let p = &mut self.position;
        (p.x, p.y) = (p.x.with_sub(seen.to.x.sub()), p.y.with_sub(seen.to.y.sub()));
    }
}
