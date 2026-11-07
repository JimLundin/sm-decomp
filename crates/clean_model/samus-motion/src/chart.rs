//! §7: the pose chart. Which pose she shows next, from her pose at the frame's start, the pad and
//! what her moves met, and what the change does (`PoseFollows`). The pad chart is data, the rows
//! of `table`, read by `PadChart`'s one rule that picks the winning row.

pub mod table;
#[cfg(test)]
mod tests;

use clean_types::Action::{
    self, AgainstWall, Crouching, Falling, Jumping, Landing, Morphing, Running, Spinning,
    StandingUp, Still, TakingOff, Turning, Unmorphing, WallJumping,
};
use clean_types::Aim::{Down, DownForward, Forward, Up, UpForward};
use clean_types::Body::{Ball, Crouched, SpringBall, Upright};
use clean_types::{Equipment, Facing, Gun, LandedFrom, Legs, Pad, Pose, Turn};
use table::{
    ASWAS, BACK, DRAW, LOSEDASH, MOONFALL, NAMED, PRESSED, Row, STANDUP, STRIDE, keys, rows,
};

/// What the chart reads: her pose at the frame's start, whether she is off the ground once her
/// moves are made, the pad held and just pressed, and her equipment.
#[derive(Clone, Copy)]
pub struct Situation {
    /// Her pose at the frame's start.
    pub pose: Pose,
    /// Off the ground once her moves are made.
    pub airborne: bool,
    /// The buttons held.
    pub pad: Pad,
    /// The buttons just pressed.
    pub pressed: Pad,
    /// Her equipment.
    pub equipment: Equipment,
}

/// `WhatHerMovesMet`: what this frame's moves met, in its table's order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Met {
    /// A wall stop (`WallsAndCeilings`).
    Wall,
    /// A block or solid enemy in the way of the aim change's step (`AimingAndShooting`).
    AimStep,
    /// A wall near her as she presses jump (`WallJump`'s kick).
    WallKick,
    /// A floor (`Landing`).
    Floor,
    /// No floor: she walks off a ledge (`FollowingTheGround`).
    NoFloor,
    /// A ceiling (`WallsAndCeilings`).
    Ceiling,
}

/// A next pose, and the change's effects on her speeds and dash (`table`'s effect flags).
pub type Choice = (Pose, u16);

/// Whether her body is changing shape.
#[must_use]
pub const fn changes_shape(a: Action) -> bool {
    matches!(a, Crouching | StandingUp | Morphing | Unmorphing)
}

/// `PadChart`: a row that needs a just-pressed button outranks one that doesn't; then the one that
/// names more buttons.
const fn rank(r: &Row) -> (bool, u32) {
    (r.0 & PRESSED != 0, (r.0 & NAMED).count_ones())
}

/// `PadChart`: whether row `a`, at `i` in its situation, beats row `b`, at `j`: as `a`'s Pad cell
/// says, else by rank unless `b`'s says it beats `a`.
fn beats((a, i): (&Row, u32), (b, j): (&Row, u32)) -> bool {
    let says = |r: &Row, n: u32| r.4.checked_shr(n).is_some_and(|m| m & 1 != 0);
    says(a, j) || !says(b, i) && rank(a) > rank(b)
}

/// `PadChart`: the rows of her situation that fit and no other fitting row beats.
fn winners(s: &Situation) -> (Vec<&'static Row>, u32) {
    let k = keys(s);
    let fit: Vec<(&Row, u32)> = rows(s.pose)
        .iter()
        .zip(0..)
        .filter(|(r, _)| r.0 & !k == 0)
        .collect();
    let won = fit.iter().filter(|&&a| !fit.iter().any(|&b| beats(b, a)));
    (won.map(|w| w.0).collect(), k)
}

/// `PadChart`: the pad chart's outcome, from her pose and the pad at the frame's start; where no
/// row fits she keeps her pose.
#[must_use]
pub fn pad_chart(s: &Situation) -> Choice {
    let (won, k) = winners(s);
    won.first()
        .map_or((s.pose, 0), |r| (outcome(r, s.pose, k), r.3))
}

/// `PadChart`: the next pose a row names, from `p` with keys `k`. "Kept" (no action) keeps her
/// pose, gun and legs; else the gun is stowed unless drawn or as it was, the legs together unless
/// striding, and the body follows the action. Facing is kept unless back; the aim is the first
/// of the row's `Aiming` rule whose keys are all there, kept where none is; a turn in the air
/// shows `DownForward` as `Down` (§2).
fn outcome(&(_, act, aims, fx, _): &Row, p: Pose, k: u32) -> Pose {
    let on = |f: u16| fx & f != 0;
    let action = act.unwrap_or(p.action);
    let aim = aims.iter().find(|a| a.0 & !k == 0).map_or(p.aim, |a| a.1);
    Pose {
        body: if on(STANDUP) || matches!(action, StandingUp | TakingOff) {
            Upright
        } else if matches!(action, Crouching | Unmorphing) {
            Crouched
        } else if action == Morphing {
            Ball
        } else {
            p.body
        },
        action,
        facing: match p.facing {
            Facing::Left if on(BACK) => Facing::Right,
            Facing::Right | Facing::Screen if on(BACK) => Facing::Left,
            f @ (Facing::Left | Facing::Right | Facing::Screen) => f,
        },
        aim: if matches!(action, Turning(Turn::Jump | Turn::Fall)) && aim == DownForward {
            Down
        } else {
            aim
        },
        gun: if on(DRAW) || (on(ASWAS) || act.is_none()) && p.gun == Gun::Drawn {
            Gun::Drawn
        } else {
            Gun::Stowed
        },
        legs: if on(STRIDE) || act.is_none() && p.legs == Legs::Striding {
            Legs::Striding
        } else {
            Legs::Together
        },
    }
}

/// `WhatHerMovesMet`: the pose what her moves met chooses over the pad; `chosen` is the pad
/// chart's, whose aim a blocked aim step keeps. Body and facing are kept, the legs together.
fn met_pose(s: &Situation, met: Met, chosen: Pose) -> Option<Pose> {
    let p = s.pose;
    let ball = matches!(p.body, Ball | SpringBall);
    let (action, aim, gun) = match met {
        Met::Wall if p.aim == Up => (Still, Up, Gun::Stowed),
        Met::Wall if matches!(p.aim, UpForward | DownForward) => (AgainstWall, p.aim, Gun::Stowed),
        Met::Wall => (AgainstWall, Forward, Gun::Stowed),
        Met::AimStep => (AgainstWall, chosen.aim, Gun::Stowed),
        Met::WallKick => (WallJumping, Forward, Gun::Stowed),
        // The ball bouncing (now off the ground again) is kept; coming to rest it is still.
        Met::Floor if ball && s.airborne => return Some(p),
        Met::Floor if ball => (Still, Forward, Gun::Stowed),
        // Aimed `Forward` with shoot held, she lands firing.
        Met::Floor if p.body == Upright && matches!(p.action, Jumping | Falling) => (
            Landing(LandedFrom::Jump),
            if p.aim == Down { Forward } else { p.aim },
            if p.aim == Forward && s.pad.shoot {
                Gun::Drawn
            } else {
                Gun::Stowed
            },
        ),
        Met::Floor if matches!(p.action, Spinning | WallJumping) => {
            (Landing(LandedFrom::Spin), Forward, Gun::Stowed)
        }
        Met::NoFloor if ball || p.body == Upright => (Falling, Forward, Gun::Stowed),
        Met::Ceiling => return Some(p),
        Met::Floor | Met::NoFloor => return None,
    };
    Some(Pose {
        action,
        aim,
        gun,
        legs: Legs::Together,
        ..p
    })
}

/// `AnimationEnds`: the pose an ending animation hands over, whatever the pad, and its effect.
/// The gun is stowed; the aim kept where the table says so, else `Forward`.
fn handed_over(s: &Situation) -> Option<Choice> {
    let p = s.pose;
    let ball = if s.equipment.spring_ball {
        SpringBall
    } else {
        Ball
    };
    let (action, body, kept, fx) = match p.action {
        TakingOff => (Jumping, Upright, true, 0),
        Turning(_) if p.body == Crouched => (Still, Crouched, true, LOSEDASH),
        Turning(Turn::Ground) => (Still, Upright, true, LOSEDASH),
        Turning(Turn::Jump) => (Jumping, Upright, true, LOSEDASH),
        Turning(Turn::Fall) => (Falling, Upright, true, LOSEDASH),
        Turning(Turn::MoonwalkJump) => (Spinning, Upright, false, MOONFALL),
        StandingUp => (Still, Upright, false, 0),
        Morphing if s.airborne => (Falling, ball, false, 0),
        Morphing => (Still, ball, false, 0),
        Landing(_) => (Still, Upright, true, 0),
        Unmorphing if s.airborne => (Falling, Upright, false, 0),
        Crouching | Unmorphing => (Still, Crouched, false, 0),
        AgainstWall
        | Falling
        | Jumping
        | Action::Moonwalking
        | Action::Rolling
        | Running
        | Spinning
        | Still
        | WallJumping => return None,
    };
    // A turn in a jump aimed `Forward` ends striding.
    let striding = p.action == Turning(Turn::Jump) && p.aim == Forward;
    let pose = Pose {
        body,
        action,
        aim: if kept { p.aim } else { Forward },
        gun: Gun::Stowed,
        legs: if striding {
            Legs::Striding
        } else {
            Legs::Together
        },
        ..p
    };
    Some((pose, fx))
}

/// `PoseFollows`: the pose a frame ends in, and its effects: an animation ending hands over its
/// pose, else what her moves met chooses (its effects acted in the moves), else `chart`, the pad
/// chart's outcome.
#[must_use]
pub fn next_pose(s: &Situation, chart: Choice, met: Option<Met>, animation_ends: bool) -> Choice {
    animation_ends
        .then(|| handed_over(s))
        .flatten()
        .or_else(|| met.and_then(|m| met_pose(s, m, chart.0)).map(|p| (p, 0)))
        .unwrap_or(chart)
}
