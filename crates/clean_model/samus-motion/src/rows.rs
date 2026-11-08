//! The pose chart's vocabulary (§7).
//!
//! What she is doing and how she moves there (a situation, a state of the chart's state
//! machine), and the rows of the chart's tables (its moves). `situation` and `chart::table` hold
//! the rows; `chart` reads them.

use crate::rules::Gait;
use clean_types::{Action, Aim, Body, Buttons, Facing, Gun, Heading, Legs, button};

pub use button::{BACK, FORWARD};

/// What she is doing, as the spec's tables name it ("She is", "She was").
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Doing {
    /// Standing still, or landing.
    Standing,
    /// Running.
    Running,
    /// Taking off: the first poses of a straight jump.
    TakingOff,
    /// In a straight jump, upright.
    StraightJump,
    /// Spinning.
    Spinning,
    /// Falling, upright.
    Falling,
    /// Wall jumping.
    WallJumping,
    /// Moonwalking.
    Moonwalking,
    /// Against the wall.
    AgainstWall,
    /// Crouched.
    Crouched,
    /// A ball, on the ground or in the air.
    Ball,
    /// The spring ball's hop.
    Hop,
    /// Turning, on the ground or in the air.
    Turning,
    /// Her body changing shape: crouching, standing up, morphing or unmorphing.
    Shaping,
}

/// `SpeedBuildsAndBrakes`, `DriftInTheAir`: how her speed builds as she moves by it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Drift {
    /// On the ground: following the pad she builds; carrying on she brakes.
    Ground,
    /// A straight jump or a fall: let go, she stops drifting.
    Air,
    /// The ball in the air: let go, it halts but still moves by its dash.
    BallAir,
    /// A spin jump, drifting only while she holds forward, skids or tops out.
    Spin,
    /// A wall jump, always drifting.
    WallJump,
}

/// `SpeedBuildsAndBrakes`: how she moves horizontally in her situation.
#[derive(Clone, Copy)]
pub enum Moves {
    /// Her speed builds by the table's row for what she is doing, as the drift says, and she
    /// moves by it and her dash.
    Build(Gait, Drift),
    /// Standing, against the wall, crouched: her dash only, then a halt.
    DashThenHalt,
    /// The ball at rest on the ground: braking or skidding it rolls on, keeping its dash; with a
    /// vertical speed, her dash only; else as standing.
    Rest,
    /// `Jumping`: taking off, her dash only, a dash to lose gone first, following the pad.
    TakeOff,
    /// While her body changes shape: her dash only, keeping both speeds.
    Dash,
}

/// A state of the pose chart: what she is doing, how she moves, and its pad chart's rows, the
/// last of them apart (`PadChart`: it covers nothing else fitting).
pub struct Situation {
    /// What she is doing.
    pub doing: Doing,
    /// How she moves.
    pub moves: Moves,
    /// Its pad chart rows but the last.
    pub rows: &'static [Row],
    /// Its last row.
    pub last: Row,
}

/// `MovesAsShePosed`: the poses a situation stands for, from her pose at the frame's start.
pub struct Class {
    /// Her body one of these.
    pub body: &'static [Body],
    /// Her action one of these.
    pub action: &'static [Action],
    /// Her aim one of these.
    pub aim: &'static [Aim],
    /// On the ground or not; `None` either.
    pub ground: Option<bool>,
    /// The situation.
    pub is: &'static Situation,
}

/// One thing a row needs. `Held` and `Pressed` are the buttons a row names (`PadChart`'s rank);
/// the rest are conditions on her situation.
#[derive(Clone, Copy)]
pub enum Test {
    /// One of these held.
    Held(Buttons),
    /// One of these just pressed.
    Pressed(Buttons),
    /// The test failing.
    Not(&'static Self),
    /// Her aim one of these.
    Aimed(&'static [Aim]),
    /// Her gun so.
    Gun(Gun),
    /// Facing this side (the screen counts as right).
    Faces(Heading),
    /// Her body one of these.
    Body(&'static [Body]),
    /// Her action one of these.
    Action(&'static [Action]),
    /// Doing one of these.
    Doing(&'static [Doing]),
    /// Off the ground once her moves are made.
    Airborne,
    /// With the spring ball.
    SpringBall,
    /// Her horizontal speed at the frame's start isn't 0.
    Moving,
    /// Her moves met this (`WhatHerMovesMet`).
    Met(Met),
}

/// `Aiming`: the aim a next pose takes when the tests pass.
pub struct Aiming {
    /// What it needs.
    pub when: &'static [Test],
    /// The aim.
    pub aim: Aim,
}

/// The way a next pose faces.
#[derive(Clone, Copy)]
pub enum Side {
    /// As she faced.
    Kept,
    /// Back.
    Back,
    /// This side of the pad, whichever way she faced.
    To(Facing),
}

/// The pose a row leads to; `None` keeps that part of her pose as it was.
#[derive(Clone, Copy)]
pub struct Next {
    /// The body.
    pub body: Option<Body>,
    /// The action.
    pub action: Option<Action>,
    /// The facing.
    pub facing: Side,
    /// The first aim whose tests pass; where none does, the aim is kept.
    pub aim: &'static [Aiming],
    /// The gun.
    pub gun: Option<Gun>,
    /// The legs.
    pub legs: Option<Legs>,
}

/// What a change does to her speeds, dash and position, in order.
#[derive(Clone, Copy)]
pub enum Effect {
    /// `Jumping`'s take-off, spin start or the spring ball's hop: she rises at the jump speed
    /// (taking off, the take-off's poses' own moves follow the pad).
    Rise,
    /// `Jumping`'s take-off from a crouch without aim: she is also lifted.
    Lift,
    /// `TurningRound`.
    TurnRound,
    /// She follows the pad again: a skid ends, along her new facing.
    EndSkid,
    /// As running: the dash joins her speed and she carries on, braking.
    AsRunning,
    /// Dash 0.
    LoseDash,
    /// Horizontal speed 0, dash 0, following the pad.
    Halt,
    /// `Moonfall`.
    Moonfall,
}

/// `PadChart`: a beats clause with a condition: the row it beats, by name, while every test
/// passes.
pub struct Beat {
    /// The row it beats.
    pub row: &'static str,
    /// While these pass.
    pub when: &'static [Test],
}

/// A move of the state machine: a row of one of §7's tables.
pub struct Row {
    /// Its Pad cell, which a beats clause names.
    pub name: &'static str,
    /// What it needs, all of it.
    pub needs: &'static [Test],
    /// The pose it leads to.
    pub next: Next,
    /// What it does.
    pub effects: &'static [Effect],
    /// `PadChart`: the rows of its situation it beats, by name.
    pub beats: &'static [&'static str],
    /// `PadChart`: the rows of its situation it beats while a condition holds.
    pub beats_if: &'static [Beat],
    /// `AimingAndShooting`: its kept running pose counts as chosen, bringing the step.
    pub steps: bool,
}

/// `PadChart`: an aim button, aim up or aim down.
pub const AIM: Buttons = button::AIM_UP.union(button::AIM_DOWN);
/// `PadChart`: up or down.
pub const UP_DOWN: Buttons = button::UP.union(button::DOWN);
/// `PadChart`: up, down or an aim button.
pub const UDA: Buttons = UP_DOWN.union(AIM);

/// A row's defaults: it needs nothing, keeps her pose, does nothing and beats nothing.
pub const ROW: Row = Row {
    name: "",
    needs: &[],
    next: KEPT,
    effects: &[],
    beats: &[],
    beats_if: &[],
    steps: false,
};

/// "kept": her pose stays as it is.
pub const KEPT: Next = Next {
    body: None,
    action: None,
    facing: Side::Kept,
    aim: &[],
    gun: None,
    legs: None,
};

/// Upright `Still`, aim `Forward`, the gun stowed and the legs together (`PadChart`: a next pose
/// has its gun stowed and its legs together unless its cell says otherwise).
pub const STILL: Next = Next {
    body: Some(Body::Upright),
    action: Some(Action::Still),
    facing: Side::Kept,
    aim: FORWARD_AIM,
    gun: Some(Gun::Stowed),
    legs: Some(Legs::Together),
};
/// `Running`, `Forward`.
pub const RUN: Next = Next {
    action: Some(Action::Running),
    ..STILL
};
/// `Jumping`, `Forward`.
pub const JUMP: Next = Next {
    action: Some(Action::Jumping),
    ..STILL
};
/// `Falling`, `Forward`.
pub const FALL: Next = Next {
    action: Some(Action::Falling),
    ..STILL
};
/// `Spinning`.
pub const SPIN: Next = Next {
    action: Some(Action::Spinning),
    ..STILL
};
/// Upright `TakingOff`, `Forward`.
pub const TAKE_OFF: Next = Next {
    action: Some(Action::TakingOff),
    ..STILL
};
/// `Moonwalking`, `Forward`.
pub const MOONWALK: Next = Next {
    action: Some(Action::Moonwalking),
    ..STILL
};
/// Crouched `Crouching`, `Forward`.
pub const CROUCH: Next = Next {
    body: Some(Body::Crouched),
    action: Some(Action::Crouching),
    ..STILL
};
/// Ball `Morphing`.
pub const MORPH: Next = Next {
    body: Some(Body::Ball),
    action: Some(Action::Morphing),
    ..STILL
};
/// `Turning` on the ground: facing back, with the aim she had (§2).
pub const TURN: Next = Next {
    action: Some(Action::Turning(clean_types::Turn::Ground)),
    facing: Side::Back,
    aim: &[],
    ..STILL
};

/// `Aiming`: `Forward`.
pub const FORWARD_AIM: &[Aiming] = &[Aiming {
    when: &[],
    aim: Aim::Forward,
}];
/// `Aiming`: `Up`.
pub const UP_AIM: &[Aiming] = &[Aiming {
    when: &[],
    aim: Aim::Up,
}];
/// `Aiming`: `UpForward` with aim up, else `DownForward`.
pub const AIM_UP_ELSE_DOWN: &[Aiming] = &[
    Aiming {
        when: &[Test::Held(button::AIM_UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[],
        aim: Aim::DownForward,
    },
];
/// `Aiming`: `Down` becomes `Forward`; else kept.
pub const DOWN_TO_FORWARD: &[Aiming] = &[Aiming {
    when: &[Test::Aimed(&[Aim::Down])],
    aim: Aim::Forward,
}];

/// `WhatHerMovesMet`: what this frame's moves met.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Met {
    /// A wall stop (`WallsAndCeilings`).
    Wall,
    /// A wall near her as she presses jump (`WallJump`'s kick).
    WallKick,
    /// A floor (`Landing`).
    Floor,
    /// No floor: she walks off a ledge (`FollowingTheGround`).
    NoFloor,
    /// A ceiling (`WallsAndCeilings`).
    Ceiling,
}
