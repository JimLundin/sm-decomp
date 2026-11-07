//! The pose chart's data (§7): the rows of each situation's pad chart, with `Aiming`'s rules,
//! then the rows of `WhatHerMovesMet` and of `AnimationEnds`.
//!
//! A pad chart row's order in its situation decides nothing (`PadChart`: rank and beats clauses
//! do); the rows of `WhatHerMovesMet` and `AnimationEnds` are read in order, the first that fits.
//! A next pose has its gun stowed and its legs together unless its cell says otherwise.

use super::Key::{AimDown, AimUp, Back, Down, Forward, Jump, Run, Shoot, Up};
use super::Test::{Aimed, Airborne, Any, Faces, Held, Not, Pressed, SpringBall};
use super::{Aiming, Effect, Met, Next, Row, Side, Test};
use crate::situation::Doing;
use clean_types::{Action, Aim, Body, Gun, Heading, LandedFrom, Legs, Turn};

/// A row's defaults: it needs nothing, keeps her pose, does nothing and beats nothing.
const ROW: Row = Row {
    name: "",
    needs: &[],
    next: KEPT,
    effects: &[],
    beats: &[],
};

/// "kept": her pose stays as it is.
const KEPT: Next = Next {
    body: None,
    action: None,
    facing: Side::Kept,
    aim: &[],
    gun: None,
    legs: None,
};

/// Upright `Still`, aim `Forward`, the gun stowed and the legs together.
const STILL: Next = Next {
    body: Some(Body::Upright),
    action: Some(Action::Still),
    facing: Side::Kept,
    aim: FORWARD,
    gun: Some(Gun::Stowed),
    legs: Some(Legs::Together),
};
/// `Running`, `Forward`.
const RUN: Next = Next {
    action: Some(Action::Running),
    ..STILL
};
/// `Jumping`, `Forward`.
const JUMP: Next = Next {
    action: Some(Action::Jumping),
    ..STILL
};
/// `Falling`, `Forward`.
const FALL: Next = Next {
    action: Some(Action::Falling),
    ..STILL
};
/// `Spinning`.
const SPIN: Next = Next {
    action: Some(Action::Spinning),
    ..STILL
};
/// Upright `TakingOff`, `Forward`.
const TAKE_OFF: Next = Next {
    action: Some(Action::TakingOff),
    ..STILL
};
/// `Moonwalking`, `Forward`.
const MOONWALK: Next = Next {
    action: Some(Action::Moonwalking),
    ..STILL
};
/// Crouched `Crouching`, `Forward`.
const CROUCH: Next = Next {
    body: Some(Body::Crouched),
    action: Some(Action::Crouching),
    ..STILL
};
/// Ball `Morphing`.
const MORPH: Next = Next {
    body: Some(Body::Ball),
    action: Some(Action::Morphing),
    ..STILL
};
/// Crouched `Unmorphing`.
const UNMORPH: Next = Next {
    body: Some(Body::Crouched),
    action: Some(Action::Unmorphing),
    ..STILL
};
/// `Turning` on the ground: facing back, with the aim she had (§2).
const TURN: Next = Next {
    action: Some(Action::Turning(Turn::Ground)),
    facing: Side::Back,
    aim: &[],
    ..STILL
};
/// `Turning` in a jump.
const TURN_IN_A_JUMP: Next = Next {
    action: Some(Action::Turning(Turn::Jump)),
    aim: TURN_IN_THE_AIR,
    ..TURN
};
/// `Turning` in a fall.
const TURN_IN_A_FALL: Next = Next {
    action: Some(Action::Turning(Turn::Fall)),
    aim: TURN_IN_THE_AIR,
    ..TURN
};

// Aiming: the rules the rows name.
/// `Forward`.
const FORWARD: &[Aiming] = &[Aiming {
    when: &[],
    aim: Aim::Forward,
}];
/// `Up`.
const UP: &[Aiming] = &[Aiming {
    when: &[],
    aim: Aim::Up,
}];
/// `UpForward` with aim up, else `DownForward`.
const AIM_UP_ELSE_DOWN: &[Aiming] = &[
    Aiming {
        when: &[Held(&[AimUp])],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[],
        aim: Aim::DownForward,
    },
];
/// `UpForward` with up, else `DownForward`.
const UP_ELSE_DOWN: &[Aiming] = &[
    Aiming {
        when: &[Held(&[Up])],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[],
        aim: Aim::DownForward,
    },
];
/// `DownForward` with aim down, else `UpForward`.
const AIM_DOWN_ELSE_UP: &[Aiming] = &[
    Aiming {
        when: &[Held(&[AimDown])],
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
        when: &[Held(&[AimUp]), Held(&[AimDown])],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Held(&[AimUp])],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Held(&[AimDown])],
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
        when: &[Held(&[Up])],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Held(&[AimUp])],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Held(&[AimDown])],
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
        when: &[Held(&[Forward]), Held(&[Up])],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Held(&[Forward]), Held(&[Down])],
        aim: Aim::DownForward,
    },
    Aiming {
        when: &[Held(&[Up])],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Held(&[Down])],
        aim: Aim::Down,
    },
    Aiming {
        when: &[Held(&[AimUp])],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Held(&[AimDown])],
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
        when: &[Held(&[Up])],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Held(&[Down])],
        aim: Aim::Down,
    },
    Aiming {
        when: &[Held(&[AimUp])],
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
        when: &[Held(&[Up])],
        aim: Aim::Up,
    },
    Aiming {
        when: &[Held(&[AimUp])],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Held(&[AimDown])],
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
        when: &[Held(&[AimUp]), Held(&[AimDown]), Faces(Heading::Left)],
        aim: Aim::UpForward,
    },
    Aiming {
        when: &[Held(&[AimUp]), Held(&[AimDown])],
        aim: Aim::DownForward,
    },
    Aiming {
        when: &[Held(&[AimUp])],
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
/// `Down` becomes `Forward`; else kept.
const DOWN_TO_FORWARD: &[Aiming] = &[Aiming {
    when: &[Aimed(&[Aim::Down])],
    aim: Aim::Forward,
}];
/// `Forward`, except landing: kept as she is.
const STANDS_FORWARD: &[Aiming] = &[Aiming {
    when: &[Test::Action(&[Action::Still])],
    aim: Aim::Forward,
}];

/// Up, down or an aim button.
const UDA: &[super::Key] = &[Up, Down, AimUp, AimDown];
/// On the ground once her moves are made.
const GROUND: Test = Not(&Airborne);

/// Standing still, or landing.
pub const STANDING: &[Row] = &[
    Row {
        name: "down pressed",
        needs: &[Pressed(Down)],
        next: Next {
            aim: CROUCH_AIM,
            ..CROUCH
        },
        effects: &[Effect::Lower(5)],
        ..ROW
    },
    Row {
        name: "jump pressed",
        needs: &[Pressed(Jump)],
        next: Next {
            aim: TAKE_OFF_AIM,
            ..TAKE_OFF
        },
        effects: &[Effect::TakeOff],
        beats: &["down pressed"],
    },
    Row {
        name: "back, shoot and an aim button held",
        needs: &[Held(&[Back]), Held(&[Shoot]), Held(&[AimUp, AimDown])],
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
        needs: &[Held(&[Back]), Held(&[AimUp]), Held(&[AimDown])],
        next: TURN,
        effects: &[Effect::TurnRound],
        ..ROW
    },
    Row {
        name: "back and shoot held",
        needs: &[Held(&[Back]), Held(&[Shoot])],
        next: Next {
            facing: Side::Back,
            ..MOONWALK
        },
        ..ROW
    },
    Row {
        name: "forward and an aim button held",
        needs: &[Held(&[Forward]), Held(&[AimUp, AimDown])],
        next: Next {
            aim: AIM_UP_ELSE_DOWN,
            ..RUN
        },
        beats: &["forward with up or down held"],
        ..ROW
    },
    Row {
        name: "forward with up or down held",
        needs: &[Held(&[Forward]), Held(&[Up, Down])],
        next: Next {
            aim: UP_ELSE_DOWN,
            ..RUN
        },
        ..ROW
    },
    Row {
        name: "both aim buttons held",
        needs: &[Held(&[AimUp]), Held(&[AimDown])],
        next: Next { aim: UP, ..STILL },
        beats: &[
            "back and shoot held",
            "forward and an aim button held",
            "forward with up or down held",
        ],
        ..ROW
    },
    Row {
        name: "forward held",
        needs: &[Held(&[Forward])],
        next: RUN,
        ..ROW
    },
    Row {
        name: "up held",
        needs: &[Held(&[Up])],
        next: Next { aim: UP, ..STILL },
        beats: &["aim up or aim down held"],
        ..ROW
    },
    Row {
        name: "aim up or aim down held",
        needs: &[Held(&[AimUp, AimDown])],
        next: Next {
            aim: AIM_UP_ELSE_DOWN,
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "back held",
        needs: &[Held(&[Back])],
        next: TURN,
        effects: &[Effect::TurnRound],
        beats: &["up held", "aim up or aim down held"],
    },
    Row {
        name: "anything else",
        next: Next {
            aim: STANDS_FORWARD,
            ..KEPT
        },
        effects: &[Effect::EndSkid],
        ..ROW
    },
];

/// Running.
pub const RUNNING: &[Row] = &[
    Row {
        name: "down pressed",
        needs: &[Pressed(Down)],
        next: CROUCH,
        effects: &[Effect::Lower(5)],
        beats: &["jump pressed"],
    },
    Row {
        name: "jump pressed",
        needs: &[Pressed(Jump)],
        next: SPIN,
        effects: &[Effect::Rise],
        ..ROW
    },
    Row {
        name: "forward and shoot held",
        needs: &[Held(&[Forward]), Held(&[Shoot])],
        next: Next {
            gun: Some(Gun::Drawn),
            ..RUN
        },
        ..ROW
    },
    Row {
        name: "forward and an aim button held",
        needs: &[Held(&[Forward]), Held(&[AimUp, AimDown])],
        next: Next {
            aim: AIM_UP_ELSE_DOWN,
            ..RUN
        },
        beats: &["forward with up or down held", "forward and shoot held"],
        ..ROW
    },
    Row {
        name: "forward with up or down held",
        needs: &[Held(&[Forward]), Held(&[Up, Down])],
        next: Next {
            aim: UP_ELSE_DOWN,
            ..RUN
        },
        beats: &["forward and shoot held"],
        ..ROW
    },
    Row {
        name: "forward held",
        needs: &[Held(&[Forward])],
        next: Next { gun: None, ..RUN },
        ..ROW
    },
    Row {
        name: "up held",
        needs: &[Held(&[Up])],
        next: Next { aim: UP, ..STILL },
        beats: &["aim up or aim down held"],
        ..ROW
    },
    Row {
        name: "aim up or aim down held",
        needs: &[Held(&[AimUp, AimDown])],
        next: Next {
            aim: AIM_UP_ELSE_DOWN,
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "back held",
        needs: &[Held(&[Back])],
        next: TURN,
        effects: &[Effect::TurnRound],
        beats: &["up held", "aim up or aim down held"],
    },
    Row {
        name: "anything else",
        next: STILL,
        effects: &[Effect::AsRunning],
        ..ROW
    },
];

/// Taking off.
pub const TAKING_OFF: &[Row] = &[
    Row {
        name: "jump and forward held",
        needs: &[Held(&[Jump]), Held(&[Forward])],
        next: Next {
            legs: Some(Legs::Striding),
            ..JUMP
        },
        effects: &[Effect::StartDrift],
        ..ROW
    },
    Row {
        name: "jump with up, down or an aim button held",
        needs: &[Held(&[Jump]), Held(UDA)],
        next: Next {
            aim: UP_DOWN_AIM,
            ..JUMP
        },
        effects: &[Effect::StartDrift],
        beats: &["jump and forward held"],
    },
    Row {
        name: "jump and back held",
        needs: &[Held(&[Jump]), Held(&[Back])],
        next: TURN_IN_A_JUMP,
        effects: &[Effect::TurnRound],
        beats: &["jump with up, down or an aim button held"],
    },
    Row {
        name: "shoot held",
        needs: &[Held(&[Shoot])],
        next: Next {
            gun: Some(Gun::Drawn),
            ..JUMP
        },
        effects: &[Effect::StartDrift],
        ..ROW
    },
    Row {
        name: "anything else",
        effects: &[Effect::AsRunning],
        ..ROW
    },
];

/// In a straight jump.
pub const STRAIGHT_JUMP: &[Row] = &[
    Row {
        name: "aiming Down: down pressed",
        needs: &[Aimed(&[Aim::Down]), Pressed(Down)],
        next: MORPH,
        effects: &[Effect::Lower(9)],
        ..ROW
    },
    Row {
        name: "aiming Down, she faces Right: forward, jump and shoot held",
        needs: &[
            Aimed(&[Aim::Down]),
            Faces(Heading::Right),
            Held(&[Forward]),
            Held(&[Jump]),
            Held(&[Shoot]),
        ],
        next: Next {
            gun: Some(Gun::Drawn),
            ..JUMP
        },
        ..ROW
    },
    Row {
        name: "forward with up, down or an aim button held",
        needs: &[Held(&[Forward]), Held(UDA)],
        next: Next {
            aim: AIR_AIM,
            ..JUMP
        },
        beats: &["aiming Down, she faces Right: forward, jump and shoot held"],
        ..ROW
    },
    Row {
        name: "forward held",
        needs: &[Held(&[Forward])],
        next: Next {
            legs: Some(Legs::Striding),
            ..JUMP
        },
        beats: &["shoot held", "gun stowed: jump held"],
        ..ROW
    },
    Row {
        name: "shoot held",
        needs: &[Held(&[Shoot])],
        next: Next {
            gun: Some(Gun::Drawn),
            ..JUMP
        },
        beats: &["gun stowed: jump held"],
        ..ROW
    },
    Row {
        name: "gun stowed: jump held",
        needs: &[Test::Gun(Gun::Stowed), Held(&[Jump])],
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
        needs: &[Held(&[Back])],
        next: TURN_IN_A_JUMP,
        effects: &[Effect::TurnRound],
        beats: &[
            "forward held",
            "shoot held",
            "gun stowed: jump held",
            "up, down or an aim button held",
            "aiming Down: jump held, and no other button a row names",
        ],
    },
    Row {
        name: "aiming Down: jump held, and no other button a row names",
        needs: &[
            Aimed(&[Aim::Down]),
            Held(&[Jump]),
            Not(&Held(&[Forward, Back, Shoot, Up, Down, AimUp, AimDown])),
        ],
        beats: &["gun stowed: jump held"],
        ..ROW
    },
    Row {
        name: "anything else",
        effects: &[Effect::AsRunning],
        ..ROW
    },
];

/// Spinning.
pub const SPINNING: &[Row] = &[
    Row {
        name: "shoot pressed",
        needs: &[Pressed(Shoot)],
        next: Next {
            gun: Some(Gun::Drawn),
            ..JUMP
        },
        effects: &[Effect::StartDrift],
        ..ROW
    },
    Row {
        name: "shoot with up, down or an aim button held",
        needs: &[Held(&[Shoot]), Held(UDA)],
        next: Next {
            aim: UP_DOWN_AIM,
            ..JUMP
        },
        effects: &[Effect::StartDrift],
        beats: &["forward and jump held"],
    },
    Row {
        name: "forward and jump held",
        needs: &[Held(&[Forward]), Held(&[Jump])],
        ..ROW
    },
    Row {
        name: "up, down or an aim button held",
        needs: &[Held(UDA)],
        next: Next {
            aim: SPIN_AIM,
            ..JUMP
        },
        effects: &[Effect::StartDrift],
        beats: &["forward held", "back held"],
    },
    Row {
        name: "back held",
        needs: &[Held(&[Back])],
        next: Next {
            facing: Side::Back,
            ..SPIN
        },
        effects: &[Effect::TurnRound],
        ..ROW
    },
    Row {
        name: "forward held",
        needs: &[Held(&[Forward])],
        ..ROW
    },
    Row {
        name: "anything else",
        ..ROW
    },
];

/// Falling, upright.
pub const FALLING: &[Row] = &[
    Row {
        name: "aiming Down: down pressed",
        needs: &[Aimed(&[Aim::Down]), Pressed(Down)],
        next: MORPH,
        effects: &[Effect::Lower(9)],
        ..ROW
    },
    Row {
        name: "forward with up or down held",
        needs: &[Held(&[Forward]), Held(&[Up, Down])],
        next: Next {
            aim: UP_ELSE_DOWN,
            ..FALL
        },
        ..ROW
    },
    Row {
        name: "shoot held",
        needs: &[Held(&[Shoot])],
        next: Next {
            gun: Some(Gun::Drawn),
            ..FALL
        },
        beats: &["forward held"],
        ..ROW
    },
    Row {
        name: "forward held",
        needs: &[Held(&[Forward])],
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
    Row {
        name: "back held",
        needs: &[Held(&[Back])],
        next: TURN_IN_A_FALL,
        effects: &[Effect::TurnRound],
        beats: &[
            "shoot held",
            "forward held",
            "up, down or an aim button held",
        ],
    },
    Row {
        name: "aiming Down or with the gun drawn: up or down held",
        needs: &[
            Any(&[Aimed(&[Aim::Down]), Test::Gun(Gun::Drawn)]),
            Held(&[Up, Down]),
        ],
        next: Next {
            aim: AIR_AIM,
            ..FALL
        },
        // The back row's clause, as a row: the same outcome as the row for up, down or an aim
        // button, which it beats so that one row wins.
        beats: &["back held", "up, down or an aim button held"],
        ..ROW
    },
    Row {
        name: "anything else",
        next: Next {
            aim: FORWARD,
            ..KEPT
        },
        effects: &[Effect::LoseDash],
        ..ROW
    },
];

/// Crouched.
pub const CROUCHED: &[Row] = &[
    Row {
        name: "down pressed",
        needs: &[Pressed(Down)],
        next: MORPH,
        effects: &[Effect::Lower(9)],
        beats: &["jump pressed"],
    },
    Row {
        name: "up pressed",
        needs: &[Pressed(Up)],
        next: Next {
            action: Some(Action::StandingUp),
            aim: CROUCH_AIM,
            ..STILL
        },
        beats: &["down pressed", "jump pressed", "back pressed"],
        ..ROW
    },
    Row {
        name: "jump pressed",
        needs: &[Pressed(Jump)],
        next: TAKE_OFF,
        effects: &[Effect::TakeOff],
        ..ROW
    },
    Row {
        name: "back pressed",
        needs: &[Pressed(Back)],
        next: Next {
            body: Some(Body::Crouched),
            ..TURN
        },
        beats: &["down pressed", "jump pressed"],
        ..ROW
    },
    Row {
        name: "forward and an aim button held",
        needs: &[Held(&[Forward]), Held(&[AimUp, AimDown])],
        next: STILL,
        ..ROW
    },
    Row {
        name: "both aim buttons held",
        needs: &[Held(&[AimUp]), Held(&[AimDown])],
        next: Next {
            body: Some(Body::Crouched),
            aim: UP,
            ..STILL
        },
        beats: &["forward and an aim button held"],
        ..ROW
    },
    Row {
        name: "forward held",
        needs: &[Held(&[Forward])],
        next: STILL,
        ..ROW
    },
    Row {
        name: "aim up or aim down held",
        needs: &[Held(&[AimUp, AimDown])],
        next: Next {
            body: Some(Body::Crouched),
            aim: AIM_UP_ELSE_DOWN,
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "anything else",
        next: Next {
            aim: FORWARD,
            ..KEPT
        },
        effects: &[Effect::EndSkid],
        ..ROW
    },
];

/// A ball, on the ground or in the air.
pub const BALL: &[Row] = &[
    Row {
        name: "jump pressed, with the spring ball, on the ground",
        needs: &[Pressed(Jump), Test::Body(&[Body::SpringBall]), GROUND],
        next: Next { body: None, ..JUMP },
        effects: &[Effect::Rise],
        ..ROW
    },
    Row {
        name: "up pressed",
        needs: &[Pressed(Up)],
        next: UNMORPH,
        beats: &[
            "jump pressed, with the spring ball, on the ground",
            "jump pressed, without the spring ball",
        ],
        ..ROW
    },
    Row {
        name: "jump pressed, without the spring ball",
        needs: &[Pressed(Jump), Test::Body(&[Body::Ball])],
        next: UNMORPH,
        ..ROW
    },
    Row {
        name: "forward held, on the ground",
        needs: &[Held(&[Forward]), GROUND],
        next: Next {
            body: None,
            action: Some(Action::Rolling),
            ..STILL
        },
        ..ROW
    },
    Row {
        name: "back held, on the ground",
        needs: &[Held(&[Back]), GROUND],
        next: Next {
            body: None,
            action: Some(Action::Rolling),
            facing: Side::Back,
            ..STILL
        },
        effects: &[Effect::TurnRound],
        ..ROW
    },
    Row {
        name: "forward held, in the air",
        needs: &[Held(&[Forward]), Airborne],
        ..ROW
    },
    Row {
        name: "back held, in the air",
        needs: &[Held(&[Back]), Airborne],
        next: Next {
            facing: Side::Back,
            ..KEPT
        },
        effects: &[Effect::TurnRound],
        ..ROW
    },
    Row {
        name: "anything else: on the ground",
        needs: &[GROUND],
        next: Next {
            body: None,
            ..STILL
        },
        effects: &[Effect::Halt],
        ..ROW
    },
    Row {
        name: "in the air, without the spring ball",
        needs: &[Airborne, Test::Body(&[Body::Ball])],
        effects: &[Effect::AsRunning],
        ..ROW
    },
    Row {
        name: "off the ground, with the spring ball",
        needs: &[Airborne, Test::Body(&[Body::SpringBall])],
        effects: &[Effect::Halt],
        ..ROW
    },
];

/// Turning on the ground or from the moonwalk's jump, not aimed `UpForward`.
pub const TURNING: &[Row] = &[
    Row {
        name: "aimed Up or DownForward: forward held and jump pressed",
        needs: &[
            Aimed(&[Aim::Up, Aim::DownForward]),
            Held(&[Forward]),
            Pressed(Jump),
        ],
        next: SPIN,
        effects: &[Effect::Rise],
        ..ROW
    },
    Row {
        name: "jump pressed",
        needs: &[Pressed(Jump)],
        next: TAKE_OFF,
        effects: &[Effect::TakeOff],
        ..ROW
    },
    Row {
        name: "aimed Forward: forward and jump held",
        needs: &[Aimed(&[Aim::Forward]), Held(&[Forward]), Held(&[Jump])],
        next: SPIN,
        effects: &[Effect::Rise],
        beats: &["jump pressed"],
    },
    Row {
        name: "forward held",
        needs: &[Held(&[Forward])],
        ..ROW
    },
    Row {
        name: "anything else",
        effects: &[Effect::EndSkid],
        ..ROW
    },
];

/// Turning in a jump or a fall, crouched turning, or turning on the ground or from the
/// moonwalk's jump aimed `UpForward`: she keeps turning.
pub const TURNING_ON: &[Row] = &[
    Row {
        name: "any button held",
        needs: &[Held(&[
            Forward, Back, Jump, Shoot, Run, Up, Down, AimUp, AimDown,
        ])],
        ..ROW
    },
    Row {
        name: "nothing held",
        effects: &[Effect::EndSkid],
        ..ROW
    },
];

/// Her body changing shape: the pad has no say until the animation ends.
pub const SHAPING: &[Row] = &[Row {
    name: "anything, or nothing held",
    ..ROW
}];

/// Moonwalking: forward is the way she walks, which she faces; back the way she looks.
pub const MOONWALKING: &[Row] = &[
    Row {
        name: "down pressed",
        needs: &[Pressed(Down)],
        next: Next {
            facing: Side::Back,
            ..CROUCH
        },
        effects: &[Effect::Lower(5)],
        beats: &["jump pressed"],
    },
    Row {
        name: "jump pressed",
        needs: &[Pressed(Jump)],
        next: Next {
            action: Some(Action::Turning(Turn::MoonwalkJump)),
            ..TURN
        },
        effects: &[Effect::TurnRound],
        ..ROW
    },
    Row {
        name: "the way she walks, shoot and an aim button held",
        needs: &[Held(&[Forward]), Held(&[Shoot]), Held(&[AimUp, AimDown])],
        next: Next {
            aim: MOON_AIM,
            ..MOONWALK
        },
        ..ROW
    },
    Row {
        name: "the way she walks and shoot held",
        needs: &[Held(&[Forward]), Held(&[Shoot])],
        next: MOONWALK,
        ..ROW
    },
    Row {
        name: "the way she looks held",
        needs: &[Held(&[Back])],
        next: Next {
            facing: Side::Back,
            ..RUN
        },
        ..ROW
    },
    Row {
        name: "the way she walks held",
        needs: &[Held(&[Forward])],
        next: TURN,
        effects: &[Effect::TurnRound],
        ..ROW
    },
    Row {
        name: "anything else",
        next: Next {
            facing: Side::Back,
            ..STILL
        },
        effects: &[Effect::EndSkid],
        ..ROW
    },
];

/// Wall jumping.
pub const WALL_JUMPING: &[Row] = &[
    Row {
        name: "down pressed",
        needs: &[Pressed(Down)],
        next: MORPH,
        effects: &[Effect::Lower(9)],
        ..ROW
    },
    Row {
        name: "shoot held",
        needs: &[Held(&[Shoot])],
        next: Next {
            gun: Some(Gun::Drawn),
            ..JUMP
        },
        effects: &[Effect::StartDrift],
        beats: &["jump held"],
    },
    Row {
        name: "aim up or aim down held",
        needs: &[Held(&[AimUp, AimDown])],
        next: Next {
            aim: AIM_UP_ELSE_DOWN,
            ..JUMP
        },
        effects: &[Effect::StartDrift],
        beats: &["shoot held", "jump held"],
    },
    Row {
        name: "back held",
        needs: &[Held(&[Back])],
        next: Next {
            facing: Side::Back,
            ..SPIN
        },
        effects: &[Effect::TurnRound, Effect::CarrySpin],
        beats: &["shoot held", "aim up or aim down held", "jump held"],
    },
    Row {
        name: "jump held",
        needs: &[Held(&[Jump])],
        ..ROW
    },
    Row {
        name: "anything else",
        next: SPIN,
        effects: &[Effect::Halt, Effect::CarrySpin],
        ..ROW
    },
];

/// Against the wall.
pub const AGAINST_WALL: &[Row] = &[
    Row {
        name: "down pressed",
        needs: &[Pressed(Down)],
        next: CROUCH,
        effects: &[Effect::Lower(5)],
        ..ROW
    },
    Row {
        name: "jump pressed",
        needs: &[Pressed(Jump)],
        next: TAKE_OFF,
        effects: &[Effect::TakeOff],
        beats: &["down pressed"],
    },
    Row {
        name: "back and an aim button held",
        needs: &[Held(&[Back]), Held(&[AimUp, AimDown])],
        next: Next {
            facing: Side::Back,
            aim: AIM_DOWN_ELSE_UP,
            ..MOONWALK
        },
        ..ROW
    },
    Row {
        name: "forward with up or down held",
        needs: &[Held(&[Forward]), Held(&[Up, Down])],
        next: Next {
            aim: UP_ELSE_DOWN,
            ..RUN
        },
        beats: &["down pressed"],
        ..ROW
    },
    Row {
        name: "forward held",
        needs: &[Held(&[Forward])],
        next: RUN,
        ..ROW
    },
    Row {
        name: "up held",
        needs: &[Held(&[Up])],
        next: Next { aim: UP, ..STILL },
        beats: &["aim up or aim down held", "back held"],
        ..ROW
    },
    Row {
        name: "aim up or aim down held",
        needs: &[Held(&[AimUp, AimDown])],
        next: Next {
            aim: AIM_UP_ELSE_DOWN,
            ..STILL
        },
        beats: &["forward held", "back held"],
        ..ROW
    },
    Row {
        name: "back held",
        needs: &[Held(&[Back])],
        next: TURN,
        effects: &[Effect::TurnRound],
        ..ROW
    },
    Row {
        name: "anything else",
        next: Next {
            aim: FORWARD,
            ..KEPT
        },
        effects: &[Effect::EndSkid],
        ..ROW
    },
];

/// `WhatHerMovesMet`: what her moves met chooses her pose over the pad, the first row that fits.
///
/// A floor met while turning or while her body changes shape has no row: the turn's own rows
/// apply, and a changing body's pad has no say. The aim change's step blocked is her pose chosen,
/// against the wall (`AimingAndShooting`).
pub const MOVES_MET: &[Row] = &[
    Row {
        name: "a wall stop, running aimed Up",
        needs: &[
            Test::Met(Met::Wall),
            Test::Doing(&[Doing::Running]),
            Aimed(&[Aim::Up]),
        ],
        next: Next { aim: UP, ..STILL },
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
            Held(&[Shoot]),
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

/// `AnimationEnds`: the pose an ending animation hands over, whatever the pad, the first row
/// that fits. A turn already faces its new way.
pub const ANIMATION_ENDS: &[Row] = &[
    Row {
        name: "TakingOff",
        needs: &[Test::Action(&[Action::TakingOff])],
        next: Next { aim: &[], ..JUMP },
        effects: &[Effect::StartDrift],
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
        effects: &[Effect::LoseDash],
        ..ROW
    },
    Row {
        name: "Turning on the ground",
        needs: &[Test::Action(&[Action::Turning(Turn::Ground)])],
        next: Next { aim: &[], ..STILL },
        effects: &[Effect::LoseDash],
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
        effects: &[Effect::LoseDash, Effect::StartDrift],
        ..ROW
    },
    Row {
        name: "Turning in a jump",
        needs: &[Test::Action(&[Action::Turning(Turn::Jump)])],
        next: Next { aim: &[], ..JUMP },
        effects: &[Effect::LoseDash, Effect::StartDrift],
        ..ROW
    },
    Row {
        name: "Turning in a fall",
        needs: &[Test::Action(&[Action::Turning(Turn::Fall)])],
        next: Next { aim: &[], ..FALL },
        effects: &[Effect::LoseDash, Effect::StartDrift],
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
        effects: &[Effect::StartDrift],
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
        effects: &[Effect::StartDrift],
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
        effects: &[Effect::StartDrift],
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
