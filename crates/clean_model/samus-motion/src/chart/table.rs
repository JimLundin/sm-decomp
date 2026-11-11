//! The rows of `WhatHerMovesMet` and of `AnimationEnds` (§7), read in order, the first that
//! fits. A next pose has its gun stowed and its legs together unless its cell says otherwise.

use crate::rows::Effect::{self, ComesToTheGround, LoseDash};
use crate::rows::Test::{Aimed, Airborne, Held, SpringBall};
use crate::rows::{
    DOWN_TO_FORWARD, Doing, Met, NO_CHANGE, Next, Row, RowId, TO_FALLING, TO_JUMPING, TO_SPINNING,
    TO_STILL, Test, UP_AIM,
};
use clean_types::button::SHOOT;
use clean_types::{Action, Aim, Body, Gun, LandedFrom, Legs, Turn};

/// `WhatHerMovesMet`: what her moves met chooses her pose over the pad, the first row that fits.
///
/// A floor met while turning or while her body changes shape has no row: the turn's own rows
/// apply, and a changing body's pad has no say. The aim change's step comes after the pose is
/// chosen, so its row's outcome is `STEP_BLOCKED`, apart.
pub const MOVES_MET: &[Row] = &[
    Row {
        id: RowId::WallStopAimedUp,
        needs: &[
            Test::Met(Met::Wall),
            Test::Doing(&[Doing::Running]),
            Aimed(&[Aim::Up]),
        ],
        next: Next {
            aim: UP_AIM,
            ..TO_STILL
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::WallStop,
        needs: &[Test::Met(Met::Wall), Test::Doing(&[Doing::Running])],
        next: Next {
            action: Some(Action::AgainstWall),
            aim: DOWN_TO_FORWARD,
            ..TO_STILL
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::WallKick,
        needs: &[Test::Met(Met::WallKick), Test::Doing(&[Doing::Spinning])],
        next: Next {
            action: Some(Action::WallJumping),
            ..TO_STILL
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::FloorFiring,
        needs: &[
            Test::Met(Met::Floor),
            Test::Doing(&[Doing::StraightJump, Doing::Falling]),
            Aimed(&[Aim::Forward]),
            Held(SHOOT),
        ],
        next: Next {
            action: Some(Action::Landing(LandedFrom::Jump)),
            gun: Some(Gun::Drawn),
            ..TO_STILL
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::FloorFromAJump,
        needs: &[
            Test::Met(Met::Floor),
            Test::Doing(&[Doing::StraightJump, Doing::Falling]),
        ],
        next: Next {
            action: Some(Action::Landing(LandedFrom::Jump)),
            aim: DOWN_TO_FORWARD,
            ..TO_STILL
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::FloorFromASpin,
        needs: &[
            Test::Met(Met::Floor),
            Test::Doing(&[Doing::Spinning, Doing::WallJumping]),
        ],
        next: Next {
            action: Some(Action::Landing(LandedFrom::Spin)),
            ..TO_STILL
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::FloorBouncing,
        needs: &[
            Test::Met(Met::Floor),
            Test::Doing(&[Doing::Ball, Doing::Hop]),
            Airborne,
        ],
        ..NO_CHANGE
    },
    Row {
        id: RowId::FloorComingToRest,
        needs: &[
            Test::Met(Met::Floor),
            Test::Doing(&[Doing::Ball, Doing::Hop]),
        ],
        next: Next {
            body: None,
            ..TO_STILL
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::NoFloorUprightOrCrouched,
        needs: &[
            Test::Met(Met::NoFloor),
            Test::Body(&[Body::Upright, Body::Crouched]),
        ],
        next: TO_FALLING,
        ..NO_CHANGE
    },
    Row {
        id: RowId::NoFloorBall,
        needs: &[
            Test::Met(Met::NoFloor),
            Test::Body(&[Body::Ball, Body::SpringBall]),
        ],
        next: Next {
            body: None,
            ..TO_FALLING
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::Ceiling,
        needs: &[Test::Met(Met::Ceiling)],
        ..NO_CHANGE
    },
];

/// `WhatHerMovesMet`, `AimingAndShooting`: the outcome of the row for a block or solid enemy in
/// the way of the aim change's step, a running pose chosen: `AgainstWall`, facing kept, with the
/// aim of the running pose chosen, and no effect. The step comes after the pose is chosen, so
/// this outcome applies to that pose (`Choice::blocked`), apart from `MOVES_MET`.
pub const STEP_BLOCKED: Next = Next {
    action: Some(Action::AgainstWall),
    aim: &[],
    ..TO_STILL
};

/// `AnimationEnds`: the pose an ending animation hands over, whatever the pad.
///
/// The first row that fits. A turn already faces its new way. An end unless a jump gives way
/// before any row is read (`chart`'s `gives_way`).
pub const ANIMATION_ENDS: &[Row] = &[
    Row {
        id: RowId::TakingOffEnds,
        needs: &[Test::Action(&[Action::TakingOff])],
        next: Next {
            aim: &[],
            ..TO_JUMPING
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::CrouchedTurnEnds,
        needs: &[
            Test::Body(&[Body::Crouched]),
            Test::Action(&[Action::Turning(Turn::Ground)]),
        ],
        next: Next {
            body: Some(Body::Crouched),
            aim: &[],
            ..TO_STILL
        },
        effects: &[LoseDash],
        ..NO_CHANGE
    },
    Row {
        id: RowId::GroundTurnEnds,
        needs: &[Test::Action(&[Action::Turning(Turn::Ground)])],
        next: Next {
            aim: &[],
            ..TO_STILL
        },
        effects: &[LoseDash],
        ..NO_CHANGE
    },
    Row {
        id: RowId::JumpTurnEndsAimedForward,
        needs: &[
            Test::Action(&[Action::Turning(Turn::Jump)]),
            Aimed(&[Aim::Forward]),
        ],
        next: Next {
            legs: Some(Legs::Striding),
            ..TO_JUMPING
        },
        effects: &[LoseDash],
        ..NO_CHANGE
    },
    Row {
        id: RowId::JumpTurnEnds,
        needs: &[Test::Action(&[Action::Turning(Turn::Jump)])],
        next: Next {
            aim: &[],
            ..TO_JUMPING
        },
        effects: &[LoseDash],
        ..NO_CHANGE
    },
    Row {
        id: RowId::FallTurnEnds,
        needs: &[Test::Action(&[Action::Turning(Turn::Fall)])],
        next: Next {
            aim: &[],
            ..TO_FALLING
        },
        effects: &[LoseDash],
        ..NO_CHANGE
    },
    Row {
        id: RowId::MoonwalkJumpTurnEnds,
        needs: &[Test::Action(&[Action::Turning(Turn::MoonwalkJump)])],
        next: TO_SPINNING,
        effects: &[Effect::Moonfall],
        ..NO_CHANGE
    },
    Row {
        id: RowId::CrouchingEnds,
        needs: &[Test::Action(&[Action::Crouching])],
        next: Next {
            body: Some(Body::Crouched),
            aim: &[],
            ..TO_STILL
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::StandingUpEnds,
        needs: &[Test::Action(&[Action::StandingUp])],
        next: Next {
            aim: &[],
            ..TO_STILL
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::MorphingEndsSpringBallInTheAir,
        needs: &[Test::Action(&[Action::Morphing]), SpringBall, Airborne],
        next: Next {
            body: Some(Body::SpringBall),
            ..TO_FALLING
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::MorphingEndsSpringBall,
        needs: &[Test::Action(&[Action::Morphing]), SpringBall],
        next: Next {
            body: Some(Body::SpringBall),
            ..TO_STILL
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::MorphingEndsInTheAir,
        needs: &[Test::Action(&[Action::Morphing]), Airborne],
        next: Next {
            body: Some(Body::Ball),
            ..TO_FALLING
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::MorphingEnds,
        needs: &[Test::Action(&[Action::Morphing])],
        next: Next {
            body: Some(Body::Ball),
            ..TO_STILL
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::LandingEnds,
        needs: &[Test::Action(&[
            Action::Landing(LandedFrom::Jump),
            Action::Landing(LandedFrom::Spin),
        ])],
        next: Next {
            aim: &[],
            ..TO_STILL
        },
        ..NO_CHANGE
    },
    Row {
        id: RowId::UnmorphingEnds,
        needs: &[Test::Action(&[Action::Unmorphing])],
        next: Next {
            body: Some(Body::Crouched),
            ..TO_STILL
        },
        effects: &[ComesToTheGround],
        ..NO_CHANGE
    },
];
