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
    FORWARD, FORWARD_AIM, GROWS, JUMP, KEPT, MOONWALK, MORPH, Moves, Next, NextFacing, ROW, RUN,
    Row, RowId, SPIN, STILL, Situation, TAKE_OFF, TURN, Test, UDA, UP_AIM, UP_DOWN, Was,
};
use crate::rules::Gait;
use Test::{Aimed, Faces, Held, Not, Pressed};
use clean_types::Action::{
    AgainstWall, Crouching, Falling, Jumping, Landing, Moonwalking, Morphing, Rolling, Running,
    Spinning, StandingUp, Still, TakingOff, Turning, Unmorphing, WallJumping,
};
use clean_types::Body::{Ball, Crouched, SpringBall, Upright};
use clean_types::button::{self as key, AIM_DOWN, AIM_UP, DOWN, LEFT, RIGHT, SHOOT, UP};
use clean_types::{Action, Aim, Body, Buttons, Facing, Gun, LandedFrom, Legs, Pose, Side, Turn};

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
pub const ACTIONS: &[Action] = &[
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
        when: &[Held(AIM_UP), Held(AIM_DOWN), Faces(Side::Left)],
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
            id: RowId::DownPressed,
            needs: &[Pressed(DOWN)],
            next: Next {
                aim: CROUCH_AIM,
                ..CROUCH
            },
            ..ROW
        },
        Row {
            id: RowId::JumpPressed,
            needs: &[Pressed(key::JUMP)],
            next: Next {
                aim: TAKE_OFF_AIM,
                ..TAKE_OFF
            },
            effects: &[Rise],
            beats: &[RowId::DownPressed],
            ..ROW
        },
        Row {
            id: RowId::BackShootAndAnAimButtonHeld,
            needs: &[Held(BACK), Held(SHOOT), Held(AIM)],
            next: Next {
                facing: NextFacing::Back,
                aim: AIM_DOWN_ELSE_UP,
                ..MOONWALK
            },
            beats: &[RowId::BackAndBothAimButtonsHeld],
            ..ROW
        },
        Row {
            id: RowId::BackAndBothAimButtonsHeld,
            needs: &[Held(BACK), Held(AIM_UP), Held(AIM_DOWN)],
            next: TURN,
            effects: &[TurnRound],
            ..ROW
        },
        Row {
            id: RowId::BackAndShootHeld,
            needs: &[Held(BACK), Held(SHOOT)],
            next: Next {
                facing: NextFacing::Back,
                ..MOONWALK
            },
            ..ROW
        },
        Row {
            id: RowId::ForwardAndAnAimButtonHeld,
            needs: &[Held(FORWARD), Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..RUN
            },
            beats: &[RowId::ForwardWithUpOrDownHeld],
            ..ROW
        },
        Row {
            id: RowId::ForwardWithUpOrDownHeld,
            needs: &[Held(FORWARD), Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..RUN
            },
            beats: &[RowId::BackAndShootHeld],
            ..ROW
        },
        Row {
            id: RowId::BothAimButtonsHeld,
            needs: &[Held(AIM_UP), Held(AIM_DOWN)],
            next: Next {
                aim: UP_AIM,
                ..STILL
            },
            beats: &[
                RowId::BackAndShootHeld,
                RowId::ForwardAndAnAimButtonHeld,
                RowId::ForwardWithUpOrDownHeld,
            ],
            ..ROW
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            next: RUN,
            ..ROW
        },
        Row {
            id: RowId::UpHeld,
            needs: &[Held(UP)],
            next: Next {
                aim: UP_AIM,
                ..STILL
            },
            beats: &[RowId::AimUpOrAimDownHeld],
            ..ROW
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
            needs: &[Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..STILL
            },
            ..ROW
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Held(BACK)],
            next: TURN,
            effects: &[TurnRound],
            beats: &[RowId::UpHeld, RowId::AimUpOrAimDownHeld, RowId::ForwardHeld],
            ..ROW
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
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
            id: RowId::DownPressed,
            needs: &[Pressed(DOWN)],
            next: CROUCH,
            beats: &[RowId::JumpPressed],
            ..ROW
        },
        Row {
            id: RowId::JumpPressed,
            needs: &[Pressed(key::JUMP)],
            next: SPIN,
            effects: &[Rise],
            ..ROW
        },
        Row {
            id: RowId::ForwardAndShootHeld,
            needs: &[Held(FORWARD), Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..RUN
            },
            ..ROW
        },
        Row {
            id: RowId::ForwardAndAnAimButtonHeld,
            needs: &[Held(FORWARD), Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..RUN
            },
            beats: &[RowId::ForwardWithUpOrDownHeld, RowId::ForwardAndShootHeld],
            ..ROW
        },
        Row {
            id: RowId::ForwardWithUpOrDownHeld,
            needs: &[Held(FORWARD), Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..RUN
            },
            beats: &[RowId::ForwardAndShootHeld],
            ..ROW
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            next: Next { gun: None, ..RUN },
            beats: &[RowId::BackHeld],
            ..ROW
        },
        Row {
            id: RowId::UpHeld,
            needs: &[Held(UP)],
            next: Next {
                aim: UP_AIM,
                ..STILL
            },
            beats: &[RowId::AimUpOrAimDownHeld],
            ..ROW
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
            needs: &[Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..STILL
            },
            ..ROW
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Held(BACK)],
            next: TURN,
            effects: &[TurnRound],
            beats: &[RowId::UpHeld, RowId::AimUpOrAimDownHeld],
            ..ROW
        },
        // The chart's last row for running gives two outcomes, by her horizontal speed at the
        // frame's start; a row has one next pose, so the cell is two rows. This one, kept while
        // the speed isn't 0, names no button: every other fitting row outranks it, so it is
        // used only where the last row would be, and its kept pose brings the step.
        Row {
            id: RowId::AnythingElseMoving,
            needs: &[Test::Moving],
            effects: &[AsRunning],
            steps: true,
            ..ROW
        },
    ],
    last: Row {
        id: RowId::AnythingElseAtRest,
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
            id: RowId::JumpAndForwardHeld,
            needs: &[Held(key::JUMP), Held(FORWARD)],
            next: Next {
                legs: Some(Legs::Striding),
                ..JUMP
            },
            ..ROW
        },
        Row {
            id: RowId::JumpWithUpDownOrAnAimButtonHeld,
            needs: &[Held(key::JUMP), Held(UDA)],
            next: Next {
                aim: UP_DOWN_AIM,
                ..JUMP
            },
            beats: &[RowId::JumpAndForwardHeld],
            ..ROW
        },
        Row {
            id: RowId::JumpAndBackHeld,
            needs: &[Held(key::JUMP), Held(BACK)],
            next: TURN_IN_A_JUMP,
            effects: &[TurnRound],
            beats: &[
                RowId::JumpWithUpDownOrAnAimButtonHeld,
                RowId::JumpAndForwardHeld,
            ],
            ..ROW
        },
        Row {
            id: RowId::ShootHeld,
            needs: &[Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..JUMP
            },
            ..ROW
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
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
            id: RowId::AimingDownDownPressed,
            needs: &[Aimed(&[Aim::Down]), Pressed(DOWN)],
            next: MORPH,
            ..ROW
        },
        Row {
            id: RowId::AimingDownFacingRightForwardJumpAndShootHeld,
            needs: &[
                Aimed(&[Aim::Down]),
                Faces(Side::Right),
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
            id: RowId::ForwardWithUpDownOrAnAimButtonHeld,
            needs: &[Held(FORWARD), Held(UDA)],
            next: Next {
                aim: AIR_AIM,
                ..JUMP
            },
            beats: &[RowId::AimingDownFacingRightForwardJumpAndShootHeld],
            ..ROW
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            next: Next {
                legs: Some(Legs::Striding),
                ..JUMP
            },
            beats: &[RowId::ShootHeld, RowId::GunStowedJumpHeld],
            ..ROW
        },
        Row {
            id: RowId::ShootHeld,
            needs: &[Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..JUMP
            },
            beats: &[RowId::GunStowedJumpHeld],
            ..ROW
        },
        Row {
            id: RowId::GunStowedJumpHeld,
            needs: &[Test::Gun(Gun::Stowed), Held(key::JUMP)],
            next: JUMP,
            ..ROW
        },
        Row {
            id: RowId::UpDownOrAnAimButtonHeld,
            needs: &[Held(UDA)],
            next: Next {
                aim: AIR_AIM,
                ..JUMP
            },
            beats: &[
                RowId::ForwardHeld,
                RowId::ShootHeld,
                RowId::GunStowedJumpHeld,
            ],
            ..ROW
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Held(BACK)],
            next: TURN_IN_A_JUMP,
            effects: &[TurnRound],
            beats: &[
                RowId::ForwardHeld,
                RowId::ShootHeld,
                RowId::GunStowedJumpHeld,
                RowId::UpDownOrAnAimButtonHeld,
                RowId::AimingDownJumpHeldAlone,
            ],
            // PadChart, left and right both held: back held beats forward with an aim button
            // held, unless jump is held too (with up or down held instead, forward wins by rank).
            beats_unless: &[Beat {
                row: RowId::ForwardWithUpDownOrAnAimButtonHeld,
                unless: &[&[Not(&Held(AIM))], &[Held(key::JUMP)]],
            }],
            ..ROW
        },
        Row {
            id: RowId::AimingDownJumpHeldAlone,
            needs: &[Aimed(&[Aim::Down]), Held(key::JUMP), Not(&Held(NAMED))],
            beats: &[RowId::GunStowedJumpHeld],
            ..ROW
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
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
            id: RowId::ShootPressed,
            needs: &[Pressed(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..JUMP
            },
            ..ROW
        },
        Row {
            id: RowId::ShootWithUpDownOrAnAimButtonHeld,
            needs: &[Held(SHOOT), Held(UDA)],
            next: Next {
                aim: UP_DOWN_AIM,
                ..JUMP
            },
            beats: &[RowId::ForwardAndJumpHeld],
            ..ROW
        },
        Row {
            id: RowId::ForwardAndJumpHeld,
            needs: &[Held(FORWARD), Held(key::JUMP)],
            ..ROW
        },
        Row {
            id: RowId::UpDownOrAnAimButtonHeld,
            needs: &[Held(UDA)],
            next: Next {
                aim: SPIN_AIM,
                ..JUMP
            },
            beats: &[RowId::ForwardHeld, RowId::BackHeld],
            ..ROW
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Held(BACK)],
            next: Next {
                facing: NextFacing::Back,
                ..SPIN
            },
            effects: &[TurnRound],
            ..ROW
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            beats: &[RowId::BackHeld],
            ..ROW
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        ..ROW
    },
};

/// Falling, upright.
static FALLING: Situation = Situation {
    doing: Doing::Falling,
    moves: Moves::Build(Gait::Falling, Drift::Air),
    rows: &[
        Row {
            id: RowId::AimingDownDownPressed,
            needs: &[Aimed(&[Aim::Down]), Pressed(DOWN)],
            next: MORPH,
            ..ROW
        },
        Row {
            id: RowId::ForwardWithUpOrDownHeld,
            needs: &[Held(FORWARD), Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..FALL
            },
            ..ROW
        },
        Row {
            id: RowId::ShootHeld,
            needs: &[Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..FALL
            },
            beats: &[RowId::ForwardHeld],
            ..ROW
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            next: Next { gun: None, ..FALL },
            ..ROW
        },
        Row {
            id: RowId::UpDownOrAnAimButtonHeld,
            needs: &[Held(UDA)],
            next: Next {
                aim: AIR_AIM,
                ..FALL
            },
            beats: &[RowId::ShootHeld, RowId::ForwardHeld],
            ..ROW
        },
        // "Aiming Down or with the gun drawn, up or down held beats it": the clause's condition,
        // under which the row it beats beats it instead.
        Row {
            id: RowId::BackHeld,
            needs: &[Held(BACK)],
            next: TURN_IN_A_FALL,
            effects: &[TurnRound],
            beats: &[RowId::ShootHeld, RowId::ForwardHeld],
            beats_unless: &[Beat {
                row: RowId::UpDownOrAnAimButtonHeld,
                unless: &[
                    &[Aimed(&[Aim::Down]), Held(UP_DOWN)],
                    &[Test::Gun(Gun::Drawn), Held(UP_DOWN)],
                ],
            }],
            ..ROW
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
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
            id: RowId::DownPressed,
            needs: &[Pressed(DOWN)],
            next: MORPH,
            beats: &[RowId::JumpPressed],
            ..ROW
        },
        Row {
            id: RowId::UpPressed,
            needs: &[Pressed(UP)],
            next: Next {
                action: Some(StandingUp),
                aim: CROUCH_AIM,
                lower: GROWS,
                ..STILL
            },
            beats: &[RowId::DownPressed, RowId::JumpPressed, RowId::BackPressed],
            ..ROW
        },
        Row {
            id: RowId::JumpPressed,
            needs: &[Pressed(key::JUMP)],
            next: TAKE_OFF,
            effects: &[Rise, Lift],
            ..ROW
        },
        Row {
            id: RowId::BackPressed,
            needs: &[Pressed(BACK)],
            next: Next {
                body: Some(Crouched),
                ..TURN
            },
            beats: &[RowId::DownPressed, RowId::JumpPressed],
            ..ROW
        },
        Row {
            id: RowId::ForwardAndAnAimButtonHeld,
            needs: &[Held(FORWARD), Held(AIM)],
            next: STILL,
            ..ROW
        },
        Row {
            id: RowId::BothAimButtonsHeld,
            needs: &[Held(AIM_UP), Held(AIM_DOWN)],
            next: Next {
                body: Some(Crouched),
                aim: UP_AIM,
                ..STILL
            },
            beats: &[RowId::ForwardAndAnAimButtonHeld],
            ..ROW
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            next: STILL,
            ..ROW
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
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
        id: RowId::AnythingElse,
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
    id: RowId::UpPressed,
    needs: &[Pressed(UP)],
    next: UNMORPH,
    beats: &[RowId::JumpPressedWithoutTheSpringBall],
    ..ROW
};
/// A ball's "without the spring ball jump pressed" row.
const UNMORPH_JUMP: Row = Row {
    id: RowId::JumpPressedWithoutTheSpringBall,
    needs: &[Pressed(key::JUMP), Test::Body(&[Ball])],
    next: UNMORPH,
    ..ROW
};

/// A ball on the ground: the spring ball's hop, unmorphing, and the side of the pad held, which
/// it faces, rolling. Left and right both held (`PadChart`), it rolls rightward.
const BALL_ON_GROUND: &[Row] = &[
    Row {
        id: RowId::JumpPressedWithTheSpringBall,
        needs: &[Pressed(key::JUMP), Test::Body(&[SpringBall])],
        next: Next { body: None, ..JUMP },
        effects: &[Rise],
        ..ROW
    },
    Row {
        beats: &[
            RowId::JumpPressedWithTheSpringBall,
            RowId::JumpPressedWithoutTheSpringBall,
        ],
        ..UNMORPH_ROW
    },
    UNMORPH_JUMP,
    Row {
        id: RowId::RightHeld,
        needs: &[Held(RIGHT)],
        next: Next {
            facing: NextFacing::To(Facing::Right),
            ..ROLL
        },
        effects: &[TurnRound],
        beats: &[RowId::LeftHeld],
        ..ROW
    },
    Row {
        id: RowId::LeftHeld,
        needs: &[Held(LEFT)],
        next: Next {
            facing: NextFacing::To(Facing::Left),
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
        id: RowId::RightHeld,
        needs: &[Held(RIGHT)],
        next: Next {
            facing: NextFacing::To(Facing::Right),
            ..KEPT
        },
        effects: &[TurnRound],
        beats_unless: &[Beat {
            row: RowId::LeftHeld,
            unless: &[
                &[Test::Body(&[Ball]), Faces(Side::Left)],
                &[Test::Body(&[SpringBall]), Faces(Side::Right)],
            ],
        }],
        ..ROW
    },
    Row {
        id: RowId::LeftHeld,
        needs: &[Held(LEFT)],
        next: Next {
            facing: NextFacing::To(Facing::Left),
            ..KEPT
        },
        effects: &[TurnRound],
        ..ROW
    },
    Row {
        id: RowId::OffTheGroundWithTheSpringBall,
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
        id: RowId::RightHeld,
        needs: &[Held(RIGHT)],
        next: Next {
            facing: NextFacing::To(Facing::Right),
            ..KEPT
        },
        effects: &[TurnRound],
        beats: &[RowId::LeftHeld],
        ..ROW
    },
    Row {
        id: RowId::LeftHeld,
        needs: &[Held(LEFT)],
        next: Next {
            facing: NextFacing::To(Facing::Left),
            ..KEPT
        },
        effects: &[TurnRound],
        ..ROW
    },
];

/// The ball's last row on the ground: ball `Still`, halted.
const BALL_STILL: Row = Row {
    id: RowId::AnythingElseOnTheGround,
    next: Next {
        body: None,
        ..STILL
    },
    effects: &[Halt],
    ..ROW
};

/// The ball's last row in the air, without the spring ball: kept, as running.
const BALL_KEPT: Row = Row {
    id: RowId::InTheAirWithoutTheSpringBall,
    effects: &[AsRunning],
    ..ROW
};

/// The spring ball's last row off the ground: kept, halted.
const SPRING_BALL_KEPT: Row = Row {
    id: RowId::OffTheGroundWithTheSpringBall,
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
            id: RowId::AimedUpOrDownForwardForwardHeldAndJumpPressed,
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
            id: RowId::JumpPressed,
            needs: &[Pressed(key::JUMP)],
            next: TAKE_OFF,
            effects: &[Rise],
            ..ROW
        },
        Row {
            id: RowId::AimedForwardForwardAndJumpHeld,
            needs: &[Aimed(&[Aim::Forward]), Held(FORWARD), Held(key::JUMP)],
            next: SPIN,
            effects: &[Rise],
            beats: &[RowId::JumpPressed],
            ..ROW
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            ..ROW
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
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
        id: RowId::AnyButtonHeld,
        needs: &[Held(ANY_BUTTON)],
        ..ROW
    }],
    last: Row {
        id: RowId::NothingHeld,
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
        id: RowId::AnythingOrNothingHeld,
        ..ROW
    },
};

/// Moonwalking: forward is the way she walks, which she faces; back the way she looks.
static MOONWALKING: Situation = Situation {
    doing: Doing::Moonwalking,
    moves: Moves::Build(Gait::Moonwalking, Drift::Ground),
    rows: &[
        Row {
            id: RowId::DownPressed,
            needs: &[Pressed(DOWN)],
            next: Next {
                facing: NextFacing::Back,
                ..CROUCH
            },
            beats: &[RowId::JumpPressed],
            ..ROW
        },
        // The moonwalk's jump: the one turn that keeps her facing, the way she walks.
        Row {
            id: RowId::JumpPressed,
            needs: &[Pressed(key::JUMP)],
            next: Next {
                action: Some(Turning(Turn::MoonwalkJump)),
                facing: NextFacing::Kept,
                ..TURN
            },
            effects: &[TurnRound],
            ..ROW
        },
        Row {
            id: RowId::WalksShootAndAnAimButtonHeld,
            needs: &[Held(FORWARD), Held(SHOOT), Held(AIM)],
            next: Next {
                aim: MOON_AIM,
                ..MOONWALK
            },
            ..ROW
        },
        Row {
            id: RowId::WalksAndShootHeld,
            needs: &[Held(FORWARD), Held(SHOOT)],
            next: MOONWALK,
            ..ROW
        },
        Row {
            id: RowId::LooksHeld,
            needs: &[Held(BACK)],
            next: Next {
                facing: NextFacing::Back,
                ..RUN
            },
            beats: &[RowId::WalksHeld],
            ..ROW
        },
        Row {
            id: RowId::WalksHeld,
            needs: &[Held(FORWARD)],
            next: TURN,
            effects: &[TurnRound],
            ..ROW
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        next: Next {
            facing: NextFacing::Back,
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
            id: RowId::DownPressed,
            needs: &[Pressed(DOWN)],
            next: MORPH,
            ..ROW
        },
        Row {
            id: RowId::ShootHeld,
            needs: &[Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..JUMP
            },
            beats: &[RowId::JumpHeld],
            ..ROW
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
            needs: &[Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..JUMP
            },
            beats: &[RowId::ShootHeld, RowId::JumpHeld],
            ..ROW
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Held(BACK)],
            next: Next {
                facing: NextFacing::Back,
                ..SPIN
            },
            effects: &[TurnRound],
            beats: &[RowId::ShootHeld, RowId::AimUpOrAimDownHeld, RowId::JumpHeld],
            ..ROW
        },
        Row {
            id: RowId::JumpHeld,
            needs: &[Held(key::JUMP)],
            ..ROW
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
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
            id: RowId::DownPressed,
            needs: &[Pressed(DOWN)],
            next: CROUCH,
            ..ROW
        },
        Row {
            id: RowId::JumpPressed,
            needs: &[Pressed(key::JUMP)],
            next: TAKE_OFF,
            effects: &[Rise],
            beats: &[RowId::DownPressed],
            ..ROW
        },
        Row {
            id: RowId::BackAndAnAimButtonHeld,
            needs: &[Held(BACK), Held(AIM)],
            next: Next {
                facing: NextFacing::Back,
                aim: AIM_DOWN_ELSE_UP,
                ..MOONWALK
            },
            ..ROW
        },
        Row {
            id: RowId::ForwardWithUpOrDownHeld,
            needs: &[Held(FORWARD), Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..RUN
            },
            beats: &[RowId::DownPressed, RowId::BackAndAnAimButtonHeld],
            ..ROW
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            next: RUN,
            ..ROW
        },
        Row {
            id: RowId::UpHeld,
            needs: &[Held(UP)],
            next: Next {
                aim: UP_AIM,
                ..STILL
            },
            beats: &[RowId::AimUpOrAimDownHeld, RowId::BackHeld],
            ..ROW
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
            needs: &[Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..STILL
            },
            beats: &[RowId::ForwardHeld, RowId::BackHeld],
            ..ROW
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Held(BACK)],
            next: TURN,
            effects: &[TurnRound],
            beats: &[RowId::ForwardHeld],
            ..ROW
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        next: Next {
            aim: FORWARD_AIM,
            ..KEPT
        },
        effects: &[EndSkid],
        ..ROW
    },
};
