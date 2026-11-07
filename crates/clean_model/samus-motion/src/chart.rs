//! §7: the pose chart's reader.
//!
//! Her situation (`situation`) is the state machine's state; its moves are rows (`rows`): what
//! a row needs held, pressed or of her situation, the pose it leads to and its effects.
//! `PadChart`'s rule picks the one winning row of her situation; the rows of `WhatHerMovesMet`
//! and `AnimationEnds` (`table`) are read in order, the first that fits.

pub mod table;
#[cfg(test)]
mod tests;

use crate::rows::Test::{Aimed, Airborne, Any, Faces, Held, Moving, Not, Pressed, SpringBall};
use crate::rows::{Effect, Met, Row, Side, Situation, Test};
use clean_types::{Action, Buttons, Equipment, Pose};

/// What the chart reads.
///
/// Her situation and pose at the frame's start, whether her horizontal speed then isn't 0,
/// whether she is off the ground once her moves are made, the pad held and just pressed, her
/// equipment and what her moves met.
pub struct Moment {
    /// Her situation.
    pub situation: &'static Situation,
    /// Her pose.
    pub pose: Pose,
    /// Her horizontal speed at the frame's start isn't 0.
    pub moving: bool,
    /// Off the ground once her moves are made.
    pub airborne: bool,
    /// The buttons held.
    pub pad: Buttons,
    /// The buttons just pressed.
    pub pressed: Buttons,
    /// Her equipment.
    pub equipment: Equipment,
    /// What her moves met.
    pub met: Option<Met>,
}

/// A chosen pose, the change's effects, and whether it brings the aim change's step.
#[derive(Clone, Copy)]
pub struct Choice {
    /// The pose.
    pub pose: Pose,
    /// Its effects.
    pub effects: &'static [Effect],
    /// `AimingAndShooting`: a running pose other than hers, or one a row counts as chosen.
    pub steps: bool,
}

/// `PadChart`: a row that needs a just-pressed button outranks one that doesn't; then the one that
/// names more buttons.
fn rank(row: &Row) -> (bool, usize) {
    let needs = row.needs.iter();
    (
        needs.clone().any(|t| matches!(t, Pressed(_))),
        needs.filter(|t| matches!(t, Held(_) | Pressed(_))).count(),
    )
}

/// `PadChart`: whether row `a` beats row `b`: as `a`'s beats clause says, else by rank unless
/// `b`'s says it beats `a`.
fn beats(a: &Row, b: &Row) -> bool {
    a.beats.contains(&b.name) || !b.beats.contains(&a.name) && rank(a) > rank(b)
}

impl Moment {
    /// `PadChart`, `Aiming`, `WhatHerMovesMet`, `AnimationEnds`: whether one test of a row
    /// passes; forward and back are by her facing at the frame's start.
    fn passes(&self, test: &Test) -> bool {
        let p = self.pose;
        match *test {
            Held(b) => self.pad.seen(p.facing).any(b),
            Pressed(b) => self.pressed.seen(p.facing).any(b),
            Not(t) => !self.passes(t),
            Any(tests) => tests.iter().any(|t| self.passes(t)),
            Aimed(aims) => aims.contains(&p.aim),
            Test::Gun(gun) => p.gun == gun,
            Faces(side) => p.facing.side() == side,
            Test::Body(bodies) => bodies.contains(&p.body),
            Test::Action(actions) => actions.contains(&p.action),
            Test::Doing(doings) => doings.contains(&self.situation.doing),
            Airborne => self.airborne,
            SpringBall => self.equipment.spring_ball,
            Moving => self.moving,
            Test::Met(met) => self.met == Some(met),
        }
    }

    /// `PadChart`: whether a row's every test passes.
    fn fits(&self, tests: &[Test]) -> bool {
        tests.iter().all(|t| self.passes(t))
    }

    /// `PadChart`, `Aiming`: a row's choice, the pose it leads to from hers and its effects;
    /// `AimingAndShooting`: a running pose other than hers, or her running pose a row counts as
    /// chosen, brings the step.
    fn choice(&self, row: &Row) -> Choice {
        let (p, n) = (self.pose, row.next);
        let pose = Pose {
            body: n.body.unwrap_or(p.body),
            action: n.action.unwrap_or(p.action),
            facing: match n.facing {
                Side::Kept => p.facing,
                Side::Back => p.facing.back(),
            },
            aim: n
                .aim
                .iter()
                .find(|a| self.fits(a.when))
                .map_or(p.aim, |a| a.aim),
            gun: n.gun.unwrap_or(p.gun),
            legs: n.legs.unwrap_or(p.legs),
        };
        Choice {
            pose,
            effects: row.effects,
            steps: pose.action == Action::Running && (pose != p || row.steps),
        }
    }

    /// `PadChart`: the one row of her situation that fits and no other fitting row beats (the
    /// chart's test proves there is at most one), else its last row.
    fn pad_chart(&self) -> &'static Row {
        let s = self.situation;
        let fit: Vec<&'static Row> = s.rows.iter().filter(|r| self.fits(r.needs)).collect();
        fit.iter()
            .find(|a| !fit.iter().any(|b| beats(b, a)))
            .map_or(&s.last, |r| r)
    }

    /// `PoseFollows`: an animation ending hands over its pose (`AnimationEnds`), else what her
    /// moves met chooses (`WhatHerMovesMet`), else the pad chart (`PadChart`). The aim change's
    /// step follows from the second and third alone (`AimingAndShooting`).
    #[must_use]
    pub fn choose(&self, animation_ends: bool) -> Choice {
        let first = |rows: &'static [Row]| rows.iter().find(|r| self.fits(r.needs));
        let by_moves = self.choice(first(table::MOVES_MET).unwrap_or_else(|| self.pad_chart()));
        first(table::ANIMATION_ENDS)
            .filter(|_| animation_ends)
            .map_or(by_moves, |r| Choice {
                steps: by_moves.steps,
                ..self.choice(r)
            })
    }
}
