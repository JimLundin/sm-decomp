//! Samus as the player can tell her and what she remembers (§1, §1.2), and what a frame shows
//! beyond her quantities: its outputs, for the caller to act on.

use crate::collide::{Bounds, Moved, Probe, Stop};
use crate::getaway::{DeferredMotion, Getaway};
use crate::jump_held::JumpHeld;
use crate::knockback::Knockback;
use crate::run::Dashing;
use crate::run::Run;
use crate::tick::Frame;
use crate::vertical::Vertical;
use types::{Action, Aim, Body, Buttons, Cell, Facing, Fixed, Item, Pose, Position};
use types::{Set, Sprite, Touch, ZERO, button::SHOOT};

/// `WallJump`, `GetawayTravels`: the full poses her latest two choices recorded.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Choices {
    /// What her latest choice recorded.
    pub latest: Pose,
    /// What the choice before that recorded, or none at a fresh start.
    pub before: Option<Pose>,
}

impl Choices {
    /// §1.2 a spin carried on: a start has the pose she starts in (`pose`) chosen once, the
    /// choice before it neither a spin nor a wall jump.
    #[must_use]
    pub const fn starting_in(pose: Pose) -> Self {
        Self {
            latest: pose,
            before: None,
        }
    }

    /// Publish a resolved choice and remember it, even if it repeats the latest pose.
    pub(crate) const fn record(&mut self, resolved: Option<Pose>) -> Option<Pose> {
        if let Some(pose) = resolved {
            self.before = Some(self.latest);
            self.latest = pose;
        }
        resolved
    }

    /// `WallJump`: in `pose`, she is in a spin carried on: a spin whose choice before her latest
    /// one was a spin or a wall jump, facing either way. It can ready a wall jump.
    pub(crate) fn carried_on(self, pose: Pose) -> bool {
        pose.action == Action::Spinning && self.before.is_some_and(|older| older.action.spins())
    }
}

/// `SlopesCarry`: whether she is seated on a slope, which makes her ground pull 1 px.
///
/// A move of exactly 0, a vertical move a solid enemy stops before blocks, and every look
/// leave it as it is. Any other vertical move clears it, before a stepped block that stops it
/// seats her again.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Seat {
    /// She is not on a slope.
    Unseated,
    /// Seated on a slope: her ground pull is 1 px.
    Seated,
}

/// `WallJump`'s output: the near-wall test's outcome this frame.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
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
    let spin = pose.action.spins();
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
/// The camera reads `skidding`, `placed` and `rising`; the HUD's mini-map
/// `position_after_moves`; samus-appearance the outputs §1.4 gives for her animation,
/// `dashing_after_moves`, `dash_began`, `dash_lost`, `handed_over`, `heading_down` and
/// `vertical_speed_after_moves`, with her pose and vertical speed at the frame's end,
/// `standing_animation`, `wall_test`, `floor_response` and `touchdown_graphics`; sound transport
/// `touchdown_requests`; the world `inside` and `reached`; weapons `pending_aim` and `travelled`;
/// samus-appearance `push_started` too (§12); a caller keeping her prior poses
/// `recorded_choice`. Each is the frame's own, not State.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each is one of the spec's yes-or-no outputs (§1, §1.4), read apart by other domains"
)]
pub struct Outcome {
    /// The pose taken on this cut, if different, for appearance's pose change.
    pub pose_change: Option<Pose>,
    /// Which caller-owned deferred movement inputs the shove discards.
    pub deferred_motion: DeferredMotion,
    /// Spec: `AcceptedFloorResponse`. The floor response accepted this frame, with her vertical
    /// speed just before it, if any.
    pub floor_response: Option<types::FloorResponse>,
    /// Spec: `TouchdownRequests`. The touchdown's sound requests, in order; appearance's later
    /// requests go after them, repeated identities kept. Transport decides what plays.
    pub touchdown_requests: Vec<types::SoundRequest>,
    /// Spec: `TouchdownRequests`. Permission for appearance to weigh its own dust and splash
    /// rules, not a claim that either shows; cinematic silences the sounds alone.
    pub touchdown_graphics: bool,
    /// Spec: `TransitionFiringAim`. Fresh optional cannon direction from the actual pose change.
    /// On `None`, the caller keeps earlier pending aim; this is not retained movement State.
    pub pending_aim: Option<types::FiringDirection>,
    /// Spec: `TravelledDistances`. Fresh complete signed ordinary travels after collision.
    /// Save this set for the next firing opportunity; aim steps and placements are excluded.
    pub travelled: types::DirectionalTravels,
    /// `WallJump`: the near-wall test's outcome.
    pub wall_test: WallTest,
    /// `WallJump`: what this frame's choice of her pose recorded in her `Choices`, or `None`
    /// where the frame chose nothing and her choices stayed as they were. A frame that chooses
    /// the pose she already has anew records it (`Some`), unlike one that chooses nothing.
    pub recorded_choice: Option<Pose>,
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
    /// §1, `MovesAsShePosed`: where her moves (the horizontal move, the near-wall test, the
    /// ground pull or her vertical move, the elevator's descent) leave her, before the aim
    /// change's step and the pose change's push, lift and lowering. The mini-map reads it.
    pub position_after_moves: Position,
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
    /// §1.4, `AnimationEnds`: her animation's end handed over her pose this frame, rather than
    /// giving way. A handed-over pose is taken without pose fit, however tall its body.
    pub handed_over: bool,
    /// §1.4, `RisingAndFalling`: she is heading down as her moves leave her, before what they
    /// met acts and before the pose change: falling (in the air, or on the ground at 0), or
    /// keeping a downward speed on the ground.
    pub heading_down: bool,
    /// §1.4, `RisingAndFalling`: her vertical speed, signed, downward positive, at the point
    /// `heading_down` is read: her moves' pull included, before what they met acts and before
    /// the pose change.
    pub vertical_speed_after_moves: Fixed,
    /// §12, `PushStarts`: a push started this frame (samus-appearance pulses her colours).
    pub push_started: bool,
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
    /// `WallJump`: what she last chose of her pose, and before that.
    pub choices: Choices,
    /// `SlopesCarry`: whether she is seated on a slope.
    pub seat: Seat,
    /// `Jumping`: how long jump has stayed held, and whether she landed or an animation ended
    /// the frame before.
    pub jump_held: JumpHeld,
    /// §12: the knockback's push time left, its side and the push under way.
    pub knockback: Knockback,
    /// `GetawayArms`, `GetawayStarts`, `GetawayTravels`: the flyby's queued or active shove.
    pub getaway: Getaway,
    /// `GetawayTravels`, `AfterThePush`: the reduced speed magnitude saved by a sideways
    /// stop, before direction, room carry, the displacement limit or contact.
    pub getaway_release_speed: Option<Fixed>,
}

impl Samus {
    /// `RidingTheElevator`: she faces the screen on a moving elevator, which carries her down;
    /// nothing else of hers changes that frame.
    pub(crate) fn rides_the_elevator(&self, f: &Frame) -> bool {
        self.pose.facing == Facing::Screen && f.elevator_moving
    }

    /// §1.3 body size: her box, about her centre, in the body of the pose she is in.
    pub(crate) fn bounds(&self, f: &Frame) -> Bounds {
        Bounds {
            at: self.position,
            r: (f.body)(self.pose),
        }
    }

    /// `SlopesSlowTheWalk`: a horizontal move made with her vertical speed exactly 0 is a walk,
    /// which a sloped floor shortens; any other is a plain move.
    pub(crate) fn horizontal_probe(&self) -> Probe {
        if self.getaway.downward_speed().unwrap_or(self.vertical.speed) == ZERO {
            Probe::Walk
        } else {
            Probe::Move
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
