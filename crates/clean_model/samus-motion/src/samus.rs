//! Samus as the player can tell her and what she remembers (§1, §1.2), and what a frame shows
//! beyond her quantities: its outputs, for the caller to act on.

use crate::collide::{Bounds, Moved, Stop};
use crate::jump_held::JumpHeld;
use crate::run::Run;
use crate::tick::Tick;
use crate::vertical::Vertical;
use clean_types::{Action, Aim, Body, Buttons, Cell, Facing, Fixed, Item, Pose, Position};
use clean_types::{Set, Touch, button::SHOOT};

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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Seat {
    /// Not seated.
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

/// `HowSheIsDrawn`: which sprite shows her, beyond her pose's parts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sprite {
    /// Plain, in her first suit.
    Plain,
    /// The spin drawn as the screw attack.
    ScrewAttack,
    /// The spin drawn as the space jump.
    SpaceJump,
    /// Facing the screen in the other suit's sprite.
    OtherSuit,
}

impl Sprite {
    /// `HowSheIsDrawn`: the spin (and a wall jump's) drawn as the screw attack with screw
    /// attack, else as the space jump with space jump; facing the screen, the other suit's
    /// sprite with varia or gravity; else plain.
    pub(crate) fn of(pose: Pose, items: &Set<Item>) -> Self {
        let spin = matches!(pose.action, Action::Spinning | Action::WallJumping);
        let suit = items[Item::VariaSuit] || items[Item::GravitySuit];
        if spin && items[Item::ScrewAttack] {
            Self::ScrewAttack
        } else if spin && items[Item::SpaceJump] {
            Self::SpaceJump
        } else if pose.facing == Facing::Screen && suit {
            Self::OtherSuit
        } else {
            Self::Plain
        }
    }
}

/// What a frame shows beyond her quantities: its outputs, for the caller to act on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
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
}

/// Samus as the player can tell her, and what she remembers.
///
/// Her body's size is not hers to remember: it is the game's data for her pose (§1.3,
/// `Frame::body`), and a frame's moves use the body of the pose she starts it in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Samus {
    /// Her body's centre.
    pub position: Position,
    /// Her horizontal motion: speed, the way it points, carrying on, her dash.
    pub run: Run,
    /// Her vertical motion: speed, its phase, the ball's bounces.
    pub vertical: Vertical,
    /// Her pose.
    pub pose: Pose,
    /// How long she has been in a spin.
    pub spin: Spin,
    /// `SlopesCarry`: seated on a slope.
    pub seat: Seat,
    /// `Jumping`: how long jump has stayed held, and whether she landed or an animation ended
    /// the frame before.
    pub jump_held: JumpHeld,
}

impl Samus {
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
