//! The samus-motion model, built from its behaviour spec alone: a character controller for
//! Samus, stepped one 60 Hz frame at a time, her motion in fixed-point subpixels.
//!
//! Every rule names the spec entry it comes from. A frame (`PoseFitAfterMoves`): she moves as
//! the pose at its start says (`MovesAsShePosed`), horizontally, then by the ground pull or her
//! vertical move, then by the aim change's step; what those moves met is known; the pose is
//! chosen (`PoseFollows`); pose fit or the shrink places her; then the change's effects act.

pub mod catalogue;
pub mod chart;
mod collide;
mod fit;
mod motion;
pub mod rows;
pub mod rules;
pub mod situation;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use chart::{Choice, Moment};
pub use rows::{Effect, Met};
pub use rules::{Rules, SpeedCap};
pub use situation::classify;

use Carry::{CarriesOn, FollowsPad};
use Effect::{AsRunning, EndSkid, Halt, LoseDash, Rise, TakeOff, TurnRound};
use clean_types::Action::{
    Crouching, Falling, Jumping, Morphing, Spinning, Still, TakingOff, WallJumping,
};
use clean_types::Body::{Crouched, Upright};
use clean_types::button::{JUMP, SHOOT};
use clean_types::{
    Aim, Buttons, Cell, Equipment, Facing, Fixed, Gun, Heading, Legs, Parity, Pose, Position,
    Radius, Touch, World,
};
use collide::{Bounds, Mover};

/// Her vertical motion's phase; her vertical speed itself is `Samus::vy`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Vertical {
    /// On the ground, keeping the vertical speed she brought with her (`FollowingTheGround`).
    Ground,
    /// Rising (`RisingAndFalling`); at the start of a frame after the pull took her upward
    /// speed past zero, the rise ends.
    Rising,
    /// Falling.
    Falling,
    /// `Moonfall`: falling with no falling limit and no jump, until she lands.
    Moonfall,
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

/// §1.2 a spin carried on: how long she has been in this spin.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Spin {
    /// Not spinning.
    Off,
    /// Its first frame.
    First,
    /// Its second frame.
    Second,
    /// In it two frames earlier, or it followed a wall jump.
    CarriedOn,
}

/// §1.2 carrying on: how her horizontal motion goes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Carry {
    /// It follows the pad.
    FollowsPad,
    /// It carries on by itself: it brakes on the ground and keeps building in the air; against
    /// her facing, a skid.
    CarriesOn,
}

/// §1.2 her dash.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DashIs {
    /// Hers to keep.
    Hers,
    /// Left on her by a landing: gone the next frame she moves.
    ToLose,
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

/// `PoseFitAfterMoves`: how the pose the frame ends in came about.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Change {
    /// The chosen pose.
    Chosen,
    /// The change refused: she keeps the pose she had.
    Refused,
    /// Crouched `Still` taken instead.
    Crouched,
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
    /// `TouchedCells`: the cells her moves and looks reached, in the order they happen.
    pub reached: Vec<Touch>,
    /// `TouchedCells`: the cells she was inside at the frame's start.
    pub inside: Vec<Cell>,
}

/// Samus as the player can tell her, and what she remembers.
#[derive(Clone, Copy)]
pub struct Samus {
    /// Her body's centre.
    pub position: Position,
    /// Her horizontal speed's size.
    pub speed: Fixed,
    /// The way her horizontal speed points, left or right.
    pub way: Heading,
    /// §1.2 carrying on.
    pub carry: Carry,
    /// Her dash, added the way she moves.
    pub dash: Fixed,
    /// §1.2 her dash.
    pub dash_is: DashIs,
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

/// `PoseFitAfterMoves`: crouched `Still`, aim `Forward`, facing as before the change.
const fn crouched(p: Pose) -> Pose {
    Pose {
        body: Crouched,
        action: Still,
        aim: Aim::Forward,
        gun: Gun::Stowed,
        legs: Legs::Together,
        ..p
    }
}

/// `HowSheIsDrawn`: the spin (and a wall jump's) drawn as the screw attack with screw attack,
/// else as the space jump with space jump; facing the screen, the other suit's sprite with it.
const fn drawn(pose: Pose, e: Equipment) -> Drawn {
    let spin = matches!(pose.action, Spinning | WallJumping);
    if spin && e.screw_attack {
        Drawn::ScrewAttack
    } else if spin && e.space_jump {
        Drawn::SpaceJump
    } else if matches!(pose.facing, Facing::Screen) && e.other_suit {
        Drawn::OtherSuit
    } else {
        Drawn::Plain
    }
}

impl Samus {
    /// One frame (`PoseFitAfterMoves`): her moves as the pose at its start says
    /// (`MovesAsShePosed`) and what they met; the pose chosen (`PoseFollows`); pose fit or the
    /// shrink; then the change's effects.
    pub fn step(&mut self, frame: &Frame, world: &dyn World, rules: &Rules) -> Outcome {
        let mut m = Mover {
            world,
            rules,
            parity: frame.parity,
            reached: Vec::new(),
        };
        // TouchedCells: the cells she is inside, before her moves.
        let inside = m.inside(self.bounds());
        let start = *self;
        let situation = classify(start.pose, start.vertical == Vertical::Ground);
        self.begin(situation.doing, frame.pad);
        let meeting = self.moves(situation, frame, &mut m);
        // PoseFitAfterMoves: the speeds her moves left her, for a change that doesn't come.
        let moved = *self;
        self.meet(meeting.met, situation.doing, frame, rules);
        let mut chosen = Moment {
            situation,
            pose: start.pose,
            moving: !start.speed.is_zero(),
            airborne: self.vertical != Vertical::Ground,
            pad: frame.pad,
            pressed: frame.pressed,
            equipment: frame.equipment,
            met: meeting.met,
        }
        .choose(frame.animation_ends);
        if chosen.steps {
            self.aim_step(&mut chosen, &mut m);
        }
        let change = self.fit(chosen.pose, frame, &mut m);
        self.take(change, chosen, moved, frame, &mut m);
        // AimingAndShooting: standing, unaimed and not landing, holding shoot restarts the
        // standing animation.
        let p = start.pose;
        let restarts = matches!((p.body, p.action, p.aim), (Upright, Still, Aim::Forward));
        let standing = if restarts && frame.pad.any(SHOOT) {
            Standing::Restarts
        } else {
            Standing::Plays
        };
        Outcome {
            wall_test: meeting.wall_test,
            standing,
            drawn: drawn(self.pose, frame.equipment),
            reached: m.reached,
            inside,
        }
    }

    /// §8: her box.
    const fn bounds(&self) -> Bounds {
        Bounds {
            at: self.position,
            r: self.radius,
        }
    }

    /// `RisingAndFalling`, `Jumping`, `WallJump`: she starts rising at this speed.
    const fn rise(&mut self, v: Fixed) {
        self.vertical = Vertical::Rising;
        self.vy = v.negated();
    }

    /// `RisingAndFalling`, `FollowingTheGround`, `WallsAndCeilings`: she falls from rest.
    const fn fall_from_rest(&mut self) {
        self.vertical = Vertical::Falling;
        self.vy = Fixed(0);
    }

    /// A halt: horizontal speed 0, dash 0, following the pad.
    const fn halt(&mut self) {
        self.speed = Fixed(0);
        self.dash = Fixed(0);
        self.carry = FollowsPad;
    }

    /// `TurningRound`, `PadChart`'s "as running": the dash joins her speed (dash 0) and she
    /// carries on.
    const fn join_dash(&mut self) {
        self.speed = self.speed.plus(self.dash);
        self.dash = Fixed(0);
        self.carry = CarriesOn;
    }

    /// `PoseFitAfterMoves`: she takes the pose the change gives. A change refused, or crouched
    /// `Still` taken instead, leaves her speeds and dash as her moves left them (`moved`), with
    /// her position and seat as her looks left them. Only the chosen pose brings its effects, in
    /// order: after pose fit's push, the new pose's speeds, then the crouch take-off's lift.
    fn take(&mut self, change: Change, chosen: Choice, moved: Self, frame: &Frame, m: &mut Mover) {
        let (start, r) = (self.pose, m.rules);
        let pose = match change {
            Change::Chosen => chosen.pose,
            Change::Refused => start,
            Change::Crouched => crouched(start),
        };
        if change != Change::Chosen {
            *self = Self {
                position: self.position,
                seat: self.seat,
                ..moved
            };
        }
        self.pose = pose;
        self.radius = (frame.body)(pose);
        if change != Change::Chosen {
            return;
        }
        for effect in chosen.effects {
            self.apply(*effect, start, frame, r);
        }
        // The change of action's own effects: ShrinkingLowersHer, the frame she starts to crouch
        // or to morph she is lowered; Jumping, the frame she changes into a take-off from a
        // crouch without aim she is lifted, and starting a straight jump or a fall she carries on
        // along her facing if she has a dash, else she follows the pad; WallJump (§1.2), a spin
        // that followed a wall jump is carried on.
        if pose.action == start.action {
            return;
        }
        if pose.action == Crouching {
            self.lower(r.crouch_lower, m);
        }
        if pose.action == Morphing {
            self.lower(r.morph_lower, m);
        }
        if pose.action == TakingOff && start.body == Crouched && start.aim == Aim::Forward {
            self.position.y = self.position.y.minus(r.crouch_lift);
        }
        if pose.action == Falling || pose.action == Jumping && pose.body == Upright {
            self.carry = if self.dash.is_zero() {
                FollowsPad
            } else {
                self.way = pose.facing.side();
                CarriesOn
            };
        }
        if start.action == WallJumping && pose.action == Spinning {
            self.spin = Spin::CarriedOn;
        }
    }

    /// `PadChart`, `AnimationEnds`: one effect of the change on her speeds, dash and position.
    fn apply(&mut self, effect: Effect, start: Pose, frame: &Frame, r: &Rules) {
        let jump = r.jump.of(frame.equipment.hi_jump);
        match effect {
            // Jumping: changing into a take-off she starts rising and follows the pad.
            TakeOff => {
                self.rise(jump);
                self.carry = FollowsPad;
            }
            Rise => self.rise(jump),
            // TurningRound: her dash joins her speed and she travels the old way, against her
            // new facing: a skid. A spin reversed while skidding turns her travel round.
            TurnRound => {
                self.way = start.facing.side();
                self.join_dash();
            }
            EndSkid if self.carry == CarriesOn => {
                self.carry = FollowsPad;
                self.way = self.pose.facing.side();
            }
            // PadChart: as running, with a horizontal speed of 0 she only follows the pad again.
            AsRunning if self.speed.is_zero() => self.carry = FollowsPad,
            AsRunning => self.join_dash(),
            LoseDash => self.dash = Fixed(0),
            Halt => self.halt(),
            // Moonfall: a spin she enters when a turn ends, with jump not just pressed, starts
            // with no rise: her vertical speed is 0 and she drops under gravity.
            Effect::Moonfall if frame.pressed.any(JUMP) => self.rise(jump),
            Effect::Moonfall => {
                self.fall_from_rest();
                self.vertical = Vertical::Moonfall;
            }
            EndSkid => {}
        }
    }
}
