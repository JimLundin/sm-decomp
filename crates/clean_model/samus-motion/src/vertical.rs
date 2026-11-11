//! Her vertical motion as a value of its own (§1 vertical speed and bounces, §1.2 falling at 0
//! and the moonfall).
//!
//! How the pull grows her speed and when a rise ends (`RisingAndFalling`, `Moonfall`), how her
//! vertical move goes and what its end meets (`FollowingTheGround`), and what a floor or a wall's
//! kick does to it (`Landing`, `WallJump`, `Jumping`). Samus holds one; her moves read it. Each
//! change names only the parts of it that it changes.

use crate::rows::Doing::{self, Hop, Spinning, StraightJump};
use crate::rows::Footing;
use crate::rows::Met::{self, Ceiling, Floor, NoFloor, Wall, WallKick};
use crate::rules::{Bounce, Bounces, Rules};
use crate::tick::Tick;
use Phase::{Falling, Ground, GroundFalling, Moonfall, Rising};
use clean_types::button::JUMP;
use clean_types::{Buttons, Fixed, Item, Parts, ZERO};

/// Her vertical motion's phase.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    /// In the air, falling, the pull growing her speed up to the fall's limit.
    Falling,
    /// `Moonfall`: falling with no falling limit and no jump, until she lands.
    Moonfall,
}

/// Her vertical motion: her vertical speed, its phase, and the ball's bounces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vertical {
    /// Her vertical speed, signed, downward positive.
    pub speed: Fixed,
    /// Whether she is on the ground, rising or falling, and how.
    pub phase: Phase,
    /// The ball's bounces since it last rested (`Landing`).
    pub bounces: Bounces,
}

/// `FollowingTheGround`, `RisingAndFalling`, `Moonfall`: how her vertical move goes this frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Course {
    /// The ground pull: on the ground, with no vertical motion she moves by (a turn in the
    /// moonfall too), she is pulled down by that frame's speed plus 1 px, 1 px seated on a slope.
    /// The floor stops her flush and never lands her.
    Pulled,
    /// The ball on the ground with a vertical motion: it moves by its vertical speed under the
    /// pull; the floor stops it flush and never lands it, and finding none it walks off a ledge.
    BallMovesByIt,
    /// In the air, or while her body changes shape with a vertical motion: she moves by her
    /// vertical speed under the pull; a floor meets her as her situation's footing says, and a
    /// block above ends the rise.
    InTheAir,
}

/// `Landing`, `WallJump`: what her moves' contact with a floor or a wall does to her motion,
/// once the change it comes with is taken (`PoseFitAfterMoves`: a change refused, or the crouch
/// taken instead, brings none of it). The pose it chooses is the chart's (`WhatHerMovesMet`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    /// §1.4, `RisingAndFalling`: whether she is heading down (`Outcome::heading_down`): falling,
    /// in the air or on the ground at 0, or keeping a downward speed on the ground. Rising, in
    /// the moonfall or with no vertical motion she isn't.
    pub(crate) fn heading_down(self) -> bool {
        match self.phase {
            Falling | GroundFalling => true,
            Ground => !self.speed.negative() && self.speed != ZERO,
            Rising | Moonfall => false,
        }
    }

    /// `RisingAndFalling`, `Jumping`, `WallJump`: she starts rising at this speed.
    fn rise(&mut self, v: Fixed) {
        self.phase = Rising;
        self.speed = -v;
    }

    /// `Jumping`: she starts rising at the jump speed, as her items and her dash (`dash`, as the
    /// frame's moves left it) make it (`Rules::rise_speed`).
    pub(crate) fn jump(&mut self, dash: Fixed, t: &Tick) {
        self.rise(t.rules.rise_speed(t.rules.jump, &t.frame.items, dash));
    }

    /// `RisingAndFalling`, `WallsAndCeilings`: she falls from rest.
    const fn fall_from_rest(&mut self) {
        self.phase = Falling;
        self.speed = ZERO;
    }

    /// `Landing`: she stands on the ground with no vertical motion.
    const fn stand(&mut self) {
        self.phase = Ground;
        self.speed = ZERO;
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
    /// (`Moonfall`). `dash`: hers after the frame's horizontal move.
    pub(crate) fn space_jump(&mut self, doing: Doing, dash: Fixed, t: &Tick) {
        let (r, f) = (t.rules, t.frame);
        if self.phase == Falling
            && doing == Spinning
            && f.pressed.any(JUMP)
            && f.items[Item::SpaceJump]
            && r.space_jump.contains(&self.speed.to_8_8().0)
        {
            self.jump(dash, t);
        }
    }

    /// `Moonfall`: a spin she enters when a turn ends starts with no rise, her vertical speed 0
    /// and the moonfall begun; with jump just pressed it is a jump instead.
    pub(crate) fn enter_spin_from_a_turn(&mut self, dash: Fixed, t: &Tick) {
        if t.frame.pressed.any(JUMP) {
            self.jump(dash, t);
        } else {
            self.phase = Moonfall;
            self.speed = ZERO;
        }
    }

    /// `AnimationEnds`: the unmorph's end hands over the crouch, a pose on the ground, in the
    /// air too: she is on the ground from then on, her vertical speed kept.
    pub(crate) const fn come_to_the_ground(&mut self) {
        if !self.on_ground() {
            self.phase = Ground;
        }
    }

    /// `FollowingTheGround`, `RisingAndFalling`, `Moonfall`: how her vertical move goes, by her
    /// situation's footing. On the ground she is pulled, unless while her body changes shape
    /// (she moves as in the air) or as the ball (on the ground) she has a vertical motion to move
    /// by; a turn in the moonfall is pulled as a turn on the ground over a drop.
    pub(crate) fn course(self, footing: Footing) -> Course {
        let ground = self.on_ground();
        let moves_by_it =
            self.has_motion() && matches!(footing, Footing::ChangingShape | Footing::Ball);
        if ground && !moves_by_it || self.phase == Moonfall && footing == Footing::Turning {
            Course::Pulled
        } else if ground && footing == Footing::Ball {
            Course::BallMovesByIt
        } else {
            Course::InTheAir
        }
    }

    /// `RisingAndFalling`: the pull grows her vertical speed, which stops growing at a pixel part
    /// of exactly 5 (a speed of 6 px or more grows on); `Moonfall`: with no limit.
    fn grow(&mut self, r: &Rules) {
        if self.phase == Moonfall || self.speed.pixel() != r.fall_settle_px {
            self.speed += r.gravity;
        }
    }

    /// `FollowingTheGround`: walking off a ledge, she falls from rest, or with an upward speed she
    /// kept on the ground rises on by it; bounces 0.
    fn walk_off(&mut self) {
        if self.speed.negative() {
            self.phase = Rising;
        } else {
            self.fall_from_rest();
        }
        self.bounces = Bounces::None;
    }

    /// What her vertical move by `by` met, stopped or not, as its course and her situation's
    /// footing say: an upward move stopped is a ceiling (`WallsAndCeilings`), with no pull that
    /// frame. Moving by her speed, the pull then grows it (`RisingAndFalling`). In the air a floor
    /// is met (`Landing`); on the ground the floor stops her flush and never lands her, and
    /// finding none she walks off a ledge (`FollowingTheGround`), unless turning, where she sinks
    /// by the pull. (Changing shape, she stays where she is instead: the move's own case,
    /// `Samus::move_vertically`, before it ends.) What it met acts afterwards (`meet_in_moves`,
    /// or at the pose change for a floor).
    pub(crate) fn end_move(
        &mut self,
        course: Course,
        by: Fixed,
        stopped: bool,
        footing: Footing,
        r: &Rules,
    ) -> Option<Met> {
        if stopped && by.negative() {
            return Some(Ceiling);
        }
        if course != Course::Pulled {
            self.grow(r);
        }
        match (course, stopped) {
            (Course::InTheAir, true) => Some(Floor),
            (Course::Pulled, false) if footing == Footing::Turning => None,
            (Course::Pulled | Course::BallMovesByIt, false) => Some(NoFloor),
            (Course::Pulled | Course::BallMovesByIt, true) | (Course::InTheAir, false) => None,
        }
    }

    /// `WallsAndCeilings`, `FollowingTheGround`: what a ceiling or no floor does to her vertical
    /// motion once her moves are made: a ceiling ends the rise, falling from rest; walking off a
    /// ledge she falls from rest, or rises on by an upward speed she kept.
    pub(crate) fn meet_in_moves(&mut self, met: Option<Met>) {
        match met {
            Some(Ceiling) => self.fall_from_rest(),
            Some(NoFloor) => self.walk_off(),
            Some(Wall | WallKick | Floor) | None => {}
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
            (Floor, Footing::OnHerFeet) => Some(Contact::Land),
            (Floor, Footing::ChangingShape) => Some(Contact::Settle),
            (Floor, Footing::Ball) if springs => Some(Contact::Hop),
            (Floor, Footing::Ball) => Some(self.bounce(r).map_or(Contact::Land, Contact::Bounce)),
            (Floor, Footing::Turning) | (Wall | NoFloor | Ceiling, _) => None,
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

    /// `Landing`, `WallJump`: what her moves' contact does to her vertical motion. The kick and
    /// the spring ball's hop rise as `Jumping` makes the wall-jump and jump speeds, with her
    /// `dash`; a bounce is a rise at the bounce speed less its own cut, with no share of the
    /// dash, and its bounces. Settling and landing stand her on the ground; a landing also
    /// rests the ball's bounces.
    pub(crate) fn meet(&mut self, contact: Contact, dash: Fixed, t: &Tick) {
        let r = t.rules;
        match contact {
            Contact::WallKick => self.rise(r.rise_speed(r.wall_jump, &t.frame.items, dash)),
            Contact::Hop => {
                self.jump(dash, t);
                self.bounces = Bounces::None;
            }
            Contact::Bounce(b) => {
                self.rise(r.bounce - b.slower);
                self.bounces = b.then;
            }
            Contact::Settle => self.stand(),
            Contact::Land => {
                self.stand();
                self.bounces = Bounces::None;
            }
        }
    }
}
