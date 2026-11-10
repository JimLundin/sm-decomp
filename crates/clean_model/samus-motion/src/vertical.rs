//! Her vertical motion as a value of its own (§1 vertical speed and bounces, §1.2 falling at 0
//! and the moonfall).
//!
//! How the pull grows her speed and when a rise ends (`RisingAndFalling`, `Moonfall`), how her
//! vertical move goes and what its end meets (`FollowingTheGround`), and what a floor or a wall's
//! kick does to it (`Landing`, `WallJump`, `Jumping`). Samus holds one; her moves read it. Her
//! vertical speed, its phase and the bounces change in one place, `Vertical::set`.

use crate::rows::Doing::{self, Hop, Spinning, StraightJump};
use crate::rows::Footing;
use crate::rows::Met::{self, Ceiling, Floor, NoFloor, Wall, WallKick};
use crate::rules::{Bounce, Bounces, Rules};
use crate::tick::Tick;
use Phase::{Falling, Ground, GroundFalling, Moonfall, Rising};
use clean_types::button::JUMP;
use clean_types::{Buttons, Fixed, Item, Parts, ZERO};

/// Her vertical motion's phase.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// On the ground, keeping the vertical speed she brought with her, downward or upward
    /// (`FollowingTheGround`); an upward one is a rise she keeps there (`RisingAndFalling`).
    Ground,
    /// On the ground, falling at a vertical speed of 0 (§1.2): only a push beyond this spec
    /// hands her back so. As the ball, or while her body changes shape, it is a vertical motion
    /// she moves by; in any other pose on the ground she keeps it and moves as with none, and
    /// only leaving the ground ends it (`FollowingTheGround`).
    GroundFalling,
    /// Rising (`RisingAndFalling`); at the start of a frame after the pull took her upward
    /// speed past zero, the rise ends.
    Rising,
    /// Falling.
    Falling,
    /// `Moonfall`: falling with no falling limit and no jump, until she lands.
    Moonfall,
}

/// Her vertical motion: her vertical speed, its phase, and the ball's bounces.
#[derive(Clone, Copy)]
pub struct Vertical {
    /// Her vertical speed, signed, downward positive.
    pub speed: Fixed,
    /// Its phase.
    pub phase: Phase,
    /// The ball's bounces since it last rested (`Landing`).
    pub bounces: Bounces,
}

/// `FollowingTheGround`, `RisingAndFalling`, `Moonfall`: how her vertical move goes this frame.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Course {
    /// The ground pull: on the ground, with no vertical motion she moves by (a turn in the
    /// moonfall too), she is pulled down by that frame's speed plus 1 px, 1 px seated on a slope.
    /// The floor stops her flush and never lands her.
    Pulled,
    /// The ball on the ground with a vertical motion: it moves by its vertical speed under the
    /// pull; the floor stops it flush and never lands it, and finding none it walks off a ledge.
    OnTheGround,
    /// In the air, or while her body changes shape with a vertical motion: she moves by her
    /// vertical speed under the pull; a floor meets her as her situation's footing says, and a
    /// block above ends the rise.
    InTheAir,
}

/// What the end of her vertical move leaves her.
pub enum MoveEnd {
    /// `FollowingTheGround`: she stays where she was: the ground pull found no floor while her
    /// body changes shape.
    Stays,
    /// She ends where the move left her, having met this, if anything.
    Moved(Option<Met>),
}

/// `Landing`, `WallJump`: what her moves' contact with a floor or a wall does to her motion,
/// once the change it comes with is taken (`PoseFitAfterMoves`: a change refused, or the crouch
/// taken instead, brings none of it). The pose it chooses is the chart's (`WhatHerMovesMet`).
#[derive(Clone, Copy)]
pub enum Contact {
    /// `WallJump`'s kick off a wall: she rises at the wall-jump speed, x speed 0, following the
    /// pad; it ends a moonfall.
    WallKick,
    /// `Landing`: the spring ball landing with jump held hops again at the jump speed, bounces 0.
    Hop,
    /// `Landing`: the ball bounces.
    Bounce(Bounce),
    /// `Landing` while her body changes shape: only her vertical motion stops; both speeds, and
    /// whether she carries on, are kept.
    Settle,
    /// `Landing`: vertical speed 0, bounces 0, horizontal speed 0, following the pad; the dash
    /// keeps its value, to lose (§1.2).
    Land,
}

impl Contact {
    /// `RisingAndFalling`: whether it starts her rising, so that she is off the ground once her
    /// moves are made.
    #[must_use]
    pub const fn rises(self) -> bool {
        matches!(self, Self::WallKick | Self::Hop | Self::Bounce(_))
    }
}

impl Vertical {
    /// The one place her vertical motion changes: its phase, its speed and the bounces.
    const fn set(&mut self, phase: Phase, speed: Fixed, bounces: Bounces) {
        *self = Self {
            speed,
            phase,
            bounces,
        };
    }

    /// `FollowingTheGround`: whether she is on the ground, falling at 0 there included.
    #[must_use]
    pub const fn on_ground(self) -> bool {
        matches!(self.phase, Ground | GroundFalling)
    }

    /// `FollowingTheGround` (§1.2): a vertical motion, on the ground: a vertical speed other
    /// than 0, or falling at 0.
    pub(crate) fn has_motion(self) -> bool {
        self.speed != ZERO || self.phase == GroundFalling
    }

    /// §1, `RisingAndFalling`: whether she is rising (`Outcome::rising`): a rise in the air, or
    /// an upward speed she keeps on the ground.
    pub(crate) fn rising(self) -> bool {
        match self.phase {
            Rising => true,
            Ground => self.speed.negative(),
            GroundFalling | Falling | Moonfall => false,
        }
    }

    /// `RisingAndFalling`, `Jumping`, `WallJump`: she starts rising at this speed.
    pub(crate) fn rise(&mut self, v: Fixed) {
        self.set(Rising, -v, self.bounces);
    }

    /// `Jumping`: she starts rising at the jump speed her items give.
    pub(crate) fn jump(&mut self, t: &Tick) {
        self.rise(t.rules.jump.of(&t.frame.items));
    }

    /// `RisingAndFalling`, `WallsAndCeilings`: she falls from rest.
    const fn fall_from_rest(&mut self) {
        self.set(Falling, ZERO, self.bounces);
    }

    /// `RisingAndFalling`: at a frame's start the rise ends, falling from rest, once the pull
    /// spent it (her speed points down by even a subpixel), or with jump not held in a straight
    /// jump, a spin or the spring ball's hop (`doing`).
    pub(crate) fn end_rise(&mut self, doing: Doing, pad: Buttons) {
        let lets_go = !pad.any(JUMP) && matches!(doing, StraightJump | Spinning | Hop);
        let spent = !self.speed.negative() && self.speed != ZERO;
        if self.phase == Rising && (lets_go || spent) {
            self.fall_from_rest();
        }
    }

    /// `Jumping`: with space jump, pressing jump while falling in a spin with her fall speed in
    /// the window (read as 8.8) jumps again, rising that same frame; not in a moonfall
    /// (`Moonfall`).
    pub(crate) fn space_jump(&mut self, doing: Doing, t: &Tick) {
        let (r, f) = (t.rules, t.frame);
        if self.phase == Falling
            && doing == Spinning
            && f.pressed.any(JUMP)
            && f.items[Item::SpaceJump]
            && r.space_jump.contains(&self.speed.to_8_8().0)
        {
            self.jump(t);
        }
    }

    /// `Moonfall`: a spin she enters when a turn ends starts with no rise, her vertical speed 0
    /// and the moonfall begun; with jump just pressed it is a jump instead.
    pub(crate) fn enter_spin_from_a_turn(&mut self, t: &Tick) {
        if t.frame.pressed.any(JUMP) {
            self.jump(t);
        } else {
            self.set(Moonfall, ZERO, self.bounces);
        }
    }

    /// `AnimationEnds`: the unmorph's end hands over the crouch, a pose on the ground, in the
    /// air too: she is on the ground from then on, her vertical speed kept.
    pub(crate) const fn come_to_the_ground(&mut self) {
        if !self.on_ground() {
            self.set(Ground, self.speed, self.bounces);
        }
    }

    /// `FollowingTheGround`, `RisingAndFalling`, `Moonfall`: how her vertical move goes, by her
    /// situation's footing. On the ground she is pulled, unless while her body changes shape
    /// (she moves as in the air) or as the ball (on the ground) she has a vertical motion to move
    /// by; a turn in the moonfall is pulled as a turn on the ground over a drop.
    pub(crate) fn course(self, footing: Footing) -> Course {
        let ground = self.on_ground();
        let moves_by_it =
            self.has_motion() && matches!(footing, Footing::Settles | Footing::Bounces);
        if ground && !moves_by_it || self.phase == Moonfall && footing == Footing::Holds {
            Course::Pulled
        } else if ground && footing == Footing::Bounces {
            Course::OnTheGround
        } else {
            Course::InTheAir
        }
    }

    /// `RisingAndFalling`: the pull grows her vertical speed, which stops growing at pixel part
    /// 5; `Moonfall`: with no limit.
    fn pull(&mut self, r: &Rules) {
        let capped = self.phase != Moonfall && self.speed.pixel() == r.fall_settle_px;
        let speed = if capped {
            self.speed
        } else {
            self.speed + r.gravity
        };
        self.set(self.phase, speed, self.bounces);
    }

    /// `FollowingTheGround`: walking off a ledge, she falls from rest, or with an upward speed she
    /// kept on the ground rises on by it; bounces 0.
    fn walk_off(&mut self) {
        if self.speed.negative() {
            self.set(Rising, self.speed, Bounces::None);
        } else {
            self.set(Falling, ZERO, Bounces::None);
        }
    }

    /// The one decision of what her vertical move by `by` met, stopped or not, as its course
    /// and her situation's footing say. An upward move stopped is a ceiling (`WallsAndCeilings`):
    /// the rise ends with no pull that frame. Moving by her speed, the pull then grows it
    /// (`RisingAndFalling`). In the air a floor is met (`Landing`); on the ground the floor stops
    /// her flush and never lands her, and finding none she walks off a ledge
    /// (`FollowingTheGround`), unless turning (she sinks) or changing shape (she stays).
    pub(crate) fn end_move(
        &mut self,
        course: Course,
        by: Fixed,
        stopped: bool,
        footing: Footing,
        r: &Rules,
    ) -> MoveEnd {
        if stopped && by.negative() {
            self.fall_from_rest();
            return MoveEnd::Moved(Some(Ceiling));
        }
        if course != Course::Pulled {
            self.pull(r);
        }
        match (course, stopped) {
            (Course::InTheAir, true) => MoveEnd::Moved(Some(Floor)),
            (Course::Pulled, false) if footing == Footing::Settles => MoveEnd::Stays,
            (Course::Pulled, false) if footing == Footing::Holds => MoveEnd::Moved(None),
            (Course::Pulled | Course::OnTheGround, false) => {
                self.walk_off();
                MoveEnd::Moved(Some(NoFloor))
            }
            (Course::Pulled | Course::OnTheGround, true) | (Course::InTheAir, false) => {
                MoveEnd::Moved(None)
            }
        }
    }

    /// `Landing`, `WallJump`: what what her moves met will do to her motion, once she takes the
    /// change: `WallJump`'s kick, or a floor, as her situation's footing says. `springs`: the
    /// spring ball with jump held, which hops again; else the ball bounces, or comes to rest.
    pub(crate) fn contact(
        self,
        met: Option<Met>,
        footing: Footing,
        springs: bool,
        r: &Rules,
    ) -> Option<Contact> {
        match (met?, footing) {
            (WallKick, _) => Some(Contact::WallKick),
            (Floor, Footing::Lands) => Some(Contact::Land),
            (Floor, Footing::Settles) => Some(Contact::Settle),
            (Floor, Footing::Bounces) if springs => Some(Contact::Hop),
            (Floor, Footing::Bounces) => {
                Some(self.bounce(r).map_or(Contact::Land, Contact::Bounce))
            }
            (Floor, Footing::Holds) | (Wall | NoFloor | Ceiling, _) => None,
        }
    }

    /// `Landing`: the ball's bounce a landing makes, if any: the one at the bounces she has had,
    /// where her vertical speed after the frame's pull reaches it (its pixel part alone counts).
    fn bounce(self, r: &Rules) -> Option<Bounce> {
        r.bounces
            .iter()
            .find(|b| b.at == self.bounces && self.speed.pixel() >= b.from_px)
            .copied()
    }

    /// `Landing`, `WallJump`: what her moves' contact does to her vertical motion; a bounce rises
    /// at the bounce speed less its own cut.
    pub(crate) fn meet(&mut self, contact: Contact, t: &Tick) {
        let (r, items) = (t.rules, &t.frame.items);
        match contact {
            Contact::WallKick => self.rise(r.wall_jump.of(items)),
            Contact::Hop => self.set(Rising, -r.jump.of(items), Bounces::None),
            Contact::Bounce(b) => self.set(Rising, -(r.bounce - b.slower), b.then),
            Contact::Settle => self.set(Ground, ZERO, self.bounces),
            Contact::Land => self.set(Ground, ZERO, Bounces::None),
        }
    }
}
