//! Her situation, classified once a frame from her pose at its start (`MovesAsShePosed`).
//!
//! The situations are the states of the pose chart's state machine (§7), each with what she is
//! doing, how she moves, and its pad chart's rows (`PadChart`), the moves out of it.
//!
//! A pad chart row's order in its situation decides nothing (`PadChart`: rank and beats clauses
//! do). Where left and right are held together, rows naming forward and back both fit, and the
//! beats clauses between them are `PadChart`'s ranks for both held.
//!
//! A ball `Still` or `Rolling` is on the ground: in the air a ball keeps its pose, falling or on
//! its hop (`PadChart`, `WhatHerMovesMet`), so her pose alone gives her situation.

use crate::rows::Effect::{
    AsRunning, EndSkid, Halt, Launch, Lift, LoseDash, Rise, SpinOn, TurnRound,
};
use crate::rows::{
    AIM, AIM_UP_ELSE_DOWN, Aiming, BACK, Beat, CROUCH, Class, Doing, Drift, Entering, FALL,
    FORWARD, FORWARD_AIM, GROWS, JUMP, KEPT, MOONWALK, MORPH, Moves, Next, ROW, RUN, Row, SPIN,
    STILL, Side, Situation, TAKE_OFF, TURN, Test, UDA, UP_AIM, UP_DOWN, Was,
};
use crate::rules::Gait;
use Test::{Aimed, Faces, Held, Not, Pressed};
use clean_types::Action::{
    AgainstWall, Crouching, Falling, Jumping, Landing, Moonwalking, Morphing, Rolling, Running,
    Spinning, StandingUp, Still, TakingOff, Turning, Unmorphing, WallJumping,
};
use clean_types::Body::{Ball, Crouched, SpringBall, Upright};
use clean_types::button::{self as key, AIM_DOWN, AIM_UP, DOWN, LEFT, RIGHT, SHOOT, UP};
use clean_types::{Action, Aim, Body, Buttons, Facing, Gun, Heading, LandedFrom, Legs, Pose, Turn};

/// `MovesAsShePosed`: her situation, from her pose at the frame's start: the first class of
/// `CLASSES` her pose is in, else `OTHERWISE`.
#[must_use]
pub fn classify(pose: Pose) -> &'static Situation {
    let fits = |c: &&Class| {
        c.body.contains(&pose.body) && c.action.contains(&pose.action) && c.aim.contains(&pose.aim)
    };
    CLASSES.iter().find(fits).map_or(OTHERWISE, |c| c.is)
}

/// Standing still, or landing: the situation of every pose no class names (upright `Still`,
/// `Landing`).
const OTHERWISE: &Situation = &STANDING;

/// Every body.
const BODIES: &[Body] = &[Upright, Crouched, Ball, SpringBall];
/// Every aim.
const AIMS: &[Aim] = &[
    Aim::Forward,
    Aim::Up,
    Aim::UpForward,
    Aim::DownForward,
    Aim::Down,
];
/// Every action, each once (the chart's test `every_action_is_listed` holds it to `Action`).
pub(crate) const ACTIONS: &[Action] = &[
    AgainstWall,
    Crouching,
    Falling,
    Jumping,
    Landing(LandedFrom::Jump),
    Landing(LandedFrom::Spin),
    Moonwalking,
    Morphing,
    Rolling,
    Running,
    Spinning,
    StandingUp,
    Still,
    TakingOff,
    Turning(Turn::Fall),
    Turning(Turn::Ground),
    Turning(Turn::Jump),
    Turning(Turn::MoonwalkJump),
    Unmorphing,
    WallJumping,
];
/// Every turn.
const TURNS: &[Action] = &[
    Turning(Turn::Fall),
    Turning(Turn::Ground),
    Turning(Turn::Jump),
    Turning(Turn::MoonwalkJump),
];
/// The change of action's own effects, whatever brought the change.
///
/// `Jumping`: starting a straight jump (upright) or a fall, from any action but that one, with
/// a dash she carries on along her facing, else she follows the pad. `WallJump` (§1.2): a spin
/// that followed a wall jump is carried on.
pub static ENTERING: &[Entering] = &[
    Entering {
        into: &[(Upright, Jumping)],
        from: Was::AnyBut(&[Jumping]),
        effect: Launch,
    },
    Entering {
        into: &[(Upright, Falling), (Ball, Falling), (SpringBall, Falling)],
        from: Was::AnyBut(&[Falling]),
        effect: Launch,
    },
    Entering {
        into: &[(Upright, Spinning)],
        from: Was::OneOf(&[WallJumping]),
        effect: SpinOn,
    },
];

/// A class of any body and aim.
const ANY: Class = Class {
    body: BODIES,
    action: ACTIONS,
    aim: AIMS,
    is: &STANDING,
};

/// The classes, in order: the first her pose is in gives her situation.
pub static CLASSES: &[Class] = &[
    Class {
        action: &[Crouching, StandingUp, Morphing, Unmorphing],
        is: &SHAPING,
        ..ANY
    },
    Class {
        body: &[Ball, SpringBall],
        action: &[Jumping],
        is: &HOP,
        ..ANY
    },
    Class {
        body: &[Ball, SpringBall],
        action: &[Rolling],
        is: &ROLLING,
        ..ANY
    },
    Class {
        body: &[Ball, SpringBall],
        action: &[Still],
        is: &RESTING,
        ..ANY
    },
    Class {
        body: &[Ball, SpringBall],
        is: &BALL_IN_AIR,
        ..ANY
    },
    Class {
        body: &[Crouched],
        action: TURNS,
        is: &TURNING_ON,
        ..ANY
    },
    Class {
        action: &[Turning(Turn::Jump), Turning(Turn::Fall)],
        is: &TURNING_ON,
        ..ANY
    },
    Class {
        action: TURNS,
        aim: &[Aim::UpForward],
        is: &TURNING_ON,
        ..ANY
    },
    Class {
        action: TURNS,
        is: &TURNING,
        ..ANY
    },
    Class {
        body: &[Crouched],
        is: &CROUCHED,
        ..ANY
    },
    Class {
        action: &[Running],
        is: &RUNNING,
        ..ANY
    },
    Class {
        action: &[TakingOff],
        is: &TAKING_OFF,
        ..ANY
    },
    Class {
        action: &[Jumping],
        is: &STRAIGHT_JUMP,
        ..ANY
    },
    Class {
        action: &[Spinning],
        is: &SPINNING,
        ..ANY
    },
    Class {
        action: &[Falling],
        is: &FALLING,
        ..ANY
    },
    Class {
        action: &[Moonwalking],
        is: &MOONWALKING,
        ..ANY
    },
    Class {
        action: &[WallJumping],
        is: &WALL_JUMPING,
        ..ANY
    },
    Class {
        action: &[AgainstWall],
        is: &AGAINST_WALL,
        ..ANY
    },
];

/// `Aiming`: `UpForward` with up, else `DownForward`.
const UP_ELSE_DOWN: &[Aiming] = &[
    Aiming {
        when: &[Held(UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[],
        aim: Aim::DownForward,
    },
];
/// `Aiming`: `DownForward` with aim down, else `UpForward`.
const AIM_DOWN_ELSE_UP: &[Aiming] = &[
    Aiming {
        when: &[Held(AIM_DOWN)],
        aim: Aim::DownForward,
    },
    Aiming {
        when: &[],
        aim: Aim::UpForward,
    },
];
/// `Aiming`'s crouch rule.
const CROUCH_AIM: &[Aiming] = &[
    Aiming {
        when: &[Held(AIM_UP), Held(AIM_DOWN)],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Held(AIM_UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Held(AIM_DOWN)],
        aim: Aim::DownForward,
    },
    Aiming {
        when: &[],
        aim: Aim::Forward,
    },
];
/// `Aiming`'s take-off rule.
const TAKE_OFF_AIM: &[Aiming] = &[
    Aiming {
        when: &[Held(UP)],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Held(AIM_UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Held(AIM_DOWN)],
        aim: Aim::DownForward,
    },
    Aiming {
        when: &[],
        aim: Aim::Forward,
    },
];
/// `Aiming`'s air rule.
const AIR_AIM: &[Aiming] = &[
    Aiming {
        when: &[Held(FORWARD), Held(UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Held(FORWARD), Held(DOWN)],
        aim: Aim::DownForward,
    },
    Aiming {
        when: &[Held(UP)],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Held(DOWN)],
        aim: Aim::Down,
    },
    Aiming {
        when: &[Held(AIM_UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Held(AIM_DOWN)],
        aim: Aim::DownForward,
    },
    Aiming {
        when: &[],
        aim: Aim::Forward,
    },
];
/// `Up` with up, else `Down` with down, else `UpForward` with aim up, else `DownForward`.
const UP_DOWN_AIM: &[Aiming] = &[
    Aiming {
        when: &[Held(UP)],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Held(DOWN)],
        aim: Aim::Down,
    },
    Aiming {
        when: &[Held(AIM_UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[],
        aim: Aim::DownForward,
    },
];
/// `Up` with up, else `UpForward` with aim up, else `DownForward` with aim down, else `Down`.
const SPIN_AIM: &[Aiming] = &[
    Aiming {
        when: &[Held(UP)],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Held(AIM_UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Held(AIM_DOWN)],
        aim: Aim::DownForward,
    },
    Aiming {
        when: &[],
        aim: Aim::Down,
    },
];
/// The moonwalk's: `UpForward` with aim up, else `DownForward`; with both aim buttons held,
/// `UpForward` facing left and `DownForward` facing right.
const MOON_AIM: &[Aiming] = &[
    Aiming {
        when: &[Held(AIM_UP), Held(AIM_DOWN), Faces(Heading::Left)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Held(AIM_UP), Held(AIM_DOWN)],
        aim: Aim::DownForward,
    },
    Aiming {
        when: &[Held(AIM_UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[],
        aim: Aim::DownForward,
    },
];
/// §2 aim in a turn: a turn in a jump or a fall shows `DownForward` as `Down`; else kept.
const TURN_IN_THE_AIR: &[Aiming] = &[Aiming {
    when: &[Aimed(&[Aim::DownForward])],
    aim: Aim::Down,
}];
/// `Forward`, except landing: kept as she is.
const STANDS_FORWARD: &[Aiming] = &[Aiming {
    when: &[Test::Action(&[Still])],
    aim: Aim::Forward,
}];

/// `Turning` in a jump.
const TURN_IN_A_JUMP: Next = Next {
    action: Some(Turning(Turn::Jump)),
    aim: TURN_IN_THE_AIR,
    ..TURN
};
/// `Turning` in a fall.
const TURN_IN_A_FALL: Next = Next {
    action: Some(Turning(Turn::Fall)),
    aim: TURN_IN_THE_AIR,
    ..TURN
};
/// Crouched `Unmorphing`, which lowers her 0 px (`ShrinkingLowersHer`).
const UNMORPH: Next = Next {
    body: Some(Crouched),
    action: Some(Unmorphing),
    lower: GROWS,
    ..STILL
};
/// The same ball, rolling.
const ROLL: Next = Next {
    body: None,
    action: Some(Rolling),
    ..STILL
};

/// Any button.
const ANY_BUTTON: Buttons = Buttons(u16::MAX);
/// Every button a straight jump's row names but jump.
const NAMED: Buttons = Buttons(FORWARD.0 | BACK.0 | SHOOT.0 | UDA.0);

/// Standing still, or landing.
pub static STANDING: Situation = Situation {
    doing: Doing::Standing,
    moves: Moves::DashThenHalt,
    rows: &[
        Row {
            name: "down pressed",
            needs: &[Pressed(DOWN)],
            next: Next {
                aim: CROUCH_AIM,
                ..CROUCH
            },
            ..ROW
        },
        Row {
            name: "jump pressed",
            needs: &[Pressed(key::JUMP)],
            next: Next {
                aim: TAKE_OFF_AIM,
                ..TAKE_OFF
            },
            effects: &[Rise],
            beats: &["down pressed"],
            ..ROW
        },
        Row {
            name: "back, shoot and an aim button held",
            needs: &[Held(BACK), Held(SHOOT), Held(AIM)],
            next: Next {
                facing: Side::Back,
                aim: AIM_DOWN_ELSE_UP,
                ..MOONWALK
            },
            beats: &["back and both aim buttons held"],
            ..ROW
        },
        Row {
            name: "back and both aim buttons held",
            needs: &[Held(BACK), Held(AIM_UP), Held(AIM_DOWN)],
            next: TURN,
            effects: &[TurnRound],
            ..ROW
        },
        Row {
            name: "back and shoot held",
            needs: &[Held(BACK), Held(SHOOT)],
            next: Next {
                facing: Side::Back,
                ..MOONWALK
            },
            ..ROW
        },
        Row {
            name: "forward and an aim button held",
            needs: &[Held(FORWARD), Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..RUN
            },
            beats: &["forward with up or down held"],
            ..ROW
        },
        Row {
            name: "forward with up or down held",
            needs: &[Held(FORWARD), Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..RUN
            },
            beats: &["back and shoot held"],
            ..ROW
        },
        Row {
            name: "both aim buttons held",
            needs: &[Held(AIM_UP), Held(AIM_DOWN)],
            next: Next {
                aim: UP_AIM,
                ..STILL
            },
            beats: &[
                "back and shoot held",
                "forward and an aim button held",
                "forward with up or down held",
            ],
            ..ROW
        },
        Row {
            name: "forward held",
            needs: &[Held(FORWARD)],
            next: RUN,
            ..ROW
        },
        Row {
            name: "up held",
            needs: &[Held(UP)],
            next: Next {
                aim: UP_AIM,
                ..STILL
            },
            beats: &["aim up or aim down held"],
            ..ROW
        },
        Row {
            name: "aim up or aim down held",
            needs: &[Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..STILL
            },
            ..ROW
        },
        Row {
            name: "back held",
            needs: &[Held(BACK)],
            next: TURN,
            effects: &[TurnRound],
            beats: &["up held", "aim up or aim down held", "forward held"],
            ..ROW
        },
    ],
    last: Row {
        name: "anything else",
        next: Next {
            aim: STANDS_FORWARD,
            ..KEPT
        },
        effects: &[EndSkid],
        ..ROW
    },
};

/// Running.
static RUNNING: Situation = Situation {
    doing: Doing::Running,
    moves: Moves::Build(Gait::Running, Drift::Ground),
    rows: &[
        Row {
            name: "down pressed",
            needs: &[Pressed(DOWN)],
            next: CROUCH,
            beats: &["jump pressed"],
            ..ROW
        },
        Row {
            name: "jump pressed",
            needs: &[Pressed(key::JUMP)],
            next: SPIN,
            effects: &[Rise],
            ..ROW
        },
        Row {
            name: "forward and shoot held",
            needs: &[Held(FORWARD), Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..RUN
            },
            ..ROW
        },
        Row {
            name: "forward and an aim button held",
            needs: &[Held(FORWARD), Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..RUN
            },
            beats: &["forward with up or down held", "forward and shoot held"],
            ..ROW
        },
        Row {
            name: "forward with up or down held",
            needs: &[Held(FORWARD), Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..RUN
            },
            beats: &["forward and shoot held"],
            ..ROW
        },
        Row {
            name: "forward held",
            needs: &[Held(FORWARD)],
            next: Next { gun: None, ..RUN },
            beats: &["back held"],
            ..ROW
        },
        Row {
            name: "up held",
            needs: &[Held(UP)],
            next: Next {
                aim: UP_AIM,
                ..STILL
            },
            beats: &["aim up or aim down held"],
            ..ROW
        },
        Row {
            name: "aim up or aim down held",
            needs: &[Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..STILL
            },
            ..ROW
        },
        Row {
            name: "back held",
            needs: &[Held(BACK)],
            next: TURN,
            effects: &[TurnRound],
            beats: &["up held", "aim up or aim down held"],
            ..ROW
        },
        // The chart's last row for running gives two outcomes, by her horizontal speed at the
        // frame's start; a row has one next pose, so the cell is two rows. This one, kept while
        // the speed isn't 0, names no button: every other fitting row outranks it, so it is
        // used only where the last row would be, and its kept pose brings the step.
        Row {
            name: "anything else, while her horizontal speed at the frame's start isn't 0",
            needs: &[Test::Moving],
            effects: &[AsRunning],
            steps: true,
            ..ROW
        },
    ],
    last: Row {
        name: "anything else, her horizontal speed at the frame's start 0",
        next: STILL,
        effects: &[AsRunning],
        ..ROW
    },
};

/// Taking off.
static TAKING_OFF: Situation = Situation {
    doing: Doing::TakingOff,
    moves: Moves::TakeOff,
    rows: &[
        Row {
            name: "jump and forward held",
            needs: &[Held(key::JUMP), Held(FORWARD)],
            next: Next {
                legs: Some(Legs::Striding),
                ..JUMP
            },
            ..ROW
        },
        Row {
            name: "jump with up, down or an aim button held",
            needs: &[Held(key::JUMP), Held(UDA)],
            next: Next {
                aim: UP_DOWN_AIM,
                ..JUMP
            },
            beats: &["jump and forward held"],
            ..ROW
        },
        Row {
            name: "jump and back held",
            needs: &[Held(key::JUMP), Held(BACK)],
            next: TURN_IN_A_JUMP,
            effects: &[TurnRound],
            beats: &[
                "jump with up, down or an aim button held",
                "jump and forward held",
            ],
            ..ROW
        },
        Row {
            name: "shoot held",
            needs: &[Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..JUMP
            },
            ..ROW
        },
    ],
    last: Row {
        name: "anything else",
        effects: &[AsRunning],
        ..ROW
    },
};

/// In a straight jump.
static STRAIGHT_JUMP: Situation = Situation {
    doing: Doing::StraightJump,
    moves: Moves::Build(Gait::StraightJump, Drift::Air),
    rows: &[
        Row {
            name: "aiming Down: down pressed",
            needs: &[Aimed(&[Aim::Down]), Pressed(DOWN)],
            next: MORPH,
            ..ROW
        },
        Row {
            name: "aiming Down, she faces Right: forward, jump and shoot held",
            needs: &[
                Aimed(&[Aim::Down]),
                Faces(Heading::Right),
                Held(FORWARD),
                Held(key::JUMP),
                Held(SHOOT),
            ],
            next: Next {
                gun: Some(Gun::Drawn),
                ..JUMP
            },
            ..ROW
        },
        Row {
            name: "forward with up, down or an aim button held",
            needs: &[Held(FORWARD), Held(UDA)],
            next: Next {
                aim: AIR_AIM,
                ..JUMP
            },
            beats: &["aiming Down, she faces Right: forward, jump and shoot held"],
            ..ROW
        },
        Row {
            name: "forward held",
            needs: &[Held(FORWARD)],
            next: Next {
                legs: Some(Legs::Striding),
                ..JUMP
            },
            beats: &["shoot held", "gun stowed: jump held"],
            ..ROW
        },
        Row {
            name: "shoot held",
            needs: &[Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..JUMP
            },
            beats: &["gun stowed: jump held"],
            ..ROW
        },
        Row {
            name: "gun stowed: jump held",
            needs: &[Test::Gun(Gun::Stowed), Held(key::JUMP)],
            next: JUMP,
            ..ROW
        },
        Row {
            name: "up, down or an aim button held",
            needs: &[Held(UDA)],
            next: Next {
                aim: AIR_AIM,
                ..JUMP
            },
            beats: &["forward held", "shoot held", "gun stowed: jump held"],
            ..ROW
        },
        Row {
            name: "back held",
            needs: &[Held(BACK)],
            next: TURN_IN_A_JUMP,
            effects: &[TurnRound],
            beats: &[
                "forward held",
                "shoot held",
                "gun stowed: jump held",
                "up, down or an aim button held",
                "aiming Down: jump held, and no other button a row names",
            ],
            // PadChart, left and right both held: back held beats forward with an aim button
            // held, unless jump is held too (with up or down held instead, forward wins by rank).
            beats_unless: &[Beat {
                row: "forward with up, down or an aim button held",
                unless: &[&[Not(&Held(AIM))], &[Held(key::JUMP)]],
            }],
            ..ROW
        },
        Row {
            name: "aiming Down: jump held, and no other button a row names",
            needs: &[Aimed(&[Aim::Down]), Held(key::JUMP), Not(&Held(NAMED))],
            beats: &["gun stowed: jump held"],
            ..ROW
        },
    ],
    last: Row {
        name: "anything else",
        effects: &[AsRunning],
        ..ROW
    },
};

/// Spinning.
static SPINNING: Situation = Situation {
    doing: Doing::Spinning,
    moves: Moves::Build(Gait::SpinJump, Drift::Spin),
    rows: &[
        Row {
            name: "shoot pressed",
            needs: &[Pressed(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..JUMP
            },
            ..ROW
        },
        Row {
            name: "shoot with up, down or an aim button held",
            needs: &[Held(SHOOT), Held(UDA)],
            next: Next {
                aim: UP_DOWN_AIM,
                ..JUMP
            },
            beats: &["forward and jump held"],
            ..ROW
        },
        Row {
            name: "forward and jump held",
            needs: &[Held(FORWARD), Held(key::JUMP)],
            ..ROW
        },
        Row {
            name: "up, down or an aim button held",
            needs: &[Held(UDA)],
            next: Next {
                aim: SPIN_AIM,
                ..JUMP
            },
            beats: &["forward held", "back held"],
            ..ROW
        },
        Row {
            name: "back held",
            needs: &[Held(BACK)],
            next: Next {
                facing: Side::Back,
                ..SPIN
            },
            effects: &[TurnRound],
            ..ROW
        },
        Row {
            name: "forward held",
            needs: &[Held(FORWARD)],
            beats: &["back held"],
            ..ROW
        },
    ],
    last: Row {
        name: "anything else",
        ..ROW
    },
};

/// Falling, upright.
static FALLING: Situation = Situation {
    doing: Doing::Falling,
    moves: Moves::Build(Gait::Falling, Drift::Air),
    rows: &[
        Row {
            name: "aiming Down: down pressed",
            needs: &[Aimed(&[Aim::Down]), Pressed(DOWN)],
            next: MORPH,
            ..ROW
        },
        Row {
            name: "forward with up or down held",
            needs: &[Held(FORWARD), Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..FALL
            },
            ..ROW
        },
        Row {
            name: "shoot held",
            needs: &[Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..FALL
            },
            beats: &["forward held"],
            ..ROW
        },
        Row {
            name: "forward held",
            needs: &[Held(FORWARD)],
            next: Next { gun: None, ..FALL },
            ..ROW
        },
        Row {
            name: "up, down or an aim button held",
            needs: &[Held(UDA)],
            next: Next {
                aim: AIR_AIM,
                ..FALL
            },
            beats: &["shoot held", "forward held"],
            ..ROW
        },
        // "Aiming Down or with the gun drawn, up or down held beats it": the clause's condition,
        // under which the row it beats beats it instead.
        Row {
            name: "back held",
            needs: &[Held(BACK)],
            next: TURN_IN_A_FALL,
            effects: &[TurnRound],
            beats: &["shoot held", "forward held"],
            beats_unless: &[Beat {
                row: "up, down or an aim button held",
                unless: &[
                    &[Aimed(&[Aim::Down]), Held(UP_DOWN)],
                    &[Test::Gun(Gun::Drawn), Held(UP_DOWN)],
                ],
            }],
            ..ROW
        },
    ],
    last: Row {
        name: "anything else",
        next: Next {
            aim: FORWARD_AIM,
            ..KEPT
        },
        effects: &[LoseDash],
        ..ROW
    },
};

/// Crouched.
static CROUCHED: Situation = Situation {
    doing: Doing::Crouched,
    moves: Moves::DashThenHalt,
    rows: &[
        Row {
            name: "down pressed",
            needs: &[Pressed(DOWN)],
            next: MORPH,
            beats: &["jump pressed"],
            ..ROW
        },
        Row {
            name: "up pressed",
            needs: &[Pressed(UP)],
            next: Next {
                action: Some(StandingUp),
                aim: CROUCH_AIM,
                lower: GROWS,
                ..STILL
            },
            beats: &["down pressed", "jump pressed", "back pressed"],
            ..ROW
        },
        Row {
            name: "jump pressed",
            needs: &[Pressed(key::JUMP)],
            next: TAKE_OFF,
            effects: &[Rise, Lift],
            ..ROW
        },
        Row {
            name: "back pressed",
            needs: &[Pressed(BACK)],
            next: Next {
                body: Some(Crouched),
                ..TURN
            },
            beats: &["down pressed", "jump pressed"],
            ..ROW
        },
        Row {
            name: "forward and an aim button held",
            needs: &[Held(FORWARD), Held(AIM)],
            next: STILL,
            ..ROW
        },
        Row {
            name: "both aim buttons held",
            needs: &[Held(AIM_UP), Held(AIM_DOWN)],
            next: Next {
                body: Some(Crouched),
                aim: UP_AIM,
                ..STILL
            },
            beats: &["forward and an aim button held"],
            ..ROW
        },
        Row {
            name: "forward held",
            needs: &[Held(FORWARD)],
            next: STILL,
            ..ROW
        },
        Row {
            name: "aim up or aim down held",
            needs: &[Held(AIM)],
            next: Next {
                body: Some(Crouched),
                aim: AIM_UP_ELSE_DOWN,
                ..STILL
            },
            ..ROW
        },
    ],
    last: Row {
        name: "anything else",
        next: Next {
            aim: FORWARD_AIM,
            ..KEPT
        },
        effects: &[EndSkid],
        ..ROW
    },
};

/// A ball's "up pressed" row, which beats the jump rows (`PadChart`: "beats the spring ball's
/// hop"; in the air, where there is no hop, the jump without the spring ball).
const UNMORPH_ROW: Row = Row {
    name: "up pressed",
    needs: &[Pressed(UP)],
    next: UNMORPH,
    beats: &["jump pressed, without the spring ball"],
    ..ROW
};
/// A ball's "without the spring ball jump pressed" row.
const UNMORPH_JUMP: Row = Row {
    name: "jump pressed, without the spring ball",
    needs: &[Pressed(key::JUMP), Test::Body(&[Ball])],
    next: UNMORPH,
    ..ROW
};

/// A ball on the ground: the spring ball's hop, unmorphing, and the side of the pad held, which
/// it faces, rolling. Left and right both held (`PadChart`), it rolls rightward.
const BALL_ON_GROUND: &[Row] = &[
    Row {
        name: "jump pressed, with the spring ball, on the ground",
        needs: &[Pressed(key::JUMP), Test::Body(&[SpringBall])],
        next: Next { body: None, ..JUMP },
        effects: &[Rise],
        ..ROW
    },
    Row {
        beats: &[
            "jump pressed, with the spring ball, on the ground",
            "jump pressed, without the spring ball",
        ],
        ..UNMORPH_ROW
    },
    UNMORPH_JUMP,
    Row {
        name: "right held",
        needs: &[Held(RIGHT)],
        next: Next {
            facing: Side::To(Facing::Right),
            ..ROLL
        },
        effects: &[TurnRound],
        beats: &["left held"],
        ..ROW
    },
    Row {
        name: "left held",
        needs: &[Held(LEFT)],
        next: Next {
            facing: Side::To(Facing::Left),
            ..ROLL
        },
        effects: &[TurnRound],
        ..ROW
    },
];

/// The ball in the air: unmorphing, the side of the pad held, which it faces, and the spring
/// ball's let-go row (the plain ball's is the last). Left and right both held (`PadChart`): the
/// spring ball turns round (the spring ball falling: in the air it is either falling or on its
/// hop, its own situation), and the ball without it keeps its facing; as one clause, right held
/// wins unless the plain ball faces left or the spring ball faces right.
const BALL_IN_THE_AIR: &[Row] = &[
    UNMORPH_ROW,
    UNMORPH_JUMP,
    Row {
        name: "right held",
        needs: &[Held(RIGHT)],
        next: Next {
            facing: Side::To(Facing::Right),
            ..KEPT
        },
        effects: &[TurnRound],
        beats_unless: &[Beat {
            row: "left held",
            unless: &[
                &[Test::Body(&[Ball]), Faces(Heading::Left)],
                &[Test::Body(&[SpringBall]), Faces(Heading::Right)],
            ],
        }],
        ..ROW
    },
    Row {
        name: "left held",
        needs: &[Held(LEFT)],
        next: Next {
            facing: Side::To(Facing::Left),
            ..KEPT
        },
        effects: &[TurnRound],
        ..ROW
    },
    Row {
        name: "off the ground, with the spring ball",
        needs: &[Test::Body(&[SpringBall])],
        effects: &[Halt],
        ..ROW
    },
];

/// The spring ball's hop: unmorphing and the side of the pad held, which it faces. Left and
/// right both held (`PadChart`), it turns rightward.
const BALL_ON_THE_HOP: &[Row] = &[
    Row {
        beats: &[],
        ..UNMORPH_ROW
    },
    Row {
        name: "right held",
        needs: &[Held(RIGHT)],
        next: Next {
            facing: Side::To(Facing::Right),
            ..KEPT
        },
        effects: &[TurnRound],
        beats: &["left held"],
        ..ROW
    },
    Row {
        name: "left held",
        needs: &[Held(LEFT)],
        next: Next {
            facing: Side::To(Facing::Left),
            ..KEPT
        },
        effects: &[TurnRound],
        ..ROW
    },
];

/// The ball's last row on the ground: ball `Still`, halted.
const BALL_STILL: Row = Row {
    name: "anything else: on the ground",
    next: Next {
        body: None,
        ..STILL
    },
    effects: &[Halt],
    ..ROW
};

/// The ball's last row in the air, without the spring ball: kept, as running.
const BALL_KEPT: Row = Row {
    name: "in the air, without the spring ball",
    effects: &[AsRunning],
    ..ROW
};

/// The spring ball's last row off the ground: kept, halted.
const SPRING_BALL_KEPT: Row = Row {
    name: "off the ground, with the spring ball",
    effects: &[Halt],
    ..ROW
};

/// A ball rolling on the ground.
static ROLLING: Situation = Situation {
    doing: Doing::Ball,
    moves: Moves::Build(Gait::Rolling, Drift::Ground),
    rows: BALL_ON_GROUND,
    last: BALL_STILL,
};

/// The ball at rest on the ground.
static RESTING: Situation = Situation {
    doing: Doing::Ball,
    moves: Moves::Rest,
    rows: BALL_ON_GROUND,
    last: BALL_STILL,
};

/// A ball in the air.
static BALL_IN_AIR: Situation = Situation {
    doing: Doing::Ball,
    moves: Moves::Build(Gait::Falling, Drift::BallAir),
    rows: BALL_IN_THE_AIR,
    last: BALL_KEPT,
};

/// The spring ball's hop.
static HOP: Situation = Situation {
    doing: Doing::Hop,
    moves: Moves::Build(Gait::StraightJump, Drift::BallAir),
    rows: BALL_ON_THE_HOP,
    last: SPRING_BALL_KEPT,
};

/// Turning on the ground or from the moonwalk's jump, not aimed `UpForward`.
static TURNING: Situation = Situation {
    doing: Doing::Turning,
    moves: Moves::Build(Gait::Turning, Drift::Ground),
    rows: &[
        Row {
            name: "aimed Up or DownForward: forward held and jump pressed",
            needs: &[
                Aimed(&[Aim::Up, Aim::DownForward]),
                Held(FORWARD),
                Pressed(key::JUMP),
            ],
            next: SPIN,
            effects: &[Rise],
            ..ROW
        },
        Row {
            name: "jump pressed",
            needs: &[Pressed(key::JUMP)],
            next: TAKE_OFF,
            effects: &[Rise],
            ..ROW
        },
        Row {
            name: "aimed Forward: forward and jump held",
            needs: &[Aimed(&[Aim::Forward]), Held(FORWARD), Held(key::JUMP)],
            next: SPIN,
            effects: &[Rise],
            beats: &["jump pressed"],
            ..ROW
        },
        Row {
            name: "forward held",
            needs: &[Held(FORWARD)],
            ..ROW
        },
    ],
    last: Row {
        name: "anything else",
        effects: &[EndSkid],
        ..ROW
    },
};

/// Turning in a jump or a fall, crouched turning, or turning on the ground or from the
/// moonwalk's jump aimed `UpForward`: she keeps turning.
static TURNING_ON: Situation = Situation {
    doing: Doing::Turning,
    moves: Moves::Build(Gait::Turning, Drift::Ground),
    rows: &[Row {
        name: "any button held",
        needs: &[Held(ANY_BUTTON)],
        ..ROW
    }],
    last: Row {
        name: "nothing held",
        effects: &[EndSkid],
        ..ROW
    },
};

/// Her body changing shape: the pad has no say until the animation ends.
static SHAPING: Situation = Situation {
    doing: Doing::Shaping,
    moves: Moves::Dash,
    rows: &[],
    last: Row {
        name: "anything, or nothing held",
        ..ROW
    },
};

/// Moonwalking: forward is the way she walks, which she faces; back the way she looks.
static MOONWALKING: Situation = Situation {
    doing: Doing::Moonwalking,
    moves: Moves::Build(Gait::Moonwalking, Drift::Ground),
    rows: &[
        Row {
            name: "down pressed",
            needs: &[Pressed(DOWN)],
            next: Next {
                facing: Side::Back,
                ..CROUCH
            },
            beats: &["jump pressed"],
            ..ROW
        },
        // The moonwalk's jump: the one turn that keeps her facing, the way she walks.
        Row {
            name: "jump pressed",
            needs: &[Pressed(key::JUMP)],
            next: Next {
                action: Some(Turning(Turn::MoonwalkJump)),
                facing: Side::Kept,
                ..TURN
            },
            effects: &[TurnRound],
            ..ROW
        },
        Row {
            name: "the way she walks, shoot and an aim button held",
            needs: &[Held(FORWARD), Held(SHOOT), Held(AIM)],
            next: Next {
                aim: MOON_AIM,
                ..MOONWALK
            },
            ..ROW
        },
        Row {
            name: "the way she walks and shoot held",
            needs: &[Held(FORWARD), Held(SHOOT)],
            next: MOONWALK,
            ..ROW
        },
        Row {
            name: "the way she looks held",
            needs: &[Held(BACK)],
            next: Next {
                facing: Side::Back,
                ..RUN
            },
            beats: &["the way she walks held"],
            ..ROW
        },
        Row {
            name: "the way she walks held",
            needs: &[Held(FORWARD)],
            next: TURN,
            effects: &[TurnRound],
            ..ROW
        },
    ],
    last: Row {
        name: "anything else",
        next: Next {
            facing: Side::Back,
            ..STILL
        },
        effects: &[EndSkid],
        ..ROW
    },
};

/// Wall jumping.
static WALL_JUMPING: Situation = Situation {
    doing: Doing::WallJumping,
    moves: Moves::Build(Gait::SpinJump, Drift::WallJump),
    rows: &[
        Row {
            name: "down pressed",
            needs: &[Pressed(DOWN)],
            next: MORPH,
            ..ROW
        },
        Row {
            name: "shoot held",
            needs: &[Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..JUMP
            },
            beats: &["jump held"],
            ..ROW
        },
        Row {
            name: "aim up or aim down held",
            needs: &[Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..JUMP
            },
            beats: &["shoot held", "jump held"],
            ..ROW
        },
        Row {
            name: "back held",
            needs: &[Held(BACK)],
            next: Next {
                facing: Side::Back,
                ..SPIN
            },
            effects: &[TurnRound],
            beats: &["shoot held", "aim up or aim down held", "jump held"],
            ..ROW
        },
        Row {
            name: "jump held",
            needs: &[Held(key::JUMP)],
            ..ROW
        },
    ],
    last: Row {
        name: "anything else",
        next: SPIN,
        effects: &[Halt],
        ..ROW
    },
};

/// Against the wall.
static AGAINST_WALL: Situation = Situation {
    doing: Doing::AgainstWall,
    moves: Moves::DashThenHalt,
    rows: &[
        Row {
            name: "down pressed",
            needs: &[Pressed(DOWN)],
            next: CROUCH,
            ..ROW
        },
        Row {
            name: "jump pressed",
            needs: &[Pressed(key::JUMP)],
            next: TAKE_OFF,
            effects: &[Rise],
            beats: &["down pressed"],
            ..ROW
        },
        Row {
            name: "back and an aim button held",
            needs: &[Held(BACK), Held(AIM)],
            next: Next {
                facing: Side::Back,
                aim: AIM_DOWN_ELSE_UP,
                ..MOONWALK
            },
            ..ROW
        },
        Row {
            name: "forward with up or down held",
            needs: &[Held(FORWARD), Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..RUN
            },
            beats: &["down pressed", "back and an aim button held"],
            ..ROW
        },
        Row {
            name: "forward held",
            needs: &[Held(FORWARD)],
            next: RUN,
            ..ROW
        },
        Row {
            name: "up held",
            needs: &[Held(UP)],
            next: Next {
                aim: UP_AIM,
                ..STILL
            },
            beats: &["aim up or aim down held", "back held"],
            ..ROW
        },
        Row {
            name: "aim up or aim down held",
            needs: &[Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..STILL
            },
            beats: &["forward held", "back held"],
            ..ROW
        },
        Row {
            name: "back held",
            needs: &[Held(BACK)],
            next: TURN,
            effects: &[TurnRound],
            beats: &["forward held"],
            ..ROW
        },
    ],
    last: Row {
        name: "anything else",
        next: Next {
            aim: FORWARD_AIM,
            ..KEPT
        },
        effects: &[EndSkid],
        ..ROW
    },
};
