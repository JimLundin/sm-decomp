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
    AsRunning, EndSkid, EndsTheBounce, Halt, Launch, LiftIfUnaimed, LoseDash, Rise, SpinOn,
    TurnRound, TurnRoundIfFacingChanges,
};
use crate::rows::{
    AIM, AIM_UP_ELSE_DOWN, Aiming, BACK, Beat, Class, Doing, Drift, Entering, FORWARD, FORWARD_AIM,
    Footing, KEEP_POSE, LOWERS_NOTHING, Moves, NO_CHANGE, Next, NextFacing, Row, RowId, Situation,
    TO_CROUCHING, TO_FALLING, TO_JUMPING, TO_MOONWALKING, TO_MORPHING, TO_RUNNING, TO_SPINNING,
    TO_STILL, TO_TAKING_OFF, TO_TURNING, Test, UDA, UP_AIM, UP_DOWN, Was,
};
use crate::rules::Gait;
use Test::{Aimed, Faces, Held, Not, Pressed};
use clean_types::Action::{
    AgainstWall, Crouching, Falling, Jumping, Landing, Moonwalking, Morphing, Rolling, Running,
    Spinning, StandingUp, Still, TakingOff, Turning, Unmorphing, WallJumping,
};
use clean_types::Body::{Ball, Crouched, SpringBall, Upright};
use clean_types::button::{self as key, AIM_DOWN, AIM_UP, DOWN, LEFT, RIGHT, SHOOT, UP};
use clean_types::{
    Action, Aim, Body, Buttons, Facing, Gun, Item, LandedFrom, Legs, Pose, Side, Turn,
};

/// A pose no situation names (no class of `CLASSES` holds it): the step gives her no motion
/// and reports it, and the chart gives it no next pose. A caller that builds her pose decides
/// what to do with one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UnknownPose(pub Pose);

/// `MovesAsShePosed`: her situation, from her pose at the frame's start: the first class of
/// `CLASSES` her pose is in.
///
/// # Errors
///
/// `UnknownPose`, where no class holds her pose.
pub fn classify(pose: Pose) -> Result<&'static Situation, UnknownPose> {
    let fits = |c: &&Class| {
        c.body.contains(&pose.body) && c.action.contains(&pose.action) && c.aim.contains(&pose.aim)
    };
    CLASSES
        .iter()
        .find(fits)
        .map(|c| c.is)
        .ok_or(UnknownPose(pose))
}

/// Every aim.
const AIMS: &[Aim] = &[
    Aim::Forward,
    Aim::Up,
    Aim::UpForward,
    Aim::DownForward,
    Aim::Down,
];
/// The ball, with the spring ball or without.
const BALLS: &[Body] = &[Ball, SpringBall];
/// The turns on the ground: a ground turn and the moonwalk's jump.
const GROUND_TURNS: &[Action] = &[Turning(Turn::Ground), Turning(Turn::MoonwalkJump)];
/// The change of action's own effects, whatever brought the change.
///
/// `Jumping`: starting a straight jump (upright) or a fall, from any action but that one, with
/// a dash she carries on along her facing, else she follows the pad. `WallJump` (§1.2): a spin
/// that followed a wall jump is carried on. `ShrinkingLowersHer`: unmorphing, the one change of
/// shape a ball starts, ends a bounce she is in.
pub static ENTERING: &[Entering] = &[
    Entering {
        into: &[(Crouched, Unmorphing)],
        from: Was::AnyBut(&[Unmorphing]),
        effect: EndsTheBounce,
    },
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

/// A class's defaults: upright, at any aim; each class names its actions and its situation.
const UPRIGHT_AT_ANY_AIM: Class = Class {
    body: &[Upright],
    action: &[],
    aim: AIMS,
    is: &STANDING,
};

/// The classes, in order: the first her pose is in gives her situation. Body follows the
/// action (§7), so each class names the bodies its actions come in, and a pose of no class
/// (upright `Rolling`, a crouched `Running`) is none of hers.
pub static CLASSES: &[Class] = &[
    Class {
        body: &[Crouched],
        action: &[Crouching, Unmorphing],
        is: &SHAPING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[StandingUp],
        is: &SHAPING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        body: BALLS,
        action: &[Morphing],
        is: &SHAPING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        body: BALLS,
        action: &[Jumping],
        is: &HOP,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        body: BALLS,
        action: &[Rolling],
        is: &ROLLING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        body: BALLS,
        action: &[Still],
        is: &RESTING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        body: BALLS,
        action: &[Falling],
        is: &BALL_IN_AIR,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        body: &[Crouched],
        action: &[Turning(Turn::Ground)],
        is: &TURNING_ON,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Turning(Turn::Jump), Turning(Turn::Fall)],
        is: &TURNING_ON,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: GROUND_TURNS,
        aim: &[Aim::UpForward],
        is: &TURNING_ON,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: GROUND_TURNS,
        is: &TURNING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        body: &[Crouched],
        action: &[Still],
        is: &CROUCHED,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Running],
        is: &RUNNING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[TakingOff],
        is: &TAKING_OFF,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Jumping],
        is: &STRAIGHT_JUMP,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Spinning],
        is: &SPINNING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Falling],
        is: &FALLING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Moonwalking],
        is: &MOONWALKING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[WallJumping],
        is: &WALL_JUMPING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[AgainstWall],
        is: &AGAINST_WALL,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Still, Landing(LandedFrom::Jump), Landing(LandedFrom::Spin)],
        is: &STANDING,
        ..UPRIGHT_AT_ANY_AIM
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
const TO_TURNING_IN_A_JUMP: Next = Next {
    action: Some(Turning(Turn::Jump)),
    aim: TURN_IN_THE_AIR,
    ..TO_TURNING
};
/// `Turning` in a fall.
const TO_TURNING_IN_A_FALL: Next = Next {
    action: Some(Turning(Turn::Fall)),
    aim: TURN_IN_THE_AIR,
    ..TO_TURNING
};
/// Crouched `Unmorphing`, which lowers her 0 px (`ShrinkingLowersHer`).
const TO_UNMORPHING: Next = Next {
    body: Some(Crouched),
    action: Some(Unmorphing),
    lower: LOWERS_NOTHING,
    ..TO_STILL
};
/// The same ball, rolling.
const TO_ROLLING: Next = Next {
    body: None,
    action: Some(Rolling),
    ..TO_STILL
};

/// Any button.
const ANY_BUTTON: Buttons = Buttons(u16::MAX);
/// Every button a straight jump's row names but jump.
const NAMED: Buttons = Buttons(FORWARD.0 | BACK.0 | SHOOT.0 | UDA.0);

/// Standing still, or landing.
static STANDING: Situation = Situation {
    doing: Doing::Standing,
    moves: Moves::HaltsAfterDash,
    footing: Footing::OnHerFeet,
    rows: &[
        Row {
            id: RowId::DownPressed,
            needs: &[Pressed(DOWN)],
            next: Next {
                aim: CROUCH_AIM,
                ..TO_CROUCHING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpPressed,
            needs: &[Pressed(key::JUMP)],
            next: Next {
                aim: TAKE_OFF_AIM,
                ..TO_TAKING_OFF
            },
            effects: &[Rise],
            beats: &[RowId::DownPressed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackShootAndAnAimButtonHeld,
            needs: &[Held(BACK), Held(SHOOT), Held(AIM)],
            next: Next {
                facing: NextFacing::Back,
                aim: AIM_DOWN_ELSE_UP,
                ..TO_MOONWALKING
            },
            beats: &[RowId::BackAndBothAimButtonsHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackAndBothAimButtonsHeld,
            needs: &[Held(BACK), Held(AIM_UP), Held(AIM_DOWN)],
            next: TO_TURNING,
            effects: &[TurnRound],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackAndShootHeld,
            needs: &[Held(BACK), Held(SHOOT)],
            next: Next {
                facing: NextFacing::Back,
                ..TO_MOONWALKING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardAndAnAimButtonHeld,
            needs: &[Held(FORWARD), Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..TO_RUNNING
            },
            beats: &[RowId::ForwardWithUpOrDownHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardWithUpOrDownHeld,
            needs: &[Held(FORWARD), Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..TO_RUNNING
            },
            beats: &[RowId::BackAndShootHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BothAimButtonsHeld,
            needs: &[Held(AIM_UP), Held(AIM_DOWN)],
            next: Next {
                aim: UP_AIM,
                ..TO_STILL
            },
            beats: &[
                RowId::BackAndShootHeld,
                RowId::ForwardAndAnAimButtonHeld,
                RowId::ForwardWithUpOrDownHeld,
            ],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            next: TO_RUNNING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::UpHeld,
            needs: &[Held(UP)],
            next: Next {
                aim: UP_AIM,
                ..TO_STILL
            },
            beats: &[RowId::AimUpOrAimDownHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
            needs: &[Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..TO_STILL
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Held(BACK)],
            next: TO_TURNING,
            effects: &[TurnRound],
            beats: &[RowId::UpHeld, RowId::AimUpOrAimDownHeld, RowId::ForwardHeld],
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        next: Next {
            aim: STANDS_FORWARD,
            ..KEEP_POSE
        },
        effects: &[EndSkid],
        ..NO_CHANGE
    },
};

/// Running.
static RUNNING: Situation = Situation {
    doing: Doing::Running,
    moves: Moves::Build(Gait::Running, Drift::Ground),
    footing: Footing::OnHerFeet,
    rows: &[
        Row {
            id: RowId::DownPressed,
            needs: &[Pressed(DOWN)],
            next: TO_CROUCHING,
            beats: &[RowId::JumpPressed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpPressed,
            needs: &[Pressed(key::JUMP)],
            next: TO_SPINNING,
            effects: &[Rise],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardAndShootHeld,
            needs: &[Held(FORWARD), Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..TO_RUNNING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardAndAnAimButtonHeld,
            needs: &[Held(FORWARD), Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..TO_RUNNING
            },
            beats: &[RowId::ForwardWithUpOrDownHeld, RowId::ForwardAndShootHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardWithUpOrDownHeld,
            needs: &[Held(FORWARD), Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..TO_RUNNING
            },
            beats: &[RowId::ForwardAndShootHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            next: Next {
                gun: None,
                ..TO_RUNNING
            },
            beats: &[RowId::BackHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::UpHeld,
            needs: &[Held(UP)],
            next: Next {
                aim: UP_AIM,
                ..TO_STILL
            },
            beats: &[RowId::AimUpOrAimDownHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
            needs: &[Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..TO_STILL
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Held(BACK)],
            next: TO_TURNING,
            effects: &[TurnRound],
            beats: &[RowId::UpHeld, RowId::AimUpOrAimDownHeld],
            ..NO_CHANGE
        },
        // The chart's last row for running gives two outcomes, by her horizontal speed at the
        // frame's start; a row has one next pose, so the cell is two rows. This one, kept while
        // the speed isn't 0, names no button: every other fitting row outranks it, so it is
        // used only where the last row would be, and its kept pose brings the step.
        Row {
            id: RowId::AnythingElseMoving,
            needs: &[Test::Moving],
            effects: &[AsRunning],
            aim_step: true,
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElseAtRest,
        next: TO_STILL,
        effects: &[AsRunning],
        ..NO_CHANGE
    },
};

/// Taking off.
static TAKING_OFF: Situation = Situation {
    doing: Doing::TakingOff,
    moves: Moves::TakeOff,
    footing: Footing::OnHerFeet,
    rows: &[
        Row {
            id: RowId::JumpAndForwardHeld,
            needs: &[Held(key::JUMP), Held(FORWARD)],
            next: Next {
                legs: Some(Legs::Striding),
                ..TO_JUMPING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpWithUpDownOrAnAimButtonHeld,
            needs: &[Held(key::JUMP), Held(UDA)],
            next: Next {
                aim: UP_DOWN_AIM,
                ..TO_JUMPING
            },
            beats: &[RowId::JumpAndForwardHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpAndBackHeld,
            needs: &[Held(key::JUMP), Held(BACK)],
            next: TO_TURNING_IN_A_JUMP,
            effects: &[TurnRound],
            beats: &[
                RowId::JumpWithUpDownOrAnAimButtonHeld,
                RowId::JumpAndForwardHeld,
            ],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ShootHeld,
            needs: &[Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..TO_JUMPING
            },
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        effects: &[AsRunning],
        ..NO_CHANGE
    },
};

/// In a straight jump.
static STRAIGHT_JUMP: Situation = Situation {
    doing: Doing::StraightJump,
    moves: Moves::Build(Gait::StraightJump, Drift::Air),
    footing: Footing::OnHerFeet,
    rows: &[
        Row {
            id: RowId::AimingDownDownPressed,
            needs: &[Aimed(&[Aim::Down]), Pressed(DOWN)],
            next: TO_MORPHING,
            ..NO_CHANGE
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
                ..TO_JUMPING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardWithUpDownOrAnAimButtonHeld,
            needs: &[Held(FORWARD), Held(UDA)],
            next: Next {
                aim: AIR_AIM,
                ..TO_JUMPING
            },
            beats: &[RowId::AimingDownFacingRightForwardJumpAndShootHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            next: Next {
                legs: Some(Legs::Striding),
                ..TO_JUMPING
            },
            beats: &[RowId::ShootHeld, RowId::GunStowedJumpHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ShootHeld,
            needs: &[Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..TO_JUMPING
            },
            beats: &[RowId::GunStowedJumpHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::GunStowedJumpHeld,
            needs: &[Test::Gun(Gun::Stowed), Held(key::JUMP)],
            next: TO_JUMPING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::UpDownOrAnAimButtonHeld,
            needs: &[Held(UDA)],
            next: Next {
                aim: AIR_AIM,
                ..TO_JUMPING
            },
            beats: &[
                RowId::ForwardHeld,
                RowId::ShootHeld,
                RowId::GunStowedJumpHeld,
            ],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Held(BACK)],
            next: TO_TURNING_IN_A_JUMP,
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
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimingDownJumpHeldAlone,
            needs: &[Aimed(&[Aim::Down]), Held(key::JUMP), Not(&Held(NAMED))],
            beats: &[RowId::GunStowedJumpHeld],
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        effects: &[AsRunning],
        ..NO_CHANGE
    },
};

/// Spinning.
static SPINNING: Situation = Situation {
    doing: Doing::Spinning,
    moves: Moves::Build(Gait::SpinJump, Drift::Spin),
    footing: Footing::OnHerFeet,
    rows: &[
        Row {
            id: RowId::ShootPressed,
            needs: &[Pressed(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..TO_JUMPING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::ShootWithUpDownOrAnAimButtonHeld,
            needs: &[Held(SHOOT), Held(UDA)],
            next: Next {
                aim: UP_DOWN_AIM,
                ..TO_JUMPING
            },
            beats: &[RowId::ForwardAndJumpHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardAndJumpHeld,
            needs: &[Held(FORWARD), Held(key::JUMP)],
            ..NO_CHANGE
        },
        Row {
            id: RowId::UpDownOrAnAimButtonHeld,
            needs: &[Held(UDA)],
            next: Next {
                aim: SPIN_AIM,
                ..TO_JUMPING
            },
            beats: &[RowId::ForwardHeld, RowId::BackHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Held(BACK)],
            next: Next {
                facing: NextFacing::Back,
                ..TO_SPINNING
            },
            effects: &[TurnRoundIfFacingChanges],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            beats: &[RowId::BackHeld],
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        ..NO_CHANGE
    },
};

/// Falling, upright.
static FALLING: Situation = Situation {
    doing: Doing::Falling,
    moves: Moves::Build(Gait::Falling, Drift::Air),
    footing: Footing::OnHerFeet,
    rows: &[
        Row {
            id: RowId::AimingDownDownPressed,
            needs: &[Aimed(&[Aim::Down]), Pressed(DOWN)],
            next: TO_MORPHING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardWithUpOrDownHeld,
            needs: &[Held(FORWARD), Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..TO_FALLING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::ShootHeld,
            needs: &[Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..TO_FALLING
            },
            beats: &[RowId::ForwardHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            next: Next {
                gun: None,
                ..TO_FALLING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::UpDownOrAnAimButtonHeld,
            needs: &[Held(UDA)],
            next: Next {
                aim: AIR_AIM,
                ..TO_FALLING
            },
            beats: &[RowId::ShootHeld, RowId::ForwardHeld],
            ..NO_CHANGE
        },
        // "Aiming Down or with the gun drawn, up or down held beats it": the clause's condition,
        // under which the row it beats beats it instead.
        Row {
            id: RowId::BackHeld,
            needs: &[Held(BACK)],
            next: TO_TURNING_IN_A_FALL,
            effects: &[TurnRound],
            beats: &[RowId::ShootHeld, RowId::ForwardHeld],
            beats_unless: &[Beat {
                row: RowId::UpDownOrAnAimButtonHeld,
                unless: &[
                    &[Aimed(&[Aim::Down]), Held(UP_DOWN)],
                    &[Test::Gun(Gun::Drawn), Held(UP_DOWN)],
                ],
            }],
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        next: Next {
            aim: FORWARD_AIM,
            ..KEEP_POSE
        },
        effects: &[LoseDash],
        ..NO_CHANGE
    },
};

/// Crouched.
static CROUCHED: Situation = Situation {
    doing: Doing::Crouched,
    moves: Moves::HaltsAfterDash,
    footing: Footing::OnHerFeet,
    rows: &[
        Row {
            id: RowId::DownPressed,
            needs: &[Pressed(DOWN)],
            next: TO_MORPHING,
            beats: &[RowId::JumpPressed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::UpPressed,
            needs: &[Pressed(UP)],
            next: Next {
                action: Some(StandingUp),
                aim: CROUCH_AIM,
                lower: LOWERS_NOTHING,
                ..TO_STILL
            },
            beats: &[RowId::DownPressed, RowId::JumpPressed, RowId::BackPressed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpPressed,
            needs: &[Pressed(key::JUMP)],
            next: TO_TAKING_OFF,
            effects: &[Rise, LiftIfUnaimed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackPressed,
            needs: &[Pressed(BACK)],
            next: Next {
                body: Some(Crouched),
                ..TO_TURNING
            },
            beats: &[RowId::DownPressed, RowId::JumpPressed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardAndAnAimButtonHeld,
            needs: &[Held(FORWARD), Held(AIM)],
            next: TO_STILL,
            ..NO_CHANGE
        },
        Row {
            id: RowId::BothAimButtonsHeld,
            needs: &[Held(AIM_UP), Held(AIM_DOWN)],
            next: Next {
                body: Some(Crouched),
                aim: UP_AIM,
                ..TO_STILL
            },
            beats: &[RowId::ForwardAndAnAimButtonHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            next: TO_STILL,
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
            needs: &[Held(AIM)],
            next: Next {
                body: Some(Crouched),
                aim: AIM_UP_ELSE_DOWN,
                ..TO_STILL
            },
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        next: Next {
            aim: FORWARD_AIM,
            ..KEEP_POSE
        },
        effects: &[EndSkid],
        ..NO_CHANGE
    },
};

/// A ball's "up pressed" row, which beats the jump rows (`PadChart`: "beats the spring ball's
/// hop"; in the air, where there is no hop, the jump without the spring ball).
const UP_PRESSED_UNMORPHS: Row = Row {
    id: RowId::UpPressed,
    needs: &[Pressed(UP)],
    next: TO_UNMORPHING,
    beats: &[RowId::JumpPressedWithoutTheSpringBall],
    ..NO_CHANGE
};
/// A ball's "without the spring ball jump pressed" row.
const JUMP_PRESSED_UNMORPHS: Row = Row {
    id: RowId::JumpPressedWithoutTheSpringBall,
    needs: &[Pressed(key::JUMP), Test::Body(&[Ball])],
    next: TO_UNMORPHING,
    ..NO_CHANGE
};

/// A ball on the ground: the spring ball's hop, unmorphing, and the side of the pad held, which
/// it faces, rolling. Left and right both held (`PadChart`), it rolls rightward.
const BALL_ON_GROUND: &[Row] = &[
    Row {
        id: RowId::JumpPressedWithTheSpringBall,
        needs: &[Pressed(key::JUMP), Test::Body(&[SpringBall])],
        next: Next {
            body: None,
            ..TO_JUMPING
        },
        effects: &[Rise],
        ..NO_CHANGE
    },
    Row {
        beats: &[
            RowId::JumpPressedWithTheSpringBall,
            RowId::JumpPressedWithoutTheSpringBall,
        ],
        ..UP_PRESSED_UNMORPHS
    },
    JUMP_PRESSED_UNMORPHS,
    Row {
        id: RowId::RightHeld,
        needs: &[Held(RIGHT)],
        next: Next {
            facing: NextFacing::To(Facing::Right),
            ..TO_ROLLING
        },
        effects: &[TurnRoundIfFacingChanges],
        beats: &[RowId::LeftHeld],
        ..NO_CHANGE
    },
    Row {
        id: RowId::LeftHeld,
        needs: &[Held(LEFT)],
        next: Next {
            facing: NextFacing::To(Facing::Left),
            ..TO_ROLLING
        },
        effects: &[TurnRoundIfFacingChanges],
        ..NO_CHANGE
    },
];

/// The ball in the air: unmorphing, the side of the pad held, which it faces, and the spring
/// ball's let-go row (the plain ball's is the last). Left and right both held (`PadChart`): the
/// spring ball turns round (the spring ball falling: in the air it is either falling or on its
/// hop, its own situation), and the ball without it keeps its facing; as one clause, right held
/// wins unless the plain ball faces left or the spring ball faces right.
const BALL_IN_THE_AIR: &[Row] = &[
    UP_PRESSED_UNMORPHS,
    JUMP_PRESSED_UNMORPHS,
    Row {
        id: RowId::RightHeld,
        needs: &[Held(RIGHT)],
        next: Next {
            facing: NextFacing::To(Facing::Right),
            ..KEEP_POSE
        },
        effects: &[TurnRoundIfFacingChanges],
        beats_unless: &[Beat {
            row: RowId::LeftHeld,
            unless: &[
                &[Test::Body(&[Ball]), Faces(Side::Left)],
                &[Test::Body(&[SpringBall]), Faces(Side::Right)],
            ],
        }],
        ..NO_CHANGE
    },
    Row {
        id: RowId::LeftHeld,
        needs: &[Held(LEFT)],
        next: Next {
            facing: NextFacing::To(Facing::Left),
            ..KEEP_POSE
        },
        effects: &[TurnRoundIfFacingChanges],
        ..NO_CHANGE
    },
    Row {
        id: RowId::OffTheGroundWithTheSpringBall,
        needs: &[Test::Body(&[SpringBall])],
        effects: &[Halt],
        ..NO_CHANGE
    },
];

/// The spring ball's hop: unmorphing and the side of the pad held, which it faces. Left and
/// right both held (`PadChart`), it turns rightward.
const BALL_ON_THE_HOP: &[Row] = &[
    Row {
        beats: &[],
        ..UP_PRESSED_UNMORPHS
    },
    Row {
        id: RowId::RightHeld,
        needs: &[Held(RIGHT)],
        next: Next {
            facing: NextFacing::To(Facing::Right),
            ..KEEP_POSE
        },
        effects: &[TurnRoundIfFacingChanges],
        beats: &[RowId::LeftHeld],
        ..NO_CHANGE
    },
    Row {
        id: RowId::LeftHeld,
        needs: &[Held(LEFT)],
        next: Next {
            facing: NextFacing::To(Facing::Left),
            ..KEEP_POSE
        },
        effects: &[TurnRoundIfFacingChanges],
        ..NO_CHANGE
    },
];

/// The ball's last row on the ground: ball `Still`, halted.
const BALL_COMES_TO_REST: Row = Row {
    id: RowId::AnythingElseOnTheGround,
    next: Next {
        body: None,
        ..TO_STILL
    },
    effects: &[Halt],
    ..NO_CHANGE
};

/// The ball's last row in the air, without the spring ball: kept, as running.
const BALL_KEEPS_ITS_POSE: Row = Row {
    id: RowId::InTheAirWithoutTheSpringBall,
    effects: &[AsRunning],
    ..NO_CHANGE
};

/// The spring ball's last row off the ground: kept, halted.
const SPRING_BALL_HALTS: Row = Row {
    id: RowId::OffTheGroundWithTheSpringBall,
    effects: &[Halt],
    ..NO_CHANGE
};

/// A ball rolling on the ground.
static ROLLING: Situation = Situation {
    doing: Doing::Ball,
    moves: Moves::Build(Gait::Rolling, Drift::Ground),
    footing: Footing::Ball,
    rows: BALL_ON_GROUND,
    last: BALL_COMES_TO_REST,
};

/// The ball at rest on the ground.
static RESTING: Situation = Situation {
    doing: Doing::Ball,
    moves: Moves::Rest,
    footing: Footing::Ball,
    rows: BALL_ON_GROUND,
    last: BALL_COMES_TO_REST,
};

/// A ball in the air.
static BALL_IN_AIR: Situation = Situation {
    doing: Doing::Ball,
    moves: Moves::Build(Gait::Falling, Drift::BallAir),
    footing: Footing::Ball,
    rows: BALL_IN_THE_AIR,
    last: BALL_KEEPS_ITS_POSE,
};

/// The spring ball's hop.
static HOP: Situation = Situation {
    doing: Doing::Hop,
    moves: Moves::Build(Gait::StraightJump, Drift::BallAir),
    footing: Footing::Ball,
    rows: BALL_ON_THE_HOP,
    last: SPRING_BALL_HALTS,
};

/// Turning on the ground or from the moonwalk's jump, not aimed `UpForward`.
static TURNING: Situation = Situation {
    doing: Doing::Turning,
    moves: Moves::Build(Gait::Turning, Drift::Ground),
    footing: Footing::Turning,
    rows: &[
        Row {
            id: RowId::AimedUpOrDownForwardForwardHeldAndJumpPressed,
            needs: &[
                Aimed(&[Aim::Up, Aim::DownForward]),
                Held(FORWARD),
                Pressed(key::JUMP),
            ],
            next: TO_SPINNING,
            effects: &[Rise],
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpPressed,
            needs: &[Pressed(key::JUMP)],
            next: TO_TAKING_OFF,
            effects: &[Rise],
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimedForwardForwardAndJumpHeld,
            needs: &[Aimed(&[Aim::Forward]), Held(FORWARD), Held(key::JUMP)],
            next: TO_SPINNING,
            effects: &[Rise],
            beats: &[RowId::JumpPressed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        effects: &[EndSkid],
        ..NO_CHANGE
    },
};

/// Turning in a jump or a fall, crouched turning, or turning on the ground or from the
/// moonwalk's jump aimed `UpForward`: she keeps turning.
static TURNING_ON: Situation = Situation {
    doing: Doing::Turning,
    moves: Moves::Build(Gait::Turning, Drift::Ground),
    footing: Footing::Turning,
    rows: &[Row {
        id: RowId::AnyButtonHeld,
        needs: &[Held(ANY_BUTTON)],
        ..NO_CHANGE
    }],
    last: Row {
        id: RowId::NothingHeld,
        effects: &[EndSkid],
        ..NO_CHANGE
    },
};

/// Her body changing shape: the pad has no say until the animation ends.
static SHAPING: Situation = Situation {
    doing: Doing::Shaping,
    moves: Moves::DashKeepingSpeeds,
    footing: Footing::ChangingShape,
    rows: &[],
    last: Row {
        id: RowId::AnythingOrNothingHeld,
        ..NO_CHANGE
    },
};

/// Moonwalking: forward is the way she walks, which she faces; back the way she looks.
static MOONWALKING: Situation = Situation {
    doing: Doing::Moonwalking,
    moves: Moves::Build(Gait::Moonwalking, Drift::Ground),
    footing: Footing::OnHerFeet,
    rows: &[
        Row {
            id: RowId::DownPressed,
            needs: &[Pressed(DOWN)],
            next: Next {
                facing: NextFacing::Back,
                ..TO_CROUCHING
            },
            beats: &[RowId::JumpPressed],
            ..NO_CHANGE
        },
        // The moonwalk's jump: the one turn that keeps her facing, the way she walks.
        Row {
            id: RowId::JumpPressed,
            needs: &[Pressed(key::JUMP)],
            next: Next {
                action: Some(Turning(Turn::MoonwalkJump)),
                facing: NextFacing::Kept,
                ..TO_TURNING
            },
            effects: &[TurnRound],
            ..NO_CHANGE
        },
        Row {
            id: RowId::WalksShootAndAnAimButtonHeld,
            needs: &[Held(FORWARD), Held(SHOOT), Held(AIM)],
            next: Next {
                aim: MOON_AIM,
                ..TO_MOONWALKING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::WalksAndShootHeld,
            needs: &[Held(FORWARD), Held(SHOOT)],
            next: TO_MOONWALKING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::LooksHeld,
            needs: &[Held(BACK)],
            next: Next {
                facing: NextFacing::Back,
                ..TO_RUNNING
            },
            beats: &[RowId::WalksHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::WalksHeld,
            needs: &[Held(FORWARD)],
            next: TO_TURNING,
            effects: &[TurnRound],
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        next: Next {
            facing: NextFacing::Back,
            ..TO_STILL
        },
        effects: &[EndSkid],
        ..NO_CHANGE
    },
};

/// Wall jumping.
static WALL_JUMPING: Situation = Situation {
    doing: Doing::WallJumping,
    moves: Moves::Build(Gait::SpinJump, Drift::WallJump),
    footing: Footing::OnHerFeet,
    rows: &[
        Row {
            id: RowId::DownPressed,
            needs: &[Pressed(DOWN)],
            next: TO_MORPHING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::ShootHeld,
            needs: &[Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..TO_JUMPING
            },
            beats: &[RowId::JumpHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
            needs: &[Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..TO_JUMPING
            },
            beats: &[RowId::ShootHeld, RowId::JumpHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Held(BACK)],
            next: Next {
                facing: NextFacing::Back,
                ..TO_SPINNING
            },
            effects: &[TurnRoundIfFacingChanges],
            beats: &[RowId::ShootHeld, RowId::AimUpOrAimDownHeld, RowId::JumpHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpHeld,
            needs: &[Held(key::JUMP)],
            ..NO_CHANGE
        },
        // The chart's last row for wall jumping, with the screw attack or space jump switched
        // on: its effect doesn't act, so she keeps her speeds and carrying on, and goes on
        // dashing. It names no button: every other fitting row outranks it, so it is used only
        // where the last row would be.
        Row {
            id: RowId::AnythingElseWithTheSpin,
            needs: &[Test::With(&[Item::ScrewAttack, Item::SpaceJump])],
            next: TO_SPINNING,
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        next: TO_SPINNING,
        effects: &[Halt],
        ..NO_CHANGE
    },
};

/// Against the wall.
static AGAINST_WALL: Situation = Situation {
    doing: Doing::AgainstWall,
    moves: Moves::HaltsAfterDash,
    footing: Footing::OnHerFeet,
    rows: &[
        Row {
            id: RowId::DownPressed,
            needs: &[Pressed(DOWN)],
            next: TO_CROUCHING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpPressed,
            needs: &[Pressed(key::JUMP)],
            next: TO_TAKING_OFF,
            effects: &[Rise],
            beats: &[RowId::DownPressed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackAndAnAimButtonHeld,
            needs: &[Held(BACK), Held(AIM)],
            next: Next {
                facing: NextFacing::Back,
                aim: AIM_DOWN_ELSE_UP,
                ..TO_MOONWALKING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardWithUpOrDownHeld,
            needs: &[Held(FORWARD), Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..TO_RUNNING
            },
            beats: &[RowId::DownPressed, RowId::BackAndAnAimButtonHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Held(FORWARD)],
            next: TO_RUNNING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::UpHeld,
            needs: &[Held(UP)],
            next: Next {
                aim: UP_AIM,
                ..TO_STILL
            },
            beats: &[RowId::AimUpOrAimDownHeld, RowId::BackHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
            needs: &[Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..TO_STILL
            },
            beats: &[RowId::ForwardHeld, RowId::BackHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Held(BACK)],
            next: TO_TURNING,
            effects: &[TurnRound],
            beats: &[RowId::ForwardHeld],
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        next: Next {
            aim: FORWARD_AIM,
            ..KEEP_POSE
        },
        effects: &[EndSkid],
        ..NO_CHANGE
    },
};
