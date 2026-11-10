//! §7: the pose chart's reader.
//!
//! Her situation (`situation`) is the state machine's state; its moves are rows (`rows`): what
//! a row needs held, pressed or of her situation, the pose it leads to and its effects.
//! `PadChart`'s rule picks the one winning row of her situation; the rows of `WhatHerMovesMet`
//! and `AnimationEnds` (`table`) are read in order, the first that fits.

pub mod table;
#[cfg(test)]
mod tests;

use crate::rows::Test::{
    Action, Aimed, Airborne, Body, Doing, Faces, Gun, Held, Moving, Not, Pressed, SpringBall,
};
use crate::rows::{Effect, Met, Row, Situation, Test, seen};
use crate::rules::Lowers;
use crate::situation::classify;
use crate::tick::Frame;
use clean_types::Action::Running;
use clean_types::button::JUMP;
use clean_types::{Buttons, Item, Pose, Side};

/// What the chart reads of a frame besides her pose.
///
/// The frame (its pad held and just pressed, the items switched on, an animation's end), whether
/// her horizontal speed at its start isn't 0, whether she is off the ground once her moves are
/// made, whether jump held through counts as a press, and what her moves met.
pub struct Moment<'a> {
    /// Her horizontal speed at the frame's start isn't 0.
    pub moving: bool,
    /// Off the ground once her moves are made.
    pub airborne: bool,
    /// `Jumping`, jump held through: jump counts as just pressed for the pad chart this frame.
    pub jump_held_through: bool,
    /// The frame.
    pub frame: &'a Frame<'a>,
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
    /// `ShrinkingLowersHer`: the look down the pose starts with.
    pub lower: Option<Lowers>,
}

impl Choice {
    /// `PoseFollows`, `PoseFitAfterMoves`, `AimingAndShooting`: a pose she ends in instead of the
    /// one chosen (the pose she had, where the change is refused; crouched `Still`; against the
    /// wall, where the aim change's step is blocked), which brings none of the chosen pose's
    /// effects: no step, no lowering and no effect on her speeds and dash.
    #[must_use]
    pub const fn instead(pose: Pose) -> Self {
        Self {
            pose,
            effects: &[],
            steps: false,
            lower: None,
        }
    }
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

impl Moment<'_> {
    /// `PoseFollows`: the pose she takes from `pose`, hers at the frame's start, at this moment:
    /// an animation's end hands over its pose, else what her moves met chooses, else the pad
    /// chart's. What the pose then does, pose fit and its effects, is the step's.
    #[must_use]
    pub fn next_pose(&self, pose: Pose) -> Pose {
        self.choose(pose, classify(pose)).pose
    }

    /// `PoseFollows`: the choice from `pose` in its `situation`, with its effects.
    pub(crate) fn choose(&self, pose: Pose, situation: &'static Situation) -> Choice {
        Reading {
            moment: self,
            pose,
            situation,
        }
        .choose()
    }
}

/// `PadChart`: the chart read at a moment for one pose, hers at the frame's start, and the
/// situation it puts her in.
struct Reading<'a> {
    /// The moment.
    moment: &'a Moment<'a>,
    /// Her pose.
    pose: Pose,
    /// Her situation.
    situation: &'static Situation,
}

impl Reading<'_> {
    /// `PadChart`: what row `a`'s beats clauses say of row `b`: that `a` beats it, or, under a
    /// clause's condition, that `b` beats `a`; nothing where they don't name it.
    fn says(&self, a: &Row, b: &Row) -> Option<bool> {
        if a.beats.contains(&b.id) {
            return Some(true);
        }
        let clause = a.beats_unless.iter().find(|c| c.row == b.id)?;
        Some(!clause.unless.iter().any(|tests| self.fits(tests)))
    }

    /// `PadChart`: whether row `a` beats row `b`: as `a`'s beats clauses say, else as `b`'s say
    /// the other way, else by rank.
    fn beats(&self, a: &Row, b: &Row) -> bool {
        self.says(a, b)
            .or_else(|| self.says(b, a).map(|b_wins| !b_wins))
            .unwrap_or_else(|| rank(a) > rank(b))
    }

    /// `PadChart`, `Jumping` (jump held through): the buttons the chart reads as just pressed:
    /// the pad's, and jump where jump held through counts as a press.
    const fn pressed(&self) -> Buttons {
        let pad = self.moment.frame.pressed;
        if self.moment.jump_held_through {
            Buttons(pad.0 | JUMP.0)
        } else {
            pad
        }
    }

    /// `PadChart`, `Aiming`, `WhatHerMovesMet`, `AnimationEnds`: whether one test of a row
    /// passes; forward and back are by her facing at the frame's start.
    fn passes(&self, test: &Test) -> bool {
        let (p, m) = (self.pose, self.moment);
        match *test {
            Held(b) => seen(m.frame.pad, p.facing).any(b),
            Pressed(b) => seen(self.pressed(), p.facing).any(b),
            Not(t) => !self.passes(t),
            Aimed(aims) => aims.contains(&p.aim),
            Gun(gun) => p.gun == gun,
            Faces(side) => Side::of(p.facing) == side,
            Body(bodies) => bodies.contains(&p.body),
            Action(actions) => actions.contains(&p.action),
            Doing(doings) => doings.contains(&self.situation.doing),
            Airborne => m.airborne,
            SpringBall => m.frame.items[Item::SpringBall],
            Moving => m.moving,
            Test::Met(met) => m.met == Some(met),
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
        let aim = row.next.aim.iter().find(|a| self.fits(a.when));
        let pose = row.next.pose_from(self.pose, aim.map(|a| a.aim));
        Choice {
            pose,
            effects: row.effects,
            steps: pose.action == Running && (pose != self.pose || row.steps),
            lower: row.next.lower,
        }
    }

    /// `PadChart`: the one row of her situation that fits and no other fitting row beats (the
    /// chart's test proves there is at most one), else its last row.
    fn pad_chart(&self) -> &'static Row {
        let s = self.situation;
        let fit: Vec<&'static Row> = s.rows.iter().filter(|r| self.fits(r.needs)).collect();
        fit.iter()
            .find(|row| !fit.iter().any(|other| self.beats(other, row)))
            .map_or(&s.last, |r| r)
    }

    /// `WhatHerMovesMet`, `AnimationEnds`: the first of a table's rows that fits.
    fn first(&self, rows: &'static [Row]) -> Option<&'static Row> {
        rows.iter().find(|r| self.fits(r.needs))
    }

    /// `PoseFollows`: an animation ending hands over its pose (`AnimationEnds`), else what her
    /// moves met chooses (`WhatHerMovesMet`), else the pad chart (`PadChart`). The aim change's
    /// step follows from the second and third alone (`AimingAndShooting`).
    fn choose(&self) -> Choice {
        let by_moves = self
            .first(table::MOVES_MET)
            .unwrap_or_else(|| self.pad_chart());
        let by_moves = self.choice(by_moves);
        self.first(table::ANIMATION_ENDS)
            .filter(|_| self.moment.frame.animation_ends)
            .map_or(by_moves, |r| Choice {
                steps: by_moves.steps,
                ..self.choice(r)
            })
    }
}
