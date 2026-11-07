//! §7: the pose chart. Which pose she shows next, from her pose at the frame's start, the pad and
//! what her moves met, and what the change does (`PoseFollows`). The pad chart is data: the rows
//! of `table`, read by the one rule of `PadChart` that picks the winning row.

mod table;

use clean_types::{Action, Aim, Body, Button, Facing, Pad, Pose, Side};

/// `PadChart`: buttons as the chart names them; forward and back are by her facing at the frame's
/// start.
type Keys = u16;
/// Forward.
const F: Keys = 1;
/// Back.
const B: Keys = 1 << 1;
/// Jump.
const J: Keys = 1 << 2;
/// Shoot.
const S: Keys = 1 << 3;
/// Run.
const RUN: Keys = 1 << 4;
/// Up.
const U: Keys = 1 << 5;
/// Down.
const D: Keys = 1 << 6;
/// Aim up.
const AU: Keys = 1 << 7;
/// Aim down.
const AD: Keys = 1 << 8;

/// `AnimationEnds`: where a turn started, which says what its end hands over.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Turn {
    /// On the ground (or crouched, by her body).
    Ground,
    /// In a jump.
    InJump,
    /// In a fall.
    InFall,
    /// The moonwalk's jump: it ends in a spin.
    MoonwalkJump,
}

/// What the chart reads besides the pad: her pose at the frame's start, whether she is off the
/// ground once her moves are made, and where a turn started.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Situation {
    /// Her pose at the frame's start.
    pub pose: Pose,
    /// Off the ground.
    pub airborne: bool,
    /// Where her turn started, when she is turning.
    pub turn: Turn,
}

/// `WhatHerMovesMet`: what a frame's floor, ceiling or kick met. At most one meets her a frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Contact {
    /// A floor she lands on (`Landing`).
    Floor,
    /// A floor the ball bounces or the spring ball hops from (`Landing`).
    Bounce,
    /// No floor: she walks off a ledge (`FollowingTheGround`).
    NoFloor,
    /// A ceiling that ends her rise (`WallsAndCeilings`).
    Ceiling,
    /// The wall-jump kick (`WallJump`).
    WallKick,
}

/// `WhatHerMovesMet`: what a frame's moves met.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Met {
    /// A wall stopped her horizontal move (`WallsAndCeilings`).
    pub wall: bool,
    /// A block or a solid enemy was in the way of the aim change's step (`AimingAndShooting`).
    pub aim_step_blocked: bool,
    /// What her vertical move or her kick met.
    pub contact: Option<Contact>,
}

/// What a pose change does to her speeds and dash, applied after the frame's moves.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effect {
    /// None (what her moves met already acted in the moves).
    Nothing,
    /// `Jumping`'s take-off: she starts rising; from a crouch without aim she is lifted.
    TakeOff,
    /// `Jumping`'s spin start: she starts rising.
    SpinStart,
    /// `Jumping`'s spring-ball hop: she starts rising.
    Hop,
    /// `TurningRound`: she skids the old way, her dash joining her speed.
    TurnRound,
    /// `TurningRound`, for the moonwalk's jump: the turn ends in a spin (`AnimationEnds`).
    TurnRoundThenSpin,
    /// `TurningRound`: she follows the pad again; a skid ends along her new facing.
    EndSkid,
    /// The let-go outcome "as running": her dash joins her speed and she carries on, braking.
    AsRunning,
    /// `Dash`: she loses her dash.
    LoseDash,
    /// She stands at once from a crouch, lifted.
    StandUp,
    /// Horizontal speed 0, dash 0, following the pad.
    Halt,
    /// `Moonfall`: a spin entered as a turn ends, with no rise.
    Moonfall,
}

/// The pose a frame ends in, and what the change does.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Choice {
    /// The next pose.
    pub pose: Pose,
    /// Its effect.
    pub effect: Effect,
}

/// A row's condition on her situation besides the pad.
#[derive(Clone, Copy)]
struct When {
    /// Her aim at the frame's start is one of these; empty for any.
    aims: &'static [Aim],
    /// On the ground (`true`) or off it.
    ground: Option<bool>,
    /// In the spring ball (`true`) or not.
    spring: Option<bool>,
}

/// How a next pose's body follows.
#[derive(Clone, Copy)]
enum BodyTo {
    /// As its action says (§7: body follows the action), else kept.
    Natural,
    /// Kept.
    Kept,
    /// This body.
    Set(Body),
}

/// How a next pose's facing follows.
#[derive(Clone, Copy)]
enum FaceTo {
    /// Kept.
    Kept,
    /// Back: the other side.
    Back,
}

/// `Aiming`: how a next pose's aim follows from the held buttons.
#[derive(Clone, Copy)]
enum AimTo {
    /// Kept.
    Kept,
    /// This aim.
    Set(Aim),
    /// The aim of the first entry whose buttons are all held, else the last.
    Pick(&'static [(Keys, Aim)], Aim),
    /// The moonwalk's: `UpForward` with aim up, else `DownForward`; with both aim buttons held,
    /// `UpForward` facing left and `DownForward` facing right.
    Moonwalk,
}

/// A next pose as a row names it.
#[derive(Clone, Copy)]
struct To {
    /// Her action; `None` keeps her pose.
    action: Option<Action>,
    body: BodyTo,
    facing: FaceTo,
    aim: AimTo,
}

/// `PadChart`: one row: what it needs, the next pose and the effect, and the rows of its
/// situation it beats against the rank.
#[derive(Clone, Copy)]
struct Row {
    when: When,
    /// Each a set of buttons of which one must have been just pressed.
    pressed: &'static [Keys],
    /// Each a set of buttons of which one must be held.
    held: &'static [Keys],
    /// Buttons that must not be held.
    without: Keys,
    to: To,
    effect: Effect,
    /// Indices of the rows of its situation it beats.
    beats: &'static [usize],
}

/// The rows of one situation, and its let-go outcomes, of which the first fitting applies when
/// no row fits.
struct Chart {
    rows: &'static [Row],
    otherwise: &'static [Row],
}

/// `PadChart`: the buttons of `pad`, forward and back by `side`.
fn keys(pad: Pad, side: Side) -> Keys {
    let (forward, back) = match side {
        Side::Left => (Button::Left, Button::Right),
        Side::Right => (Button::Right, Button::Left),
    };
    [
        (forward, F),
        (back, B),
        (Button::Jump, J),
        (Button::Shoot, S),
        (Button::Run, RUN),
        (Button::Up, U),
        (Button::Down, D),
        (Button::AimUp, AU),
        (Button::AimDown, AD),
    ]
    .into_iter()
    .filter(|&(button, _)| pad.held(button))
    .fold(0, |all, (_, key)| all | key)
}

/// The other side's facing.
const fn back(facing: Facing) -> Facing {
    match facing {
        Facing::Left => Facing::Right,
        Facing::Right | Facing::Screen => Facing::Left,
    }
}

/// §7: the body an action has; `None` where the row says it or it is kept.
const fn natural_body(action: Action) -> Option<Body> {
    match action {
        Action::Running
        | Action::Moonwalking
        | Action::TakingOff
        | Action::Jumping
        | Action::Spinning
        | Action::Falling
        | Action::WallJumping
        | Action::AgainstWall
        | Action::StandingUp => Some(Body::Upright),
        Action::Crouching | Action::Unmorphing => Some(Body::Crouched),
        Action::Morphing => Some(Body::Ball),
        Action::Still | Action::Turning | Action::Rolling => None,
    }
}

impl When {
    fn holds(self, s: Situation) -> bool {
        (self.aims.is_empty() || self.aims.contains(&s.pose.aim))
            && self.ground.is_none_or(|ground| ground != s.airborne)
            && self
                .spring
                .is_none_or(|spring| spring == (s.pose.body == Body::SpringBall))
    }
}

impl To {
    /// The pose this names, from `from` with `held` held.
    fn pose(self, from: Pose, held: Keys) -> Pose {
        let body = match self.body {
            BodyTo::Natural => self.action.and_then(natural_body).unwrap_or(from.body),
            BodyTo::Kept => from.body,
            BodyTo::Set(body) => body,
        };
        let facing = match self.facing {
            FaceTo::Kept => from.facing,
            FaceTo::Back => back(from.facing),
        };
        let all = |k: Keys| held & k == k;
        let aim = match self.aim {
            AimTo::Kept => from.aim,
            AimTo::Set(aim) => aim,
            AimTo::Pick(list, other) => list
                .iter()
                .find(|&&(k, _)| all(k))
                .map_or(other, |&(_, aim)| aim),
            AimTo::Moonwalk if all(AU | AD) && facing == Facing::Left => Aim::UpForward,
            AimTo::Moonwalk if all(AU) && !all(AD) => Aim::UpForward,
            AimTo::Moonwalk => Aim::DownForward,
        };
        Pose {
            body,
            action: self.action.unwrap_or(from.action),
            facing,
            aim,
        }
    }
}

impl Row {
    fn fits(&self, s: Situation, held: Keys, pressed: Keys) -> bool {
        self.when.holds(s)
            && self.pressed.iter().all(|&k| pressed & k != 0)
            && self.held.iter().all(|&k| held & k != 0)
            && held & self.without == 0
    }

    /// `PadChart`: a row that needs a just-pressed button outranks one that doesn't; then the
    /// one that names more buttons.
    const fn rank(&self) -> (bool, usize) {
        (
            !self.pressed.is_empty(),
            self.pressed.len().saturating_add(self.held.len()),
        )
    }
}

/// `PadChart`: whether row `a` beats row `b`: as its cell says, else by rank unless `b`'s cell
/// says it beats `a`.
fn beats((ia, a): (usize, &Row), (ib, b): (usize, &Row)) -> bool {
    a.beats.contains(&ib) || (!b.beats.contains(&ia) && a.rank() > b.rank())
}

/// The chart for her situation (§7's "She is").
fn chart(s: Situation) -> &'static Chart {
    let p = s.pose;
    if p.action.changes_shape() {
        return &table::CHANGING_BODY;
    }
    if p.body.is_ball() {
        return &table::BALL;
    }
    match p.action {
        Action::Turning
            if p.body == Body::Crouched
                || p.aim == Aim::UpForward
                || matches!(s.turn, Turn::InJump | Turn::InFall) =>
        {
            &table::TURNING_KEPT
        }
        Action::Turning => &table::TURNING_ON_GROUND,
        Action::Still if p.body == Body::Crouched => &table::CROUCHED,
        Action::Still => &table::STANDING,
        Action::Running => &table::RUNNING,
        Action::TakingOff => &table::TAKING_OFF,
        Action::Jumping => &table::STRAIGHT_JUMP,
        Action::Spinning => &table::SPINNING,
        Action::Falling => &table::FALLING,
        Action::Moonwalking => &table::MOONWALKING,
        Action::WallJumping => &table::WALL_JUMPING,
        Action::AgainstWall => &table::AGAINST_WALL,
        Action::Rolling => &table::BALL,
        Action::Crouching | Action::StandingUp | Action::Morphing | Action::Unmorphing => {
            &table::CHANGING_BODY
        }
    }
}

/// `PadChart`: the pad chart's outcome, from her pose and the pad at the frame's start: the
/// fitting row no other fitting row beats, else her let-go outcome; with none, she keeps her pose.
#[must_use]
pub fn pad_chart(s: Situation, pad: Pad, pressed: Pad) -> Choice {
    let side = s.pose.facing.side();
    let (held, pressed) = (keys(pad, side), keys(pressed, side));
    let chart = chart(s);
    let fitting: Vec<(usize, &Row)> = chart
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.fits(s, held, pressed))
        .collect();
    let winner = fitting
        .iter()
        .find(|&&a| !fitting.iter().any(|&b| beats(b, a)))
        .map(|&(_, row)| row)
        .or_else(|| {
            chart
                .otherwise
                .iter()
                .find(|row| row.fits(s, held, pressed))
        });
    winner.map_or(
        Choice {
            pose: s.pose,
            effect: Effect::Nothing,
        },
        |row| Choice {
            pose: row.to.pose(s.pose, held),
            effect: row.effect,
        },
    )
}

/// `AnimationEnds`: the pose an ending animation hands over, whatever the pad.
#[must_use]
pub const fn handed_over(s: Situation) -> Option<Choice> {
    let p = s.pose;
    let (action, body, effect) = match (p.action, p.body, s.turn) {
        (Action::TakingOff, _, _) => (Action::Jumping, p.body, Effect::Nothing),
        (Action::Turning, Body::Crouched, _) => (Action::Still, Body::Crouched, Effect::LoseDash),
        (Action::Turning, _, Turn::Ground) => (Action::Still, Body::Upright, Effect::LoseDash),
        (Action::Turning, _, Turn::InJump) => (Action::Jumping, Body::Upright, Effect::LoseDash),
        (Action::Turning, _, Turn::InFall) => (Action::Falling, Body::Upright, Effect::LoseDash),
        (Action::Turning, _, Turn::MoonwalkJump) => {
            (Action::Spinning, Body::Upright, Effect::Moonfall)
        }
        (Action::StandingUp, _, _) => (Action::Still, Body::Upright, Effect::Nothing),
        (Action::Morphing, _, _) if s.airborne => (Action::Falling, Body::Ball, Effect::Nothing),
        (Action::Morphing, _, _) => (Action::Still, Body::Ball, Effect::Nothing),
        (Action::Unmorphing, _, _) if s.airborne => {
            (Action::Falling, Body::Upright, Effect::Nothing)
        }
        (Action::Crouching | Action::Unmorphing, _, _) => {
            (Action::Still, Body::Crouched, Effect::Nothing)
        }
        (
            Action::AgainstWall
            | Action::Falling
            | Action::Jumping
            | Action::Moonwalking
            | Action::Rolling
            | Action::Running
            | Action::Spinning
            | Action::Still
            | Action::WallJumping,
            _,
            _,
        ) => return None,
    };
    Some(Choice {
        pose: Pose { action, body, ..p },
        effect,
    })
}

/// `WhatHerMovesMet`: the pose what her moves met chooses over the pad, if it does; `chosen` is
/// the pad chart's pose, whose aim a blocked aim step keeps. Its effects acted in the moves.
#[must_use]
pub const fn met_pose(s: Situation, met: Met, chosen: Pose) -> Option<Choice> {
    let p = s.pose;
    let running = matches!(p.action, Action::Running);
    let upright = matches!(p.body, Body::Upright);
    let (action, body, aim) = if met.wall && running {
        match p.aim {
            Aim::Up => (Action::Still, Body::Upright, Aim::Up),
            Aim::UpForward | Aim::DownForward => (Action::AgainstWall, Body::Upright, p.aim),
            Aim::Forward | Aim::Down => (Action::AgainstWall, Body::Upright, Aim::Forward),
        }
    } else if met.aim_step_blocked && running {
        (Action::AgainstWall, Body::Upright, chosen.aim)
    } else {
        match (met.contact, p.action) {
            (Some(Contact::WallKick), Action::Spinning) => (Action::WallJumping, p.body, p.aim),
            (Some(Contact::Floor), _) if p.body.is_ball() => (Action::Still, p.body, p.aim),
            (Some(Contact::Bounce | Contact::Ceiling), _) => (p.action, p.body, p.aim),
            (Some(Contact::Floor), Action::Jumping | Action::Falling) if upright => {
                let aim = if matches!(p.aim, Aim::Down) {
                    Aim::Forward
                } else {
                    p.aim
                };
                (Action::Still, Body::Upright, aim)
            }
            (Some(Contact::Floor), Action::Spinning | Action::WallJumping) => {
                (Action::Still, Body::Upright, Aim::Forward)
            }
            (Some(Contact::NoFloor), _) if p.body.is_ball() => (Action::Falling, p.body, p.aim),
            (Some(Contact::NoFloor), _) if upright => {
                (Action::Falling, Body::Upright, Aim::Forward)
            }
            _ => return None,
        }
    };
    Some(Choice {
        pose: Pose {
            body,
            action,
            facing: p.facing,
            aim,
        },
        effect: Effect::Nothing,
    })
}

/// `PoseFollows`: the pose a frame ends in: an animation ending hands over its pose, else what
/// her moves met chooses, else the pad chart's outcome `chart`.
#[must_use]
pub fn next_pose(s: Situation, chart: Choice, met: Met, animation_ends: bool) -> Choice {
    animation_ends
        .then(|| handed_over(s))
        .flatten()
        .or_else(|| met_pose(s, met, chart.pose))
        .unwrap_or(chart)
}
