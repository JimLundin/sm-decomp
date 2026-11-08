//! The rows of `WhatHerMovesMet` and of `AnimationEnds` (§7), read in order, the first that
//! fits. A next pose has its gun stowed and its legs together unless its cell says otherwise.

use crate::rows::Effect::{self, LoseDash};
use crate::rows::Test::{Aimed, Airborne, Held, SpringBall};
use crate::rows::{
    DOWN_TO_FORWARD, Doing, FALL, JUMP, Met, Next, ROW, Row, SPIN, STILL, Test, UP_AIM,
};
use clean_types::button::SHOOT;
use clean_types::{Action, Aim, Body, Gun, LandedFrom, Legs, Turn};

/// `WhatHerMovesMet`: what her moves met chooses her pose over the pad, the first row that fits.
///
/// A floor met while turning or while her body changes shape has no row: the turn's own rows
/// apply, and a changing body's pad has no say. The aim change's step blocked is read of the
/// running pose chosen, which it puts against the wall (`AimingAndShooting`).
pub const MOVES_MET: &[Row] = &[
    Row {
        name: "a wall stop, running aimed Up",
        needs: &[
            Test::Met(Met::Wall),
            Test::Doing(&[Doing::Running]),
            Aimed(&[Aim::Up]),
        ],
        next: Next {
            aim: UP_AIM,
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "a wall stop, running",
        needs: &[Test::Met(Met::Wall), Test::Doing(&[Doing::Running])],
        next: Next {
            action: Some(Action::AgainstWall),
            aim: DOWN_TO_FORWARD,
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "a block or solid enemy in the way of the aim change's step",
        needs: &[
            Test::Met(Met::StepBlocked),
            Test::Action(&[Action::Running]),
        ],
        next: Next {
            action: Some(Action::AgainstWall),
            aim: &[],
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "a wall near her as she presses jump, spinning",
        needs: &[Test::Met(Met::WallKick), Test::Doing(&[Doing::Spinning])],
        next: Next {
            action: Some(Action::WallJumping),
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "a floor, upright in a straight jump or falling, aimed Forward with shoot held",
        needs: &[
            Test::Met(Met::Floor),
            Test::Doing(&[Doing::StraightJump, Doing::Falling]),
            Aimed(&[Aim::Forward]),
            Held(SHOOT),
        ],
        next: Next {
            action: Some(Action::Landing(LandedFrom::Jump)),
            gun: Some(Gun::Drawn),
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "a floor, upright in a straight jump or falling",
        needs: &[
            Test::Met(Met::Floor),
            Test::Doing(&[Doing::StraightJump, Doing::Falling]),
        ],
        next: Next {
            action: Some(Action::Landing(LandedFrom::Jump)),
            aim: DOWN_TO_FORWARD,
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "a floor, spinning or wall jumping",
        needs: &[
            Test::Met(Met::Floor),
            Test::Doing(&[Doing::Spinning, Doing::WallJumping]),
        ],
        next: Next {
            action: Some(Action::Landing(LandedFrom::Spin)),
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "a floor, a ball bouncing",
        needs: &[
            Test::Met(Met::Floor),
            Test::Doing(&[Doing::Ball, Doing::Hop]),
            Airborne,
        ],
        ..ROW
    },
    Row {
        name: "a floor, a ball coming to rest",
        needs: &[
            Test::Met(Met::Floor),
            Test::Doing(&[Doing::Ball, Doing::Hop]),
        ],
        next: Next {
            body: None,
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "no floor, upright on the ground",
        needs: &[Test::Met(Met::NoFloor), Test::Body(&[Body::Upright])],
        next: FALL,
        ..ROW
    },
    Row {
        name: "no floor, a ball",
        needs: &[
            Test::Met(Met::NoFloor),
            Test::Body(&[Body::Ball, Body::SpringBall]),
        ],
        next: Next { body: None, ..FALL },
        ..ROW
    },
    Row {
        name: "a ceiling, rising",
        needs: &[Test::Met(Met::Ceiling)],
        ..ROW
    },
];

/// `AnimationEnds`: the pose an ending animation hands over, whatever the pad.
///
/// The first row that fits. A turn already faces its new way.
pub const ANIMATION_ENDS: &[Row] = &[
    Row {
        name: "TakingOff",
        needs: &[Test::Action(&[Action::TakingOff])],
        next: Next { aim: &[], ..JUMP },
        ..ROW
    },
    Row {
        name: "crouched Turning",
        needs: &[
            Test::Body(&[Body::Crouched]),
            Test::Action(&[Action::Turning(Turn::Ground)]),
        ],
        next: Next {
            body: Some(Body::Crouched),
            aim: &[],
            ..STILL
        },
        effects: &[LoseDash],
        ..ROW
    },
    Row {
        name: "Turning on the ground",
        needs: &[Test::Action(&[Action::Turning(Turn::Ground)])],
        next: Next { aim: &[], ..STILL },
        effects: &[LoseDash],
        ..ROW
    },
    Row {
        name: "Turning in a jump, aimed Forward",
        needs: &[
            Test::Action(&[Action::Turning(Turn::Jump)]),
            Aimed(&[Aim::Forward]),
        ],
        next: Next {
            legs: Some(Legs::Striding),
            ..JUMP
        },
        effects: &[LoseDash],
        ..ROW
    },
    Row {
        name: "Turning in a jump",
        needs: &[Test::Action(&[Action::Turning(Turn::Jump)])],
        next: Next { aim: &[], ..JUMP },
        effects: &[LoseDash],
        ..ROW
    },
    Row {
        name: "Turning in a fall",
        needs: &[Test::Action(&[Action::Turning(Turn::Fall)])],
        next: Next { aim: &[], ..FALL },
        effects: &[LoseDash],
        ..ROW
    },
    Row {
        name: "Turning from the moonwalk's jump",
        needs: &[Test::Action(&[Action::Turning(Turn::MoonwalkJump)])],
        next: SPIN,
        effects: &[Effect::Moonfall],
        ..ROW
    },
    Row {
        name: "Crouching",
        needs: &[Test::Action(&[Action::Crouching])],
        next: Next {
            body: Some(Body::Crouched),
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "StandingUp",
        needs: &[Test::Action(&[Action::StandingUp])],
        next: STILL,
        ..ROW
    },
    Row {
        name: "Morphing, with the spring ball, in the air",
        needs: &[Test::Action(&[Action::Morphing]), SpringBall, Airborne],
        next: Next {
            body: Some(Body::SpringBall),
            ..FALL
        },
        ..ROW
    },
    Row {
        name: "Morphing, with the spring ball",
        needs: &[Test::Action(&[Action::Morphing]), SpringBall],
        next: Next {
            body: Some(Body::SpringBall),
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "Morphing, in the air",
        needs: &[Test::Action(&[Action::Morphing]), Airborne],
        next: Next {
            body: Some(Body::Ball),
            ..FALL
        },
        ..ROW
    },
    Row {
        name: "Morphing",
        needs: &[Test::Action(&[Action::Morphing])],
        next: Next {
            body: Some(Body::Ball),
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "Landing",
        needs: &[Test::Action(&[
            Action::Landing(LandedFrom::Jump),
            Action::Landing(LandedFrom::Spin),
        ])],
        next: Next { aim: &[], ..STILL },
        ..ROW
    },
    Row {
        name: "Unmorphing, in the air",
        needs: &[Test::Action(&[Action::Unmorphing]), Airborne],
        next: FALL,
        ..ROW
    },
    Row {
        name: "Unmorphing",
        needs: &[Test::Action(&[Action::Unmorphing])],
        next: Next {
            body: Some(Body::Crouched),
            ..STILL
        },
        ..ROW
    },
];
