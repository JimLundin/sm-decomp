//! The pose chart's vocabulary (§7).
//!
//! What she is doing and how she moves there (a situation, a state of the chart's state
//! machine), and the rows of the chart's tables (its moves). `situation` and `chart::table` hold
//! the rows; `chart` reads them.

use crate::rules::{Gait, Lowers};
use clean_types::button::{self, LEFT, PAD, RIGHT};
use clean_types::{Action, Aim, Body, Buttons, Facing, Gun, Legs, Pose, Side};

/// `PadChart`: forward held, the side she faces (§2): the first set above the pad's own buttons,
/// so never on the pad. Only `seen` makes it.
pub const FORWARD: Buttons = Buttons(PAD.0 + 1);
/// `PadChart`: back held, the other side, the set above forward; likewise made by `seen` alone.
pub const BACK: Buttons = Buttons(FORWARD.0 << 1);

/// `PadChart`: the pad as the chart sees it, facing `facing`.
///
/// Forward and back are by her facing at the frame's start, facing the screen counting as right:
/// the pad's own buttons, with `FORWARD` where she holds the side she faces and `BACK` where she
/// holds the other. Any other bits a caller set are dropped, so forward and back come from this
/// view alone.
#[must_use]
pub const fn seen(pad: Buttons, facing: Facing) -> Buttons {
    let (ahead, behind) = if matches!(facing, Facing::Left) {
        (LEFT, RIGHT)
    } else {
        (RIGHT, LEFT)
    };
    let mut seen = pad.0 & PAD.0;
    if pad.any(ahead) {
        seen |= FORWARD.0;
    }
    if pad.any(behind) {
        seen |= BACK.0;
    }
    Buttons(seen)
}

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
    Faces(Side),
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
pub enum NextFacing {
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
    pub facing: NextFacing,
    /// The first aim whose tests pass; where none does, the aim is kept.
    pub aim: &'static [Aiming],
    /// The gun.
    pub gun: Option<Gun>,
    /// The legs.
    pub legs: Option<Legs>,
    /// `ShrinkingLowersHer`: the frame this pose starts, she looks down its own distance (the
    /// rules' crouch or morph distance) and is lowered by the room it finds; standing up and
    /// unmorphing lower her 0 px.
    pub lower: Option<Lowers>,
}

impl Next {
    /// The pose it leads to from `p`, aimed `aim` where its aim rule (`Aiming`) gave one, else
    /// with her aim kept.
    #[must_use]
    pub fn pose_from(self, p: Pose, aim: Option<Aim>) -> Pose {
        let facing = match self.facing {
            NextFacing::Kept => p.facing,
            NextFacing::Back => p.facing.back(),
            NextFacing::To(side) => side,
        };
        Pose {
            body: self.body.unwrap_or(p.body),
            action: self.action.unwrap_or(p.action),
            facing,
            aim: aim.unwrap_or(p.aim),
            gun: self.gun.unwrap_or(p.gun),
            legs: self.legs.unwrap_or(p.legs),
        }
    }
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
    /// `Jumping`: the change into a straight jump or a fall: with a dash she carries on along
    /// her facing, else she follows the pad.
    Launch,
    /// `WallJump` (§1.2): the change from a wall jump into a spin: the spin is carried on.
    SpinOn,
    /// `AnimationEnds`: the unmorph's end hands over the crouch, a pose on the ground, in the
    /// air too: she is on the ground from then on, her vertical speed kept
    /// (`FollowingTheGround`).
    OnTheGround,
}

/// A change of action's own effect, whatever brought the change.
///
/// The pad chart, what her moves met or an animation's end: a pose whose body and action are
/// one of `into`, taken from a pose whose action is as `from` says (never the actions it
/// changes into), brings `effect` after the chosen row's own.
pub struct Entering {
    /// The bodies and actions it changes into.
    pub into: &'static [(Body, Action)],
    /// The actions it changes from.
    pub from: Was,
    /// What it brings.
    pub effect: Effect,
}

/// The actions a change of action's own effect is taken from (`Entering::from`).
#[derive(Clone, Copy)]
pub enum Was {
    /// One of these.
    OneOf(&'static [Action]),
    /// Any action but these: whatever the set of actions holds, but these.
    AnyBut(&'static [Action]),
}

impl Entering {
    /// `Jumping`, `WallJump`: whether the change from action `was` into pose `now` brings it.
    #[must_use]
    pub fn on(&self, was: Action, now: Pose) -> bool {
        let from = match self.from {
            Was::OneOf(actions) => actions.contains(&was),
            Was::AnyBut(actions) => !actions.contains(&was),
        };
        from && self.into.contains(&(now.body, now.action))
    }
}

/// `PadChart`: a beats clause with a condition it doesn't apply under: the row it beats, unless
/// one of these sets of tests all pass, where that row beats it instead.
pub struct Beat {
    /// The row it beats.
    pub row: RowId,
    /// Where any of these holds (every test of it passing), the other row beats it.
    pub unless: &'static [&'static [Test]],
}

/// A move of the state machine: a row of one of §7's tables.
pub struct Row {
    /// Who it is among its situation's rows or its table's, which a beats clause names.
    pub id: RowId,
    /// What it needs, all of it.
    pub needs: &'static [Test],
    /// The pose it leads to.
    pub next: Next,
    /// What it does.
    pub effects: &'static [Effect],
    /// `PadChart`: the rows of its situation it beats.
    pub beats: &'static [RowId],
    /// `PadChart`: the rows of its situation it beats, but for a condition.
    pub beats_unless: &'static [Beat],
    /// `AimingAndShooting`: its kept running pose counts as chosen, bringing the step.
    pub steps: bool,
}

/// A row's identity: its first cell in §7's tables, the Pad cell for the pad chart's rows. It is
/// one row's alone within its situation or its table (the chart's test holds it), so a beats
/// clause names the rows it beats by it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RowId {
    /// "down pressed".
    DownPressed,
    /// "jump pressed".
    JumpPressed,
    /// "up pressed".
    UpPressed,
    /// "back pressed".
    BackPressed,
    /// "shoot pressed".
    ShootPressed,
    /// "back, shoot and an aim button held".
    BackShootAndAnAimButtonHeld,
    /// "back and both aim buttons held".
    BackAndBothAimButtonsHeld,
    /// "back and shoot held".
    BackAndShootHeld,
    /// "back and an aim button held".
    BackAndAnAimButtonHeld,
    /// "forward and an aim button held".
    ForwardAndAnAimButtonHeld,
    /// "forward with up or down held".
    ForwardWithUpOrDownHeld,
    /// "forward with up, down or an aim button held".
    ForwardWithUpDownOrAnAimButtonHeld,
    /// "forward and shoot held".
    ForwardAndShootHeld,
    /// "forward and jump held".
    ForwardAndJumpHeld,
    /// "both aim buttons held".
    BothAimButtonsHeld,
    /// "forward held".
    ForwardHeld,
    /// "back held".
    BackHeld,
    /// "up held".
    UpHeld,
    /// "aim up or aim down held".
    AimUpOrAimDownHeld,
    /// "up, down or an aim button held".
    UpDownOrAnAimButtonHeld,
    /// "shoot held".
    ShootHeld,
    /// "shoot with up, down or an aim button held".
    ShootWithUpDownOrAnAimButtonHeld,
    /// "jump held".
    JumpHeld,
    /// "gun stowed: jump held".
    GunStowedJumpHeld,
    /// "jump and forward held".
    JumpAndForwardHeld,
    /// "jump with up, down or an aim button held".
    JumpWithUpDownOrAnAimButtonHeld,
    /// "jump and back held".
    JumpAndBackHeld,
    /// "aiming `Down`: down pressed".
    AimingDownDownPressed,
    /// "aiming `Down`, she faces `Right`: forward, jump and shoot held".
    AimingDownFacingRightForwardJumpAndShootHeld,
    /// "aiming `Down`: jump held, and no other button a row names".
    AimingDownJumpHeldAlone,
    /// "jump pressed, with the spring ball, on the ground".
    JumpPressedWithTheSpringBall,
    /// "without the spring ball jump pressed".
    JumpPressedWithoutTheSpringBall,
    /// "left or right held": right held.
    RightHeld,
    /// "left or right held": left held.
    LeftHeld,
    /// "off the ground, with the spring ball".
    OffTheGroundWithTheSpringBall,
    /// "aimed `Up` or `DownForward`: forward held and jump pressed".
    AimedUpOrDownForwardForwardHeldAndJumpPressed,
    /// "aimed `Forward`: forward and jump held".
    AimedForwardForwardAndJumpHeld,
    /// "any button held".
    AnyButtonHeld,
    /// "nothing held".
    NothingHeld,
    /// "the way she walks, shoot and an aim button held".
    WalksShootAndAnAimButtonHeld,
    /// "the way she walks and shoot held".
    WalksAndShootHeld,
    /// "the way she looks held".
    LooksHeld,
    /// "the way she walks held".
    WalksHeld,
    /// "anything else, or nothing held".
    AnythingElse,
    /// Running's "anything else, or nothing held", her horizontal speed at the frame's start not
    /// 0.
    AnythingElseMoving,
    /// Running's "anything else, or nothing held", her horizontal speed at the frame's start 0.
    AnythingElseAtRest,
    /// A ball's "anything else, or nothing held: on the ground".
    AnythingElseOnTheGround,
    /// A ball's "in the air, without the spring ball".
    InTheAirWithoutTheSpringBall,
    /// A changing body's "anything, or nothing held".
    AnythingOrNothingHeld,
    /// `WhatHerMovesMet`: a wall stop, running aimed `Up`.
    WallStopAimedUp,
    /// `WhatHerMovesMet`: a wall stop, running.
    WallStop,
    /// `WhatHerMovesMet`: a wall near her as she presses jump, spinning.
    WallKick,
    /// `WhatHerMovesMet`: a floor, upright in a straight jump or falling, aimed `Forward` with
    /// shoot held: she lands firing.
    FloorFiring,
    /// `WhatHerMovesMet`: a floor, upright in a straight jump or falling.
    FloorFromAJump,
    /// `WhatHerMovesMet`: a floor, spinning or wall jumping.
    FloorFromASpin,
    /// `WhatHerMovesMet`: a floor, a ball bouncing.
    FloorBouncing,
    /// `WhatHerMovesMet`: a floor, a ball coming to rest.
    FloorComingToRest,
    /// `WhatHerMovesMet`: no floor, upright or crouched on the ground.
    NoFloorUprightOrCrouched,
    /// `WhatHerMovesMet`: no floor, a ball.
    NoFloorBall,
    /// `WhatHerMovesMet`: a ceiling, rising.
    Ceiling,
    /// `AnimationEnds`: `TakingOff`.
    TakingOffEnds,
    /// `AnimationEnds`: crouched `Turning`.
    CrouchedTurnEnds,
    /// `AnimationEnds`: `Turning` on the ground.
    GroundTurnEnds,
    /// `AnimationEnds`: `Turning` in a jump, aimed `Forward`.
    JumpTurnEndsAimedForward,
    /// `AnimationEnds`: `Turning` in a jump.
    JumpTurnEnds,
    /// `AnimationEnds`: `Turning` in a fall.
    FallTurnEnds,
    /// `AnimationEnds`: `Turning` from the moonwalk's jump.
    MoonwalkJumpTurnEnds,
    /// `AnimationEnds`: `Crouching`.
    CrouchingEnds,
    /// `AnimationEnds`: `StandingUp`.
    StandingUpEnds,
    /// `AnimationEnds`: `Morphing`, with the spring ball, in the air.
    MorphingEndsSpringBallInTheAir,
    /// `AnimationEnds`: `Morphing`, with the spring ball.
    MorphingEndsSpringBall,
    /// `AnimationEnds`: `Morphing`, in the air.
    MorphingEndsInTheAir,
    /// `AnimationEnds`: `Morphing`.
    MorphingEnds,
    /// `AnimationEnds`: `Landing`.
    LandingEnds,
    /// `AnimationEnds`: `Unmorphing`.
    UnmorphingEnds,
}

/// `PadChart`: an aim button, aim up or aim down.
pub const AIM: Buttons = Buttons(button::AIM_UP.0 | button::AIM_DOWN.0);
/// `PadChart`: up or down.
pub const UP_DOWN: Buttons = Buttons(button::UP.0 | button::DOWN.0);
/// `PadChart`: up, down or an aim button.
pub const UDA: Buttons = Buttons(UP_DOWN.0 | AIM.0);

/// A row's defaults: the last row's identity; it needs nothing, keeps her pose, does nothing and
/// beats nothing.
pub const ROW: Row = Row {
    id: RowId::AnythingElse,
    needs: &[],
    next: KEPT,
    effects: &[],
    beats: &[],
    beats_unless: &[],
    steps: false,
};

/// "kept": her pose stays as it is.
pub const KEPT: Next = Next {
    body: None,
    action: None,
    facing: NextFacing::Kept,
    aim: &[],
    gun: None,
    legs: None,
    lower: None,
};

/// Upright `Still`, aim `Forward`, the gun stowed and the legs together (`PadChart`: a next pose
/// has its gun stowed and its legs together unless its cell says otherwise).
pub const STILL: Next = Next {
    body: Some(Body::Upright),
    action: Some(Action::Still),
    facing: NextFacing::Kept,
    aim: FORWARD_AIM,
    gun: Some(Gun::Stowed),
    legs: Some(Legs::Together),
    lower: None,
};

/// `ShrinkingLowersHer`: standing up or unmorphing lowers her 0 px.
pub const GROWS: Option<Lowers> = Some(Lowers::Nothing);
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
/// Crouched `Crouching`, `Forward`: she is lowered by the crouch's distance, 5 px in vanilla
/// (`ShrinkingLowersHer`).
pub const CROUCH: Next = Next {
    body: Some(Body::Crouched),
    action: Some(Action::Crouching),
    lower: Some(Lowers::Crouch),
    ..STILL
};
/// Ball `Morphing`: she is lowered by the morph's distance, 9 px in vanilla
/// (`ShrinkingLowersHer`).
pub const MORPH: Next = Next {
    body: Some(Body::Ball),
    action: Some(Action::Morphing),
    lower: Some(Lowers::Morph),
    ..STILL
};
/// `Turning` on the ground: facing back, with the aim she had (§2).
pub const TURN: Next = Next {
    action: Some(Action::Turning(clean_types::Turn::Ground)),
    facing: NextFacing::Back,
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

/// `WhatHerMovesMet`: what this frame's moves met, before the aim change's step (a block in its
/// way puts the running pose chosen against the wall, `chart::table::STEP_BLOCKED`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
