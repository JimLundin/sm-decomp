//! `PoseFitAfterMoves`: when the pose she takes has a taller body, she looks up and down for
//! room and is pushed clear of what is above or below her, or the change is refused or becomes
//! a crouch.

use crate::collide::{Bounds, Probe};
use crate::samus::Samus;
use crate::tick::Tick;
use core::num::Wrapping;
use types::{Action, Aim, Body, Facing, Gun, Legs, Pose, Px, Radius, px};

/// `PoseFitAfterMoves`: the crouch she takes instead, crouched `Still`, aim `Forward`, facing
/// as before the change.
pub const fn crouched(facing: Facing) -> Pose {
    Pose {
        body: Body::Crouched,
        action: Action::Still,
        facing,
        aim: Aim::Forward,
        gun: Gun::Stowed,
        legs: Legs::Together,
    }
}

/// `PoseFitAfterMoves`: what fitting the chosen pose decided.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fitted {
    /// She takes it where she is.
    Taken,
    /// She takes it, pushed away from what is on one side of her.
    Pushed,
    /// The change is refused: she keeps the pose she had.
    Refused,
    /// She takes crouched `Still` instead, raised by the difference in half-heights.
    Crouched {
        /// The whole pixels she rises, her feet staying where they were.
        rise: Px,
    },
}

impl Fitted {
    /// `PoseFitAfterMoves`: whether pose fit moved her, which places her: a push, or the crouch
    /// taken instead raising her by more than 0 px.
    #[must_use]
    pub fn moved(self) -> bool {
        match self {
            Self::Pushed => true,
            Self::Crouched { rise } => rise != Wrapping(0),
            Self::Taken | Self::Refused => false,
        }
    }

    /// `PoseFitAfterMoves`: whether she takes the chosen pose, and with it its effects and what
    /// her moves met.
    #[must_use]
    pub const fn takes_the_change(self) -> bool {
        matches!(self, Self::Taken | Self::Pushed)
    }
}

/// `PoseFitAfterMoves`: the room the looks found on each side, in whole px, where something
/// stopped them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Rooms {
    /// What stopped the look up, and how far it got.
    above: Option<Px>,
    /// What stopped the look down, and how far it got.
    below: Option<Px>,
}

impl Rooms {
    /// These rooms, and where one side found nothing, the other looks' room on that side.
    fn or(self, other: Self) -> Self {
        Self {
            above: self.above.or(other.above),
            below: self.below.or(other.below),
        }
    }

    /// `PoseFitAfterMoves`: with something on one side only, the signed distance she still
    /// needs on the far side, `growth − room`: down from what is above her, up from what is
    /// below her.
    fn one_sided(self, growth: Px) -> Option<Px> {
        match (self.above, self.below) {
            (Some(room), None) => Some(growth - room),
            (None, Some(room)) => Some(room - growth),
            (Some(_), Some(_)) | (None, None) => None,
        }
    }
}

impl Samus {
    /// `PoseFitAfterMoves`: fits the chosen pose's body, from where her moves left her, at her
    /// old size. A body no taller needs nothing. A taller one, by `growth` px, looks up then
    /// down against solid enemies, then (with an enemy on one side alone) looks on the far
    /// side at her new size; then likewise against blocks at her old size. A far-side look
    /// stopped refuses the change, and the looks after it don't run. Then, as the looks found:
    /// a push away from one side, the crouch or a refusal between two, or nothing.
    pub(crate) fn fit(&mut self, chosen: Pose, t: &mut Tick) -> Fitted {
        let (old, new) = ((t.frame.body)(self.pose), (t.frame.body)(chosen));
        if new.y <= old.y {
            return Fitted::Taken;
        }
        let growth = new.y - old.y;
        let enemies = self.rooms(growth, Probe::Enemies, t);
        if self.far_side_blocked(enemies, growth, chosen, Probe::Enemies, t) {
            return Fitted::Refused;
        }
        let blocks = self.rooms(growth, Probe::Blocks, t);
        if self.far_side_blocked(blocks, growth, self.pose, Probe::Blocks, t) {
            return Fitted::Refused;
        }
        self.settle(blocks.or(enemies), growth, old, t)
    }

    /// `PoseFitAfterMoves`: a look up and then a look down of `growth` px at her old size, each
    /// seeing what the one before it wrote.
    fn rooms(&mut self, growth: Px, probe: Probe, t: &mut Tick) -> Rooms {
        let above = self.look(-growth, self.pose, probe, t);
        let below = self.look(growth, self.pose, probe, t);
        Rooms { above, below }
    }

    /// `PoseFitAfterMoves`: with something on one side alone, whether the far side's look, in
    /// the body of `pose`, is stopped.
    fn far_side_blocked(
        &mut self,
        rooms: Rooms,
        growth: Px,
        pose: Pose,
        probe: Probe,
        t: &mut Tick,
    ) -> bool {
        rooms
            .one_sided(growth)
            .is_some_and(|far| self.look(far, pose, probe, t).is_some())
    }

    /// `PoseFitAfterMoves`: a look by the signed `by` px from where she is, in the body of
    /// `pose`. It doesn't move her, but what stops it writes to her as a stop would. Returns
    /// the room it found where something stopped it.
    fn look(&mut self, by: Px, pose: Pose, probe: Probe, t: &mut Tick) -> Option<Px> {
        let from = Bounds {
            at: self.position,
            r: (t.frame.body)(pose),
        };
        let looked = t.up_or_down(from, px(by), probe);
        self.take_result(&looked);
        looked.stop.map(|_| looked.gone)
    }

    /// `PoseFitAfterMoves`: what the looks found decides. Something on one side only pushes her
    /// away from it by `growth − room` whole px, her subpixel as the looks left it. Something
    /// on both sides refuses the change from an old body under the small body's half-height,
    /// and from any other she takes the crouch instead, rising by the difference in
    /// half-heights where her old body is shorter. Nothing in reach: she grows about her centre.
    fn settle(&mut self, found: Rooms, growth: Px, old: Radius, t: &Tick) -> Fitted {
        if let Some(push) = found.one_sided(growth) {
            self.position.y += px(push);
            return Fitted::Pushed;
        }
        if found.above.is_none() || found.below.is_none() {
            return Fitted::Taken;
        }
        if old.y < t.rules.small_body {
            return Fitted::Refused;
        }
        let crouch = (t.frame.body)(crouched(self.pose.facing));
        let rise = crouch.y.max(old.y) - old.y;
        self.position.y -= px(rise);
        Fitted::Crouched { rise }
    }
}
