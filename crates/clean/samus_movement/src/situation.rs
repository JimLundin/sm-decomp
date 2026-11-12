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

use crate::rows::{
    AIM, AIM_UP_ELSE_DOWN, Aiming, BACK, Beat, Class, Doing, Drift, Effect, Entering, FORWARD,
    FORWARD_AIM, Footing, KEEP_POSE, LOWERS_NOTHING, Moves, NO_CHANGE, Next, NextFacing, Row,
    RowId, Situation, TO_CROUCHING, TO_FALLING, TO_JUMPING, TO_MOONWALKING, TO_MORPHING,
    TO_RUNNING, TO_SPINNING, TO_STILL, TO_TAKING_OFF, TO_TURNING, Test, UDA, UP_AIM, UP_DOWN, Was,
};
use crate::rules::Gait;
use types::button::{self as key, AIM_DOWN, AIM_UP, DOWN, LEFT, RIGHT, SHOOT, UP};
use types::{Action, Aim, Body, Buttons, Facing, Gun, Item, LandedFrom, Legs, Pose, Side, Turn};

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
        c.body.contains(&pose.body)
            && c.action.contains(&pose.action)
            && c.aim.contains(&pose.aim)
            && c.facing.contains(&pose.facing)
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
const BALLS: &[Body] = &[Body::Ball, Body::SpringBall];
/// The turns on the ground: a ground turn and the moonwalk's jump.
const GROUND_TURNS: &[Action] = &[
    Action::Turning(Turn::Ground),
    Action::Turning(Turn::MoonwalkJump),
];
/// The change of action's own effects, whatever brought the change.
///
/// `Jumping`: starting a straight jump (upright) or a fall, from any action but that one, with
/// a dash she carries on along her facing, else she follows the pad. `WallJump` (§1.2): a spin
/// that followed a wall jump is carried on. `ShrinkingLowersHer`: unmorphing, the one change of
/// shape a ball starts, ends a bounce she is in.
pub static ENTERING: &[Entering] = &[
    Entering {
        into: &[(Body::Crouched, Action::Unmorphing)],
        from: Was::AnyBut(&[Action::Unmorphing]),
        effect: Effect::EndsTheBounce,
    },
    Entering {
        into: &[(Body::Upright, Action::Jumping)],
        from: Was::AnyBut(&[Action::Jumping]),
        effect: Effect::Launch,
    },
    Entering {
        into: &[
            (Body::Upright, Action::Falling),
            (Body::Ball, Action::Falling),
            (Body::SpringBall, Action::Falling),
        ],
        from: Was::AnyBut(&[Action::Falling]),
        effect: Effect::Launch,
    },
    Entering {
        into: &[(Body::Upright, Action::Spinning)],
        from: Was::OneOf(&[Action::WallJumping]),
        effect: Effect::SpinOn,
    },
];

/// A class's defaults: upright, at any aim, facing any way; each class names its actions and its
/// situation.
const UPRIGHT_AT_ANY_AIM: Class = Class {
    body: &[Body::Upright],
    action: &[],
    aim: AIMS,
    facing: &[Facing::Left, Facing::Right, Facing::Screen],
    is: &STANDING,
};

/// The classes, in order: the first her pose is in gives her situation. Body follows the
/// action (§7), so each class names the bodies its actions come in, and a pose of no class
/// (upright `Rolling`, a crouched `Running`) is none of hers.
pub static CLASSES: &[Class] = &[
    Class {
        body: &[Body::Crouched],
        action: &[Action::Crouching, Action::Unmorphing],
        is: &SHAPING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Action::StandingUp],
        is: &SHAPING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        body: BALLS,
        action: &[Action::Morphing],
        is: &SHAPING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        body: BALLS,
        action: &[Action::Jumping],
        is: &HOP,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        body: BALLS,
        action: &[Action::Rolling],
        is: &ROLLING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        body: BALLS,
        action: &[Action::Still],
        is: &RESTING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        body: BALLS,
        action: &[Action::Falling],
        is: &BALL_IN_AIR,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        body: &[Body::Crouched],
        action: &[Action::Turning(Turn::Ground)],
        is: &TURNING_ON,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Action::Turning(Turn::Jump), Action::Turning(Turn::Fall)],
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
        body: &[Body::Crouched],
        action: &[Action::Still],
        is: &CROUCHED,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Action::Running],
        is: &RUNNING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Action::TakingOff],
        is: &TAKING_OFF,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Action::Jumping],
        is: &STRAIGHT_JUMP,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Action::Spinning],
        is: &SPINNING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Action::Falling],
        is: &FALLING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Action::Moonwalking],
        is: &MOONWALKING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Action::WallJumping],
        is: &WALL_JUMPING,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Action::AgainstWall],
        is: &AGAINST_WALL,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[Action::Still],
        facing: &[Facing::Screen],
        is: &FACING_THE_SCREEN,
        ..UPRIGHT_AT_ANY_AIM
    },
    Class {
        action: &[
            Action::Still,
            Action::Landing(LandedFrom::Jump),
            Action::Landing(LandedFrom::Spin),
        ],
        is: &STANDING,
        ..UPRIGHT_AT_ANY_AIM
    },
];

/// `Aiming`: `UpForward` with up, else `DownForward`.
const UP_ELSE_DOWN: &[Aiming] = &[
    Aiming {
        when: &[Test::Held(UP)],
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
        when: &[Test::Held(AIM_DOWN)],
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
        when: &[Test::Held(AIM_UP), Test::Held(AIM_DOWN)],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Test::Held(AIM_UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Test::Held(AIM_DOWN)],
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
        when: &[Test::Held(UP)],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Test::Held(AIM_UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Test::Held(AIM_DOWN)],
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
        when: &[Test::Held(FORWARD), Test::Held(UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Test::Held(FORWARD), Test::Held(DOWN)],
        aim: Aim::DownForward,
    },
    Aiming {
        when: &[Test::Held(UP)],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Test::Held(DOWN)],
        aim: Aim::Down,
    },
    Aiming {
        when: &[Test::Held(AIM_UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Test::Held(AIM_DOWN)],
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
        when: &[Test::Held(UP)],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Test::Held(DOWN)],
        aim: Aim::Down,
    },
    Aiming {
        when: &[Test::Held(AIM_UP)],
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
        when: &[Test::Held(UP)],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Test::Held(AIM_UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Test::Held(AIM_DOWN)],
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
        when: &[
            Test::Held(AIM_UP),
            Test::Held(AIM_DOWN),
            Test::Faces(Side::Left),
        ],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Test::Held(AIM_UP), Test::Held(AIM_DOWN)],
        aim: Aim::DownForward,
    },
    Aiming {
        when: &[Test::Held(AIM_UP)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[],
        aim: Aim::DownForward,
    },
];
/// §2 aim in a turn: a turn in a jump or a fall shows `DownForward` as `Down`; else kept.
const TURN_IN_THE_AIR: &[Aiming] = &[Aiming {
    when: &[Test::Aimed(&[Aim::DownForward])],
    aim: Aim::Down,
}];
/// `Forward`, except landing: kept as she is.
const STANDS_FORWARD: &[Aiming] = &[Aiming {
    when: &[Test::Action(&[Action::Still])],
    aim: Aim::Forward,
}];

/// `Turning` in a jump.
const TO_TURNING_IN_A_JUMP: Next = Next {
    action: Some(Action::Turning(Turn::Jump)),
    aim: TURN_IN_THE_AIR,
    ..TO_TURNING
};
/// `Turning` in a fall.
const TO_TURNING_IN_A_FALL: Next = Next {
    action: Some(Action::Turning(Turn::Fall)),
    aim: TURN_IN_THE_AIR,
    ..TO_TURNING
};
/// The moonwalk's turns: `Turning` with her facing kept, the way she walks.
const TO_MOONWALK_TURN: Next = Next {
    facing: NextFacing::Kept,
    ..TO_TURNING
};
/// Crouched `Unmorphing`, which lowers her 0 px (`ShrinkingLowersHer`).
const TO_UNMORPHING: Next = Next {
    body: Some(Body::Crouched),
    action: Some(Action::Unmorphing),
    lower: LOWERS_NOTHING,
    ..TO_STILL
};
/// The same ball, rolling.
const TO_ROLLING: Next = Next {
    body: None,
    action: Some(Action::Rolling),
    ..TO_STILL
};

/// Any button.
const ANY_BUTTON: Buttons = Buttons(u16::MAX);
/// Every button a straight jump's row names but jump.
const NAMED: Buttons = Buttons(FORWARD.0 | BACK.0 | SHOOT.0 | UDA.0);

/// Standing still facing the screen: a side held turns her to face it (right held beats left
/// held), a turn like any other, from a skid of zero; nothing else she holds or presses moves her
/// out of it. Her moves facing the screen don't move her (`SpeedBuildsAndBrakes`).
static FACING_THE_SCREEN: Situation = Situation {
    doing: Doing::Standing,
    moves: Moves::HaltsAfterDash,
    footing: Footing::OnHerFeet,
    rows: &[
        Row {
            id: RowId::RightHeld,
            needs: &[Test::Held(RIGHT)],
            next: Next {
                facing: NextFacing::To(Facing::Right),
                ..TO_TURNING
            },
            effects: &[Effect::TurnRound],
            beats: &[RowId::LeftHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::LeftHeld,
            needs: &[Test::Held(LEFT)],
            next: Next {
                facing: NextFacing::To(Facing::Left),
                ..TO_TURNING
            },
            effects: &[Effect::TurnRound],
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        effects: &[Effect::EndSkid],
        ..NO_CHANGE
    },
};

/// Standing still, or landing.
static STANDING: Situation = Situation {
    doing: Doing::Standing,
    moves: Moves::HaltsAfterDash,
    footing: Footing::OnHerFeet,
    rows: &[
        Row {
            id: RowId::DownPressed,
            needs: &[Test::Pressed(DOWN)],
            next: Next {
                aim: CROUCH_AIM,
                ..TO_CROUCHING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpPressed,
            needs: &[Test::Pressed(key::JUMP)],
            next: Next {
                aim: TAKE_OFF_AIM,
                ..TO_TAKING_OFF
            },
            effects: &[Effect::Rise],
            beats: &[RowId::DownPressed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackShootAndAnAimButtonHeld,
            needs: &[Test::Held(BACK), Test::Held(SHOOT), Test::Held(AIM)],
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
            needs: &[Test::Held(BACK), Test::Held(AIM_UP), Test::Held(AIM_DOWN)],
            next: TO_TURNING,
            effects: &[Effect::TurnRound],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackAndShootHeld,
            needs: &[Test::Held(BACK), Test::Held(SHOOT)],
            next: Next {
                facing: NextFacing::Back,
                ..TO_MOONWALKING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardAndAnAimButtonHeld,
            needs: &[Test::Held(FORWARD), Test::Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..TO_RUNNING
            },
            beats: &[RowId::ForwardWithUpOrDownHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardWithUpOrDownHeld,
            needs: &[Test::Held(FORWARD), Test::Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..TO_RUNNING
            },
            beats: &[RowId::BackAndShootHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BothAimButtonsHeld,
            needs: &[Test::Held(AIM_UP), Test::Held(AIM_DOWN)],
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
            needs: &[Test::Held(FORWARD)],
            next: TO_RUNNING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::UpHeld,
            needs: &[Test::Held(UP)],
            next: Next {
                aim: UP_AIM,
                ..TO_STILL
            },
            beats: &[RowId::AimUpOrAimDownHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
            needs: &[Test::Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..TO_STILL
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Test::Held(BACK)],
            next: TO_TURNING,
            effects: &[Effect::TurnRound],
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
        effects: &[Effect::EndSkid],
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
            needs: &[Test::Pressed(DOWN)],
            next: TO_CROUCHING,
            beats: &[RowId::JumpPressed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpPressed,
            needs: &[Test::Pressed(key::JUMP)],
            next: TO_SPINNING,
            effects: &[Effect::Rise],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardAndShootHeld,
            needs: &[Test::Held(FORWARD), Test::Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..TO_RUNNING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardAndAnAimButtonHeld,
            needs: &[Test::Held(FORWARD), Test::Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..TO_RUNNING
            },
            beats: &[RowId::ForwardWithUpOrDownHeld, RowId::ForwardAndShootHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardWithUpOrDownHeld,
            needs: &[Test::Held(FORWARD), Test::Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..TO_RUNNING
            },
            beats: &[RowId::ForwardAndShootHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Test::Held(FORWARD)],
            next: Next {
                gun: None,
                ..TO_RUNNING
            },
            beats: &[RowId::BackHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::UpHeld,
            needs: &[Test::Held(UP)],
            next: Next {
                aim: UP_AIM,
                ..TO_STILL
            },
            beats: &[RowId::AimUpOrAimDownHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
            needs: &[Test::Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..TO_STILL
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Test::Held(BACK)],
            next: TO_TURNING,
            effects: &[Effect::TurnRound],
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
            effects: &[Effect::AsRunning],
            aim_step: true,
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElseAtRest,
        next: TO_STILL,
        effects: &[Effect::AsRunning],
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
            needs: &[Test::Held(key::JUMP), Test::Held(FORWARD)],
            next: Next {
                legs: Some(Legs::Striding),
                ..TO_JUMPING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpWithUpDownOrAnAimButtonHeld,
            needs: &[Test::Held(key::JUMP), Test::Held(UDA)],
            next: Next {
                aim: UP_DOWN_AIM,
                ..TO_JUMPING
            },
            beats: &[RowId::JumpAndForwardHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpAndBackHeld,
            needs: &[Test::Held(key::JUMP), Test::Held(BACK)],
            next: TO_TURNING_IN_A_JUMP,
            effects: &[Effect::TurnRound],
            beats: &[
                RowId::JumpWithUpDownOrAnAimButtonHeld,
                RowId::JumpAndForwardHeld,
            ],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ShootHeld,
            needs: &[Test::Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..TO_JUMPING
            },
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        effects: &[Effect::AsRunning],
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
            needs: &[Test::Aimed(&[Aim::Down]), Test::Pressed(DOWN)],
            next: TO_MORPHING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimingDownFacingRightForwardJumpAndShootHeld,
            needs: &[
                Test::Aimed(&[Aim::Down]),
                Test::Faces(Side::Right),
                Test::Held(FORWARD),
                Test::Held(key::JUMP),
                Test::Held(SHOOT),
            ],
            next: Next {
                gun: Some(Gun::Drawn),
                ..TO_JUMPING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardWithUpDownOrAnAimButtonHeld,
            needs: &[Test::Held(FORWARD), Test::Held(UDA)],
            next: Next {
                aim: AIR_AIM,
                ..TO_JUMPING
            },
            beats: &[RowId::AimingDownFacingRightForwardJumpAndShootHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Test::Held(FORWARD)],
            next: Next {
                legs: Some(Legs::Striding),
                ..TO_JUMPING
            },
            beats: &[RowId::ShootHeld, RowId::GunStowedJumpHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ShootHeld,
            needs: &[Test::Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..TO_JUMPING
            },
            beats: &[RowId::GunStowedJumpHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::GunStowedJumpHeld,
            needs: &[Test::Gun(Gun::Stowed), Test::Held(key::JUMP)],
            next: TO_JUMPING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::UpDownOrAnAimButtonHeld,
            needs: &[Test::Held(UDA)],
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
            needs: &[Test::Held(BACK)],
            next: TO_TURNING_IN_A_JUMP,
            effects: &[Effect::TurnRound],
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
                unless: &[&[Test::Not(&Test::Held(AIM))], &[Test::Held(key::JUMP)]],
            }],
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimingDownJumpHeldAlone,
            needs: &[
                Test::Aimed(&[Aim::Down]),
                Test::Held(key::JUMP),
                Test::Not(&Test::Held(NAMED)),
            ],
            beats: &[RowId::GunStowedJumpHeld],
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        effects: &[Effect::AsRunning],
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
            needs: &[Test::Pressed(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..TO_JUMPING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::ShootWithUpDownOrAnAimButtonHeld,
            needs: &[Test::Held(SHOOT), Test::Held(UDA)],
            next: Next {
                aim: UP_DOWN_AIM,
                ..TO_JUMPING
            },
            beats: &[RowId::ForwardAndJumpHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardAndJumpHeld,
            needs: &[Test::Held(FORWARD), Test::Held(key::JUMP)],
            ..NO_CHANGE
        },
        Row {
            id: RowId::UpDownOrAnAimButtonHeld,
            needs: &[Test::Held(UDA)],
            next: Next {
                aim: SPIN_AIM,
                ..TO_JUMPING
            },
            beats: &[RowId::ForwardHeld, RowId::BackHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Test::Held(BACK)],
            next: Next {
                facing: NextFacing::Back,
                ..TO_SPINNING
            },
            effects: &[Effect::TurnRoundIfFacingChanges],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Test::Held(FORWARD)],
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
            needs: &[Test::Aimed(&[Aim::Down]), Test::Pressed(DOWN)],
            next: TO_MORPHING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardWithUpOrDownHeld,
            needs: &[Test::Held(FORWARD), Test::Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..TO_FALLING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::ShootHeld,
            needs: &[Test::Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..TO_FALLING
            },
            beats: &[RowId::ForwardHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Test::Held(FORWARD)],
            next: Next {
                gun: None,
                ..TO_FALLING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::UpDownOrAnAimButtonHeld,
            needs: &[Test::Held(UDA)],
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
            needs: &[Test::Held(BACK)],
            next: TO_TURNING_IN_A_FALL,
            effects: &[Effect::TurnRound],
            beats: &[RowId::ShootHeld, RowId::ForwardHeld],
            beats_unless: &[Beat {
                row: RowId::UpDownOrAnAimButtonHeld,
                unless: &[
                    &[Test::Aimed(&[Aim::Down]), Test::Held(UP_DOWN)],
                    &[Test::Gun(Gun::Drawn), Test::Held(UP_DOWN)],
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
        effects: &[Effect::LoseDash],
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
            needs: &[Test::Pressed(DOWN)],
            next: TO_MORPHING,
            beats: &[RowId::JumpPressed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::UpPressed,
            needs: &[Test::Pressed(UP)],
            next: Next {
                action: Some(Action::StandingUp),
                aim: CROUCH_AIM,
                lower: LOWERS_NOTHING,
                ..TO_STILL
            },
            beats: &[RowId::DownPressed, RowId::JumpPressed, RowId::BackPressed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpPressed,
            needs: &[Test::Pressed(key::JUMP)],
            next: TO_TAKING_OFF,
            effects: &[Effect::Rise, Effect::LiftIfUnaimed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackPressed,
            needs: &[Test::Pressed(BACK)],
            next: Next {
                body: Some(Body::Crouched),
                ..TO_TURNING
            },
            effects: &[Effect::TurnRound],
            beats: &[RowId::DownPressed, RowId::JumpPressed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardAndAnAimButtonHeld,
            needs: &[Test::Held(FORWARD), Test::Held(AIM)],
            next: TO_STILL,
            ..NO_CHANGE
        },
        Row {
            id: RowId::BothAimButtonsHeld,
            needs: &[Test::Held(AIM_UP), Test::Held(AIM_DOWN)],
            next: Next {
                body: Some(Body::Crouched),
                aim: UP_AIM,
                ..TO_STILL
            },
            beats: &[RowId::ForwardAndAnAimButtonHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Test::Held(FORWARD)],
            next: TO_STILL,
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
            needs: &[Test::Held(AIM)],
            next: Next {
                body: Some(Body::Crouched),
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
        effects: &[Effect::EndSkid],
        ..NO_CHANGE
    },
};

/// A ball's "up pressed" row, which beats the jump rows (`PadChart`: "beats the spring ball's
/// hop"; in the air, where there is no hop, the jump without the spring ball).
const UP_PRESSED_UNMORPHS: Row = Row {
    id: RowId::UpPressed,
    needs: &[Test::Pressed(UP)],
    next: TO_UNMORPHING,
    beats: &[RowId::JumpPressedWithoutTheSpringBall],
    ..NO_CHANGE
};
/// A ball's "without the spring ball jump pressed" row.
const JUMP_PRESSED_UNMORPHS: Row = Row {
    id: RowId::JumpPressedWithoutTheSpringBall,
    needs: &[Test::Pressed(key::JUMP), Test::Body(&[Body::Ball])],
    next: TO_UNMORPHING,
    ..NO_CHANGE
};

/// A ball on the ground: the spring ball's hop, unmorphing, and the side of the pad held, which
/// it faces, rolling. Left and right both held (`PadChart`), it rolls rightward.
const BALL_ON_GROUND: &[Row] = &[
    Row {
        id: RowId::JumpPressedWithTheSpringBall,
        needs: &[Test::Pressed(key::JUMP), Test::Body(&[Body::SpringBall])],
        next: Next {
            body: None,
            ..TO_JUMPING
        },
        effects: &[Effect::Rise],
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
        needs: &[Test::Held(RIGHT)],
        next: Next {
            facing: NextFacing::To(Facing::Right),
            ..TO_ROLLING
        },
        effects: &[Effect::TurnRoundIfFacingChanges],
        beats: &[RowId::LeftHeld],
        ..NO_CHANGE
    },
    Row {
        id: RowId::LeftHeld,
        needs: &[Test::Held(LEFT)],
        next: Next {
            facing: NextFacing::To(Facing::Left),
            ..TO_ROLLING
        },
        effects: &[Effect::TurnRoundIfFacingChanges],
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
        needs: &[Test::Held(RIGHT)],
        next: Next {
            facing: NextFacing::To(Facing::Right),
            ..KEEP_POSE
        },
        effects: &[Effect::TurnRoundIfFacingChanges],
        beats_unless: &[Beat {
            row: RowId::LeftHeld,
            unless: &[
                &[Test::Body(&[Body::Ball]), Test::Faces(Side::Left)],
                &[Test::Body(&[Body::SpringBall]), Test::Faces(Side::Right)],
            ],
        }],
        ..NO_CHANGE
    },
    Row {
        id: RowId::LeftHeld,
        needs: &[Test::Held(LEFT)],
        next: Next {
            facing: NextFacing::To(Facing::Left),
            ..KEEP_POSE
        },
        effects: &[Effect::TurnRoundIfFacingChanges],
        ..NO_CHANGE
    },
    Row {
        id: RowId::OffTheGroundWithTheSpringBall,
        needs: &[Test::Body(&[Body::SpringBall])],
        effects: &[Effect::Halt],
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
        needs: &[Test::Held(RIGHT)],
        next: Next {
            facing: NextFacing::To(Facing::Right),
            ..KEEP_POSE
        },
        effects: &[Effect::TurnRoundIfFacingChanges],
        beats: &[RowId::LeftHeld],
        ..NO_CHANGE
    },
    Row {
        id: RowId::LeftHeld,
        needs: &[Test::Held(LEFT)],
        next: Next {
            facing: NextFacing::To(Facing::Left),
            ..KEEP_POSE
        },
        effects: &[Effect::TurnRoundIfFacingChanges],
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
    effects: &[Effect::Halt],
    ..NO_CHANGE
};

/// The ball's last row in the air, without the spring ball: kept, as running.
const BALL_KEEPS_ITS_POSE: Row = Row {
    id: RowId::InTheAirWithoutTheSpringBall,
    effects: &[Effect::AsRunning],
    ..NO_CHANGE
};

/// The spring ball's last row off the ground: kept, halted.
const SPRING_BALL_HALTS: Row = Row {
    id: RowId::OffTheGroundWithTheSpringBall,
    effects: &[Effect::Halt],
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
                Test::Aimed(&[Aim::Up, Aim::DownForward]),
                Test::Held(FORWARD),
                Test::Pressed(key::JUMP),
            ],
            next: TO_SPINNING,
            effects: &[Effect::Rise],
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpPressed,
            needs: &[Test::Pressed(key::JUMP)],
            next: TO_TAKING_OFF,
            effects: &[Effect::Rise],
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimedForwardForwardAndJumpHeld,
            needs: &[
                Test::Aimed(&[Aim::Forward]),
                Test::Held(FORWARD),
                Test::Held(key::JUMP),
            ],
            next: TO_SPINNING,
            effects: &[Effect::Rise],
            beats: &[RowId::JumpPressed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Test::Held(FORWARD)],
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        effects: &[Effect::EndSkid],
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
        needs: &[Test::Held(ANY_BUTTON)],
        ..NO_CHANGE
    }],
    last: Row {
        id: RowId::NothingHeld,
        effects: &[Effect::EndSkid],
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
            needs: &[Test::Pressed(DOWN)],
            next: Next {
                facing: NextFacing::Back,
                ..TO_CROUCHING
            },
            beats: &[RowId::JumpPressed],
            ..NO_CHANGE
        },
        // The moonwalk's two turns, its jump and the one holding the way she walks, keep her
        // facing: it already points the way she walks.
        Row {
            id: RowId::JumpPressed,
            needs: &[Test::Pressed(key::JUMP)],
            next: Next {
                action: Some(Action::Turning(Turn::MoonwalkJump)),
                ..TO_MOONWALK_TURN
            },
            effects: &[Effect::TurnRound],
            ..NO_CHANGE
        },
        Row {
            id: RowId::WalksShootAndAnAimButtonHeld,
            needs: &[Test::Held(FORWARD), Test::Held(SHOOT), Test::Held(AIM)],
            next: Next {
                aim: MOON_AIM,
                ..TO_MOONWALKING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::WalksAndShootHeld,
            needs: &[Test::Held(FORWARD), Test::Held(SHOOT)],
            next: TO_MOONWALKING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::LooksHeld,
            needs: &[Test::Held(BACK)],
            next: Next {
                facing: NextFacing::Back,
                ..TO_RUNNING
            },
            beats: &[RowId::WalksHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::WalksHeld,
            needs: &[Test::Held(FORWARD)],
            next: TO_MOONWALK_TURN,
            effects: &[Effect::TurnRound],
            ..NO_CHANGE
        },
    ],
    last: Row {
        id: RowId::AnythingElse,
        next: Next {
            facing: NextFacing::Back,
            ..TO_STILL
        },
        effects: &[Effect::EndSkid],
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
            needs: &[Test::Pressed(DOWN)],
            next: TO_MORPHING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::ShootHeld,
            needs: &[Test::Held(SHOOT)],
            next: Next {
                gun: Some(Gun::Drawn),
                ..TO_JUMPING
            },
            beats: &[RowId::JumpHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
            needs: &[Test::Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..TO_JUMPING
            },
            beats: &[RowId::ShootHeld, RowId::JumpHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Test::Held(BACK)],
            next: Next {
                facing: NextFacing::Back,
                ..TO_SPINNING
            },
            effects: &[Effect::TurnRoundIfFacingChanges],
            beats: &[RowId::ShootHeld, RowId::AimUpOrAimDownHeld, RowId::JumpHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpHeld,
            needs: &[Test::Held(key::JUMP)],
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
        effects: &[Effect::Halt],
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
            needs: &[Test::Pressed(DOWN)],
            next: TO_CROUCHING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::JumpPressed,
            needs: &[Test::Pressed(key::JUMP)],
            next: TO_TAKING_OFF,
            effects: &[Effect::Rise],
            beats: &[RowId::DownPressed],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackAndAnAimButtonHeld,
            needs: &[Test::Held(BACK), Test::Held(AIM)],
            next: Next {
                facing: NextFacing::Back,
                aim: AIM_DOWN_ELSE_UP,
                ..TO_MOONWALKING
            },
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardWithUpOrDownHeld,
            needs: &[Test::Held(FORWARD), Test::Held(UP_DOWN)],
            next: Next {
                aim: UP_ELSE_DOWN,
                ..TO_RUNNING
            },
            beats: &[RowId::DownPressed, RowId::BackAndAnAimButtonHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::ForwardHeld,
            needs: &[Test::Held(FORWARD)],
            next: TO_RUNNING,
            ..NO_CHANGE
        },
        Row {
            id: RowId::UpHeld,
            needs: &[Test::Held(UP)],
            next: Next {
                aim: UP_AIM,
                ..TO_STILL
            },
            beats: &[RowId::AimUpOrAimDownHeld, RowId::BackHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::AimUpOrAimDownHeld,
            needs: &[Test::Held(AIM)],
            next: Next {
                aim: AIM_UP_ELSE_DOWN,
                ..TO_STILL
            },
            beats: &[RowId::ForwardHeld, RowId::BackHeld],
            ..NO_CHANGE
        },
        Row {
            id: RowId::BackHeld,
            needs: &[Test::Held(BACK)],
            next: TO_TURNING,
            effects: &[Effect::TurnRound],
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
        effects: &[Effect::EndSkid],
        ..NO_CHANGE
    },
};
