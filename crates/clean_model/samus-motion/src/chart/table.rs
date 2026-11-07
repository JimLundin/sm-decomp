//! `PadChart`'s table (§7) as data: one `Chart` per situation, its rows in the spec's order, and
//! `Aiming`'s rules. A row's index is its place in its chart, which `beats` names.

use super::{
    AD, AU, AimTo, B, BodyTo, Chart, D, Effect, F, FaceTo, J, Keys, RUN, Row, S, To, U, When,
};
use clean_types::{Action, Aim, Body};

const UD: Keys = U | D;
const AIMS: Keys = AU | AD;
const ANY: Keys = F | B | J | S | RUN | U | D | AU | AD;

// Aiming: the rules a row names.
const TAKE_OFF_AIM: AimTo = AimTo::Pick(
    &[(U, Aim::Up), (AU, Aim::UpForward), (AD, Aim::DownForward)],
    Aim::Forward,
);
const CROUCH_AIM: AimTo = AimTo::Pick(
    &[
        (AU | AD, Aim::Up),
        (AU, Aim::UpForward),
        (AD, Aim::DownForward),
    ],
    Aim::Forward,
);
const AIR_AIM: AimTo = AimTo::Pick(
    &[
        (F | U, Aim::UpForward),
        (F | D, Aim::DownForward),
        (U, Aim::Up),
        (D, Aim::Down),
        (AU, Aim::UpForward),
        (AD, Aim::DownForward),
    ],
    Aim::Forward,
);
/// `UpForward` with aim up, else `DownForward`.
const AU_ELSE_AD: AimTo = AimTo::Pick(&[(AU, Aim::UpForward)], Aim::DownForward);
/// `UpForward` with up, else `DownForward`.
const U_ELSE_D: AimTo = AimTo::Pick(&[(U, Aim::UpForward)], Aim::DownForward);
/// `DownForward` with aim down, else `UpForward`.
const AD_ELSE_AU: AimTo = AimTo::Pick(&[(AD, Aim::DownForward)], Aim::UpForward);
/// `Up` with up, else `Down` with down, else `UpForward` with aim up, else `DownForward`.
const UP_DOWN_THEN_AIMS: AimTo = AimTo::Pick(
    &[(U, Aim::Up), (D, Aim::Down), (AU, Aim::UpForward)],
    Aim::DownForward,
);

const ALWAYS: When = When {
    aims: &[],
    ground: None,
    spring: None,
};
const fn aimed(aims: &'static [Aim]) -> When {
    When { aims, ..ALWAYS }
}
const fn on_ground(ground: bool, spring: Option<bool>) -> When {
    When {
        ground: Some(ground),
        spring,
        ..ALWAYS
    }
}

const fn row(pressed: &'static [Keys], held: &'static [Keys], to: To, effect: Effect) -> Row {
    Row {
        when: ALWAYS,
        pressed,
        held,
        without: 0,
        to,
        effect,
        beats: &[],
    }
}

impl Row {
    const fn when(self, when: When) -> Self {
        Self { when, ..self }
    }
    const fn beats(self, beats: &'static [usize]) -> Self {
        Self { beats, ..self }
    }
    const fn without(self, without: Keys) -> Self {
        Self { without, ..self }
    }
}

/// "kept": her pose stays as it is.
const KEPT: To = To {
    action: None,
    body: BodyTo::Kept,
    facing: FaceTo::Kept,
    aim: AimTo::Kept,
};
const fn go(action: Action) -> To {
    To {
        action: Some(action),
        body: BodyTo::Natural,
        ..KEPT
    }
}

impl To {
    const fn body(self, body: Body) -> Self {
        Self {
            body: BodyTo::Set(body),
            ..self
        }
    }
    const fn body_kept(self) -> Self {
        Self {
            body: BodyTo::Kept,
            ..self
        }
    }
    const fn back(self) -> Self {
        Self {
            facing: FaceTo::Back,
            ..self
        }
    }
    const fn aim(self, aim: Aim) -> Self {
        Self {
            aim: AimTo::Set(aim),
            ..self
        }
    }
    const fn pick(self, aim: AimTo) -> Self {
        Self { aim, ..self }
    }
}

use Action::{
    Crouching, Falling, Jumping, Moonwalking, Morphing, Rolling, Running, Spinning, StandingUp,
    Still, TakingOff, Turning, Unmorphing,
};
use Aim::{Down, DownForward, Forward, Up};
use Effect::{
    AsRunning, EndSkid, Halt, Hop, LoseDash, Nothing, SpinStart, StandUp, TakeOff, TurnRound,
    TurnRoundThenSpin,
};

pub const STANDING: Chart = Chart {
    rows: &[
        /* 0 */ row(&[D], &[], go(Crouching).pick(CROUCH_AIM), Nothing),
        /* 1 */ row(&[J], &[], go(TakingOff).pick(TAKE_OFF_AIM), TakeOff).beats(&[0]),
        /* 2 */
        row(
            &[],
            &[B, S, AIMS],
            go(Moonwalking).back().pick(AD_ELSE_AU),
            Nothing,
        )
        .beats(&[3]),
        /* 3 */ row(&[], &[B, AU, AD], go(Turning).back(), TurnRound),
        /* 4 */ row(&[], &[B, S], go(Moonwalking).back().aim(Forward), Nothing),
        /* 5 */ row(&[], &[F, AIMS], go(Running).pick(AU_ELSE_AD), Nothing).beats(&[6]),
        /* 6 */ row(&[], &[F, UD], go(Running).pick(U_ELSE_D), Nothing),
        /* 7 */ row(&[], &[AU, AD], go(Still).aim(Up), Nothing).beats(&[4, 5, 6]),
        /* 8 */ row(&[], &[F], go(Running).aim(Forward), Nothing),
        /* 9 */ row(&[], &[U], go(Still).aim(Up), Nothing).beats(&[10]),
        /* 10 */ row(&[], &[AIMS], go(Still).pick(AU_ELSE_AD), Nothing),
        /* 11 */ row(&[], &[B], go(Turning).back(), TurnRound).beats(&[9, 10]),
    ],
    otherwise: &[row(&[], &[], KEPT.aim(Forward), EndSkid)],
};

pub const RUNNING: Chart = Chart {
    rows: &[
        /* 0 */ row(&[D], &[], go(Crouching).aim(Forward), Nothing).beats(&[1]),
        /* 1 */ row(&[J], &[], go(Spinning), SpinStart),
        /* 2 */ row(&[], &[F, AIMS], go(Running).pick(AU_ELSE_AD), Nothing).beats(&[3]),
        /* 3 */ row(&[], &[F, UD], go(Running).pick(U_ELSE_D), Nothing),
        /* 4 */ row(&[], &[F], go(Running).aim(Forward), Nothing),
        /* 5 */ row(&[], &[U], go(Still).body(Body::Upright).aim(Up), Nothing).beats(&[6]),
        /* 6 */
        row(
            &[],
            &[AIMS],
            go(Still).body(Body::Upright).pick(AU_ELSE_AD),
            Nothing,
        ),
        /* 7 */ row(&[], &[B], go(Turning).back(), TurnRound).beats(&[5, 6]),
    ],
    otherwise: &[row(
        &[],
        &[],
        go(Still).body(Body::Upright).aim(Forward),
        AsRunning,
    )],
};

pub const TAKING_OFF: Chart = Chart {
    rows: &[
        /* 0 */ row(&[], &[J, F], go(Jumping).aim(Forward), Nothing),
        /* 1 */
        row(
            &[],
            &[J, UD | AIMS],
            go(Jumping).pick(UP_DOWN_THEN_AIMS),
            Nothing,
        )
        .beats(&[0]),
        /* 2 */ row(&[], &[J, B], go(Turning).back(), TurnRound).beats(&[1]),
        /* 3 */ row(&[], &[S], go(Jumping).aim(Forward), Nothing),
    ],
    otherwise: &[row(&[], &[], KEPT, AsRunning)],
};

pub const STRAIGHT_JUMP: Chart = Chart {
    rows: &[
        /* 0 */ row(&[D], &[], go(Morphing).aim(Forward), Nothing).when(aimed(&[Down])),
        /* 1 */ row(&[], &[F, UD | AIMS], go(Jumping).pick(AIR_AIM), Nothing),
        /* 2 */ row(&[], &[F | J | S], go(Jumping).aim(Forward), Nothing),
        /* 3 */ row(&[], &[UD | AIMS], go(Jumping).pick(AIR_AIM), Nothing).beats(&[2]),
        /* 4 */ row(&[], &[B], go(Turning).back(), TurnRound).beats(&[2, 3, 5]),
        /* 5 */
        row(&[], &[J], KEPT, Nothing)
            .when(aimed(&[Down]))
            .without(F | B | S | UD | AIMS)
            .beats(&[2]),
    ],
    otherwise: &[row(&[], &[], KEPT, AsRunning)],
};

const SPIN_TO_JUMP_AIM: AimTo = AimTo::Pick(
    &[(U, Aim::Up), (AU, Aim::UpForward), (AD, Aim::DownForward)],
    Aim::Down,
);

pub const SPINNING: Chart = Chart {
    rows: &[
        /* 0 */ row(&[S], &[], go(Jumping).aim(Forward), Nothing),
        /* 1 */
        row(
            &[],
            &[S, UD | AIMS],
            go(Jumping).pick(UP_DOWN_THEN_AIMS),
            Nothing,
        )
        .beats(&[2]),
        /* 2 */ row(&[], &[F, J], KEPT, Nothing),
        /* 3 */
        row(
            &[],
            &[UD | AIMS],
            go(Jumping).pick(SPIN_TO_JUMP_AIM),
            Nothing,
        )
        .beats(&[4, 5]),
        /* 4 */ row(&[], &[B], go(Spinning).back(), TurnRound),
        /* 5 */ row(&[], &[F], KEPT, Nothing),
    ],
    otherwise: &[row(&[], &[], KEPT, Nothing)],
};

pub const FALLING: Chart = Chart {
    rows: &[
        /* 0 */ row(&[D], &[], go(Morphing).aim(Forward), Nothing).when(aimed(&[Down])),
        /* 1 */ row(&[], &[F, UD], go(Falling).pick(U_ELSE_D), Nothing),
        /* 2 */ row(&[], &[F | S], go(Falling).aim(Forward), Nothing),
        /* 3 */ row(&[], &[UD | AIMS], go(Falling).pick(AIR_AIM), Nothing).beats(&[2]),
        // Row 3 for up or down held aiming Down, which beats the back row.
        /* 4 */
        row(&[], &[UD], go(Falling).pick(AIR_AIM), Nothing)
            .when(aimed(&[Down]))
            .beats(&[5]),
        /* 5 */ row(&[], &[B], go(Turning).back(), TurnRound).beats(&[2, 3]),
    ],
    otherwise: &[row(&[], &[], KEPT.aim(Forward), LoseDash)],
};

pub const CROUCHED: Chart = Chart {
    rows: &[
        /* 0 */ row(&[D], &[], go(Morphing).aim(Forward), Nothing),
        /* 1 */ row(&[U], &[], go(StandingUp).pick(CROUCH_AIM), Nothing).beats(&[0, 2, 3]),
        /* 2 */ row(&[J], &[], go(TakingOff).aim(Forward), TakeOff),
        /* 3 */
        row(&[B], &[], go(Turning).body(Body::Crouched).back(), Nothing).beats(&[0, 2]),
        /* 4 */
        row(
            &[],
            &[F, AIMS],
            go(Still).body(Body::Upright).aim(Forward),
            StandUp,
        ),
        /* 5 */
        row(
            &[],
            &[AU, AD],
            go(Still).body(Body::Crouched).aim(Up),
            Nothing,
        )
        .beats(&[4]),
        /* 6 */
        row(
            &[],
            &[F],
            go(Still).body(Body::Upright).aim(Forward),
            StandUp,
        ),
        /* 7 */
        row(
            &[],
            &[AIMS],
            go(Still).body(Body::Crouched).pick(AU_ELSE_AD),
            Nothing,
        ),
    ],
    otherwise: &[row(&[], &[], KEPT.aim(Forward), EndSkid)],
};

pub const BALL: Chart = Chart {
    rows: &[
        /* 0 */
        row(&[J], &[], go(Jumping).body_kept(), Hop).when(on_ground(true, Some(true))),
        /* 1 */ row(&[U], &[], go(Unmorphing).aim(Forward), Nothing).beats(&[0]),
        /* 2 */
        row(&[J], &[], go(Unmorphing).aim(Forward), Nothing).when(When {
            spring: Some(false),
            ..ALWAYS
        }),
        /* 3 */ row(&[], &[F], go(Rolling), Nothing).when(on_ground(true, None)),
        /* 4 */ row(&[], &[B], go(Rolling).back(), TurnRound).when(on_ground(true, None)),
        /* 5 */ row(&[], &[F], KEPT, Nothing).when(on_ground(false, None)),
        /* 6 */ row(&[], &[B], KEPT.back(), TurnRound).when(on_ground(false, None)),
    ],
    otherwise: &[
        row(&[], &[], go(Still), Halt).when(on_ground(true, None)),
        row(&[], &[], KEPT, AsRunning).when(on_ground(false, Some(false))),
        row(&[], &[], KEPT, Halt).when(on_ground(false, Some(true))),
    ],
};

pub const TURNING_ON_GROUND: Chart = Chart {
    rows: &[
        /* 0 */ row(&[J], &[F], go(Spinning), SpinStart).when(aimed(&[Up, DownForward])),
        /* 1 */ row(&[J], &[], go(TakingOff), TakeOff),
        /* 2 */
        row(&[], &[F, J], go(Spinning), SpinStart)
            .when(aimed(&[Forward]))
            .beats(&[1]),
        /* 3 */ row(&[], &[F], KEPT, Nothing),
    ],
    otherwise: &[row(&[], &[], KEPT, EndSkid)],
};

/// Turning in the air, crouched turning, or turning on the ground aimed `UpForward`.
pub const TURNING_KEPT: Chart = Chart {
    rows: &[row(&[], &[ANY], KEPT, Nothing)],
    otherwise: &[row(&[], &[], KEPT, EndSkid)],
};

pub const CHANGING_BODY: Chart = Chart {
    rows: &[],
    otherwise: &[],
};

/// Forward is the way she walks, back the way she looks.
pub const MOONWALKING: Chart = Chart {
    rows: &[
        /* 0 */ row(&[D], &[], go(Crouching).back().aim(Forward), Nothing).beats(&[1]),
        /* 1 */
        row(
            &[J],
            &[],
            go(Turning).back().aim(Forward),
            TurnRoundThenSpin,
        ),
        /* 2 */
        row(
            &[],
            &[F, S, AIMS],
            go(Moonwalking).pick(AimTo::Moonwalk),
            Nothing,
        ),
        /* 3 */ row(&[], &[F, S], go(Moonwalking).aim(Forward), Nothing),
        /* 4 */ row(&[], &[B], go(Running).back().aim(Forward), Nothing),
        /* 5 */ row(&[], &[F], go(Turning).back(), TurnRound),
    ],
    otherwise: &[row(&[], &[], go(Still).body(Body::Upright).back(), EndSkid)],
};

pub const WALL_JUMPING: Chart = Chart {
    rows: &[
        /* 0 */ row(&[D], &[], go(Morphing).aim(Forward), Nothing),
        /* 1 */ row(&[], &[S], go(Jumping).aim(Forward), Nothing).beats(&[4]),
        /* 2 */ row(&[], &[AIMS], go(Jumping).pick(AU_ELSE_AD), Nothing).beats(&[1, 4]),
        /* 3 */ row(&[], &[B], go(Spinning).back(), TurnRound).beats(&[1, 2, 4]),
        /* 4 */ row(&[], &[J], KEPT, Nothing),
    ],
    otherwise: &[row(&[], &[], go(Spinning), Halt)],
};

pub const AGAINST_WALL: Chart = Chart {
    rows: &[
        /* 0 */ row(&[D], &[], go(Crouching).aim(Forward), Nothing),
        /* 1 */ row(&[J], &[], go(TakingOff).aim(Forward), TakeOff).beats(&[0]),
        /* 2 */
        row(
            &[],
            &[B, AIMS],
            go(Moonwalking).back().pick(AD_ELSE_AU),
            Nothing,
        ),
        /* 3 */ row(&[], &[F, UD], go(Running).pick(U_ELSE_D), Nothing).beats(&[0]),
        /* 4 */ row(&[], &[F], go(Running).aim(Forward), Nothing),
        /* 5 */
        row(&[], &[U], go(Still).body(Body::Upright).aim(Up), Nothing).beats(&[6, 7]),
        /* 6 */
        row(
            &[],
            &[AIMS],
            go(Still).body(Body::Upright).pick(AU_ELSE_AD),
            Nothing,
        )
        .beats(&[4, 7]),
        /* 7 */ row(&[], &[B], go(Turning).back(), TurnRound),
    ],
    otherwise: &[row(&[], &[], KEPT.aim(Forward), EndSkid)],
};
