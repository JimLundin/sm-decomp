//! §7: the pose chart, a state machine.
//!
//! Her situation (`situation`) is its state; its moves are rows (`table`): what a row needs held,
//! pressed or of her situation, the pose it leads to and its effects. `PadChart`'s rule picks the
//! one winning row of her situation; the rows of `WhatHerMovesMet` and `AnimationEnds` are read
//! in their tables' order, the first that fits.

pub mod table;
#[cfg(test)]
mod tests;

use crate::situation::{Doing, Situation};
use clean_types::{Action, Aim, Body, Equipment, Facing, Gun, Heading, Legs, Pad, Pose};

/// A button by what it does; forward and back are by her facing at the frame's start.
#[derive(Clone, Copy)]
pub enum Key {
    /// Towards her facing.
    Forward,
    /// Away from her facing.
    Back,
    /// Jump.
    Jump,
    /// Shoot.
    Shoot,
    /// Run.
    Run,
    /// Up.
    Up,
    /// Down.
    Down,
    /// Aim up.
    AimUp,
    /// Aim down.
    AimDown,
}

/// One thing a row needs. `Held` and `Pressed` are the buttons a row names (`PadChart`'s rank);
/// the rest are conditions on her situation.
#[derive(Clone, Copy)]
pub enum Test {
    /// One of these held.
    Held(&'static [Key]),
    /// This just pressed.
    Pressed(Key),
    /// The test failing.
    Not(&'static Self),
    /// Any of these passing.
    Any(&'static [Self]),
    /// Her aim one of these.
    Aimed(&'static [Aim]),
    /// Her gun so.
    Gun(Gun),
    /// Facing this side (the screen counts as right).
    Faces(Heading),
    /// Her body one of these.
    Body(&'static [Body]),
    /// Her action one of these.
    Action(&'static [Action]),
    /// Doing one of these.
    Doing(&'static [Doing]),
    /// Off the ground once her moves are made.
    Airborne,
    /// With the spring ball.
    SpringBall,
    /// Her moves met this (`WhatHerMovesMet`).
    Met(Met),
}

/// `Aiming`: the aim a next pose takes when the tests pass.
#[derive(Clone, Copy)]
pub struct Aiming {
    /// What it needs.
    pub when: &'static [Test],
    /// The aim.
    pub aim: Aim,
}

/// The way a next pose faces.
#[derive(Clone, Copy)]
pub enum Side {
    /// As she faced.
    Kept,
    /// Back.
    Back,
}

/// The pose a row leads to; `None` keeps that part of her pose as it was.
#[derive(Clone, Copy)]
pub struct Next {
    /// The body.
    pub body: Option<Body>,
    /// The action.
    pub action: Option<Action>,
    /// The facing.
    pub facing: Side,
    /// The first aim whose tests pass; where none does, the aim is kept.
    pub aim: &'static [Aiming],
    /// The gun.
    pub gun: Option<Gun>,
    /// The legs.
    pub legs: Option<Legs>,
}

/// What a change does to her speeds, dash and position, in order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effect {
    /// `Jumping`'s take-off: she rises, follows the pad, and from a crouch unaimed is lifted.
    TakeOff,
    /// `Jumping`'s spin start or the spring ball's hop: she rises.
    Rise,
    /// `TurningRound`.
    TurnRound,
    /// She follows the pad again: a skid ends, along her new facing.
    EndSkid,
    /// As running: the dash joins her speed and she carries on, braking.
    AsRunning,
    /// Dash 0.
    LoseDash,
    /// Horizontal speed 0, dash 0, following the pad.
    Halt,
    /// `Moonfall`.
    Moonfall,
    /// `ShrinkingLowersHer`: she is lowered this many px.
    Lower(u16),
    /// `Jumping`: starting a straight jump or a fall, she carries on along her facing if she
    /// has a dash, else follows the pad.
    StartDrift,
    /// §1.2: a spin that followed a wall jump is carried on.
    CarrySpin,
}

/// A move of the state machine: a row of one of §7's tables.
#[derive(Clone, Copy)]
pub struct Row {
    /// Its Pad cell, which a beats clause names.
    pub name: &'static str,
    /// What it needs, all of it.
    pub needs: &'static [Test],
    /// The pose it leads to.
    pub next: Next,
    /// What it does.
    pub effects: &'static [Effect],
    /// `PadChart`: the rows of its situation it beats, by name.
    pub beats: &'static [&'static str],
}

/// `WhatHerMovesMet`: what this frame's moves met.
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

/// What the chart reads: her situation and pose at the frame's start, whether she is off the
/// ground once her moves are made, the pad held and just pressed, her equipment and what her
/// moves met.
#[derive(Clone, Copy)]
pub struct Moment {
    /// Her situation.
    pub situation: &'static Situation,
    /// Her pose.
    pub pose: Pose,
    /// Off the ground once her moves are made.
    pub airborne: bool,
    /// The buttons held.
    pub pad: Pad,
    /// The buttons just pressed.
    pub pressed: Pad,
    /// Her equipment.
    pub equipment: Equipment,
    /// What her moves met.
    pub met: Option<Met>,
}

/// A chosen pose and the change's effects.
#[derive(Clone, Copy)]
pub struct Choice {
    /// The pose.
    pub pose: Pose,
    /// Its effects.
    pub effects: &'static [Effect],
}

/// `PadChart`: a row that needs a just-pressed button outranks one that doesn't; then the one that
/// names more buttons.
fn rank(row: &Row) -> (bool, usize) {
    let pressed = |t: &&Test| matches!(t, Test::Pressed(_));
    let named = |t: &&Test| matches!(t, Test::Held(_) | Test::Pressed(_));
    (
        row.needs.iter().any(|t| pressed(&t)),
        row.needs.iter().filter(named).count(),
    )
}

/// `PadChart`: whether row `a` beats row `b`: as `a`'s beats clause says, else by rank unless
/// `b`'s says it beats `a`.
fn beats(a: &Row, b: &Row) -> bool {
    a.beats.contains(&b.name) || !b.beats.contains(&a.name) && rank(a) > rank(b)
}

impl Moment {
    /// Whether `pad` holds `key`.
    const fn holds(&self, pad: Pad, key: Key) -> bool {
        let left = matches!(self.pose.facing, Facing::Left);
        match key {
            Key::Forward => (left && pad.left) || (!left && pad.right),
            Key::Back => (left && pad.right) || (!left && pad.left),
            Key::Jump => pad.jump,
            Key::Shoot => pad.shoot,
            Key::Run => pad.run,
            Key::Up => pad.up,
            Key::Down => pad.down,
            Key::AimUp => pad.aim_up,
            Key::AimDown => pad.aim_down,
        }
    }

    /// Whether one test passes.
    fn passes(&self, test: &Test) -> bool {
        let p = self.pose;
        match *test {
            Test::Held(keys) => keys.iter().any(|k| self.holds(self.pad, *k)),
            Test::Pressed(key) => self.holds(self.pressed, key),
            Test::Not(t) => !self.passes(t),
            Test::Any(tests) => tests.iter().any(|t| self.passes(t)),
            Test::Aimed(aims) => aims.contains(&p.aim),
            Test::Gun(gun) => p.gun == gun,
            Test::Faces(side) => p.facing.side() == side,
            Test::Body(bodies) => bodies.contains(&p.body),
            Test::Action(actions) => actions.contains(&p.action),
            Test::Doing(doings) => doings.contains(&self.situation.doing),
            Test::Airborne => self.airborne,
            Test::SpringBall => self.equipment.spring_ball,
            Test::Met(met) => self.met == Some(met),
        }
    }

    /// Whether every test passes.
    fn fits(&self, tests: &[Test]) -> bool {
        tests.iter().all(|t| self.passes(t))
    }

    /// A row's choice: the pose it leads to from hers, and its effects.
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
        }
    }

    /// The first row of a table that fits.
    fn first(&self, rows: &'static [Row]) -> Option<&'static Row> {
        rows.iter().find(|r| self.fits(r.needs))
    }

    /// `PadChart`: the one row of her situation that fits and no other fitting row beats (the
    /// chart's test proves there is exactly one).
    #[must_use]
    pub fn pad_chart(&self) -> Option<&'static Row> {
        let fit: Vec<&Row> = self
            .situation
            .rows
            .iter()
            .filter(|r| self.fits(r.needs))
            .collect();
        fit.iter()
            .find(|a| !fit.iter().any(|b| beats(b, a)))
            .copied()
    }

    /// `PoseFollows` items 2 and 3: what her moves met chooses (`WhatHerMovesMet`), else the pad
    /// chart. Where no row fits she keeps her pose.
    #[must_use]
    pub fn by_moves(&self) -> Choice {
        self.first(table::MOVES_MET)
            .or_else(|| self.pad_chart())
            .map_or(
                Choice {
                    pose: self.pose,
                    effects: &[],
                },
                |row| self.choice(row),
            )
    }

    /// `AnimationEnds`: the pose an ending animation hands over, whatever the pad.
    #[must_use]
    pub fn handed_over(&self) -> Option<Choice> {
        self.first(table::ANIMATION_ENDS)
            .map(|row| self.choice(row))
    }

    /// `PoseFollows`: an animation ending hands over its pose, else what her moves met chooses,
    /// else the pad chart.
    #[must_use]
    pub fn choose(&self, animation_ends: bool) -> Choice {
        animation_ends
            .then(|| self.handed_over())
            .flatten()
            .unwrap_or_else(|| self.by_moves())
    }
}
