//! §7: the pose chart's reader.
//!
//! Her situation (`situation`) is the state machine's state; its moves are rows (`rows`): what
//! a row needs held, pressed or of her situation, the pose it leads to and its effects.
//! `PadChart`'s rule picks the one winning row of her situation; the rows of `WhatHerMovesMet`
//! and `AnimationEnds` (`table`) are read in order, the first that fits.

pub mod table;
#[cfg(test)]
mod tests;

use crate::rows::{Effect, Met, Row, Situation, Test, seen};
use crate::rules::Lowers;
use crate::situation::{UnknownPose, classify};
use types::{Action, Aim, Buttons, ClipEnd, Fixed, Item, Pose, Set, Side, ZERO};

/// What the chart reads of a frame besides her pose.
///
/// The buttons held and those just pressed, the items switched on, the end her animation
/// reaches, her horizontal speed at the frame's start, whether she is off the ground once her
/// moves are made, and what they met.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Moment {
    /// The buttons held.
    pub pad: Buttons,
    /// The buttons the chart reads as just pressed: the pad's, and jump where jump held through
    /// counts as a press (`Jumping`).
    pub pressed: Buttons,
    /// The items switched on (§1.3): the chart reads the spring ball (`PadChart`,
    /// `AnimationEnds`), and the screw attack and space jump (`PadChart`'s wall jump let-go).
    pub items: Set<Item>,
    /// `AnimationEnds`: the end her pose's animation reaches this frame, if any: an end, or an
    /// end unless a jump.
    pub animation_ends: Option<ClipEnd>,
    /// Her horizontal speed at the frame's start.
    pub speed: Fixed,
    /// Off the ground once her moves are made.
    pub airborne: bool,
    /// What her moves met.
    pub met: Option<Met>,
}

/// A chosen pose: the pose, what the change does, whether it brings the aim change's step,
/// and whether her animation's end handed it over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    /// The pose chosen.
    pub pose: Pose,
    /// What the change does to her speeds, dash and position, in order.
    pub effects: &'static [Effect],
    /// `AimingAndShooting`: the choice brings the aim change's step: a running pose other than
    /// hers, or one a row counts as chosen.
    pub aim_step: bool,
    /// `ShrinkingLowersHer`: the look down the pose starts with.
    pub lower: Option<Lowers>,
    /// `AnimationEnds`: her animation's end handed the pose over.
    pub handed_over: bool,
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
            aim_step: false,
            lower: None,
            handed_over: false,
        }
    }

    /// `WhatHerMovesMet`, `AimingAndShooting`: the choice once a block or solid enemy blocked the
    /// aim change's step. A running pose chosen goes against the wall (`table::STEP_BLOCKED`),
    /// with none of its row's effects; any other (an animation's end handed one over) stands.
    #[must_use]
    pub fn blocked(self) -> Self {
        if self.pose.action == Action::Running {
            Self::instead(table::STEP_BLOCKED.pose_from(self.pose, None))
        } else {
            self
        }
    }
}

/// `AnimationEnds`: an end unless a jump gives way to the pad chart's pose this frame where that
/// pose is a take-off aimed `Forward` or a spin: no animation ends then. A plain end never gives
/// way.
fn gives_way(end: ClipEnd, pad_chart: Pose) -> bool {
    let jumps = matches!(
        (pad_chart.action, pad_chart.aim),
        (Action::TakingOff, Aim::Forward)
    ) || pad_chart.action == Action::Spinning;
    end == ClipEnd::EndUnlessJump && jumps
}

/// `PadChart`: a row that needs a just-pressed button outranks one that doesn't; then the one that
/// names more buttons.
fn rank(row: &Row) -> (bool, usize) {
    let needs = row.needs.iter();
    (
        needs.clone().any(|t| matches!(t, Test::Pressed(_))),
        needs
            .filter(|t| matches!(t, Test::Held(_) | Test::Pressed(_)))
            .count(),
    )
}

impl Moment {
    /// `PoseFollows`: the pose she takes from `pose`, hers at the frame's start, at this moment:
    /// an animation's end hands over its pose, else what her moves met chooses, else the pad
    /// chart's. What the pose then does, pose fit and its effects, is the step's. A pose no
    /// situation names has no next pose: it is reported.
    ///
    /// # Errors
    ///
    /// `UnknownPose`, where no situation names `pose`.
    pub fn next_pose(&self, pose: Pose) -> Result<Pose, UnknownPose> {
        Ok(self.choose(pose, classify(pose)?).pose)
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
    moment: &'a Moment,
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

    /// `PadChart`, `Aiming`, `WhatHerMovesMet`, `AnimationEnds`: whether one test of a row
    /// passes; forward and back are by her facing at the frame's start.
    fn passes(&self, test: &Test) -> bool {
        let (p, m) = (self.pose, self.moment);
        match *test {
            Test::Held(b) => seen(m.pad, p.facing).any(b),
            Test::Pressed(b) => seen(m.pressed, p.facing).any(b),
            Test::Not(t) => !self.passes(t),
            Test::Aimed(aims) => aims.contains(&p.aim),
            Test::Gun(gun) => p.gun == gun,
            Test::Faces(side) => Side::of(p.facing) == side,
            Test::Body(bodies) => bodies.contains(&p.body),
            Test::Action(actions) => actions.contains(&p.action),
            Test::Doing(doings) => doings.contains(&self.situation.doing),
            Test::Airborne => m.airborne,
            Test::With(items) => items.iter().any(|&i| m.items[i]),
            Test::Moving => m.speed != ZERO,
            Test::Met(met) => m.met == Some(met),
        }
    }

    /// `PadChart`: whether a row's every test passes.
    fn fits(&self, tests: &[Test]) -> bool {
        tests.iter().all(|t| self.passes(t))
    }

    /// `PadChart`, `Aiming`: a row's choice, the pose it leads to from hers and its effects;
    /// `AimingAndShooting`: a running pose other than hers, or her running pose a row counts as
    /// chosen, brings the aim change's step.
    fn choice(&self, row: &Row) -> Choice {
        let aim = row.next.aim.iter().find(|a| self.fits(a.when));
        let pose = row.next.pose_from(self.pose, aim.map(|a| a.aim));
        Choice {
            pose,
            effects: row.effects,
            aim_step: pose.action == Action::Running && (pose != self.pose || row.aim_step),
            lower: row.next.lower,
            handed_over: false,
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

    /// `PoseFollows`: the reading's choice, the first that applies. An animation ending this
    /// frame, unless its end gives way, hands over its row's pose and effects (`AnimationEnds`);
    /// else what her moves met chooses (`WhatHerMovesMet`); else the pad chart's row
    /// (`PadChart`). Whether the aim change's step comes is always the second's or third's, even
    /// where an animation's end hands over the pose: the end neither brings nor cancels it
    /// (`AimingAndShooting`).
    fn choose(&self) -> Choice {
        let pad_chart = self.choice(self.pad_chart());
        let by_moves = self
            .first(table::MOVES_MET)
            .map_or(pad_chart, |r| self.choice(r));
        let ends = self
            .moment
            .animation_ends
            .is_some_and(|end| !gives_way(end, pad_chart.pose));
        self.first(table::ANIMATION_ENDS)
            .filter(|_| ends)
            .map_or(by_moves, |r| Choice {
                aim_step: by_moves.aim_step,
                handed_over: true,
                ..self.choice(r)
            })
    }
}
