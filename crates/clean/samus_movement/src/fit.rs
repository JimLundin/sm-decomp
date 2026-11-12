//! `PoseFitAfterMoves`: placing her body when the pose change grows it, by looks that test a
//! move without making it, from her box and her poses alone.

use crate::collide::{Bounds, Probe};
use crate::samus::Samus;
use crate::tick::Tick;
use Probe::{Blocks, Enemies};
use core::num::Wrapping;
use types::Action::{Morphing, Still};
use types::{Aim, Body, Facing, Gun, Item, Legs, Pose, Px, Radius, px};

/// `PoseFitAfterMoves`: crouched `Still`, aim `Forward`, facing `facing` (she takes it facing
/// as before the change).
pub const fn crouched(facing: Facing) -> Pose {
    Pose {
        body: Body::Crouched,
        action: Still,
        facing,
        aim: Aim::Forward,
        gun: Gun::Stowed,
        legs: Legs::Together,
    }
}

/// `PoseFitAfterMoves`: the room the looks found above her and below her, where something
/// stopped them, in whole px.
type Rooms = (Option<Px>, Option<Px>);

/// `PoseFitAfterMoves`: with something on one side alone, the push away from it, `g` less the
/// room on its side, in whole px read signed, downward positive (down from what is above her,
/// up from what is below her).
fn alone(rooms: Rooms, g: Px) -> Option<Px> {
    match rooms {
        (Some(room), None) => Some(g - room),
        (None, Some(room)) => Some(room - g),
        (Some(_), Some(_)) | (None, None) => None,
    }
}

/// `PoseFitAfterMoves`: what pose fit made of the change, each case with what it did to her.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fitted {
    /// She takes the chosen pose where her moves left her: her body doesn't grow, or the looks
    /// found nothing on either side and she grows about her centre.
    Taken,
    /// She takes the chosen pose, pushed away from what is on one side of her: up from what is
    /// below her, down from what is above her.
    Pushed,
    /// The change is refused: she keeps the pose she had, aim included.
    Refused,
    /// Something above her and something below her: she takes crouched `Still` instead, rising
    /// by these whole px, her feet where they were (0 where her old body is already the crouch's
    /// height).
    Crouched {
        /// How far she rose.
        rise: Px,
    },
}

impl Fitted {
    /// §1, `PoseFitAfterMoves`: whether pose fit moved her, which places her: a push, or the
    /// crouch taken instead raising her.
    #[must_use]
    pub fn moved(self) -> bool {
        match self {
            Self::Pushed => true,
            Self::Crouched { rise } => rise != Wrapping(0),
            Self::Taken | Self::Refused => false,
        }
    }
}

impl Samus {
    /// `PoseFitAfterMoves`: when the chosen pose differs from hers, doesn't face the screen and
    /// has a body taller by g px, she looks g px up and down from where her moves left her, at
    /// her old size: solid enemies up and down, then the enemy's far-side look (at her new size);
    /// blocks up and down, then the block's far-side look. A far-side look stopped refuses the
    /// change. With something on one side only she is pushed away from it (a block's room before
    /// an enemy's on the same side), keeping the subpixel her looks left. With something on both,
    /// the change from an old body under 8 px half-height (the ball, `Rules::small_body_px`) is
    /// refused, and from one of 8 px or more she takes crouched `Still` instead, rising by the
    /// crouch's growth over her old body, her feet where they were.
    ///
    /// `PoseFollows`: a change into `Morphing` without the morph ball is refused as pose fit
    /// refuses one, before any look.
    pub(crate) fn fit(&mut self, t: &mut Tick, chosen: Pose) -> Fitted {
        let into_morph = chosen.action == Morphing && self.pose.action != Morphing;
        if into_morph && !t.frame.items[Item::MorphBall] {
            return Fitted::Refused;
        }
        if chosen == self.pose || chosen.facing == Facing::Screen {
            return Fitted::Taken;
        }
        let (old, new) = (t.body(self.pose), t.body(chosen));
        if new.y <= old.y {
            return Fitted::Taken;
        }
        let g = new.y - old.y;
        let mut found: Rooms = (None, None);
        for (probe, far) in [(Enemies, new), (Blocks, old)] {
            let mut look = |by, r| self.look(t, by, r, probe);
            let here = (look(-g, old), look(g, old));
            if alone(here, g).is_some_and(|push| look(push, far).is_some()) {
                return Fitted::Refused;
            }
            found = (here.0.or(found.0), here.1.or(found.1));
        }
        if let Some(push) = alone(found, g) {
            self.position.y += px(push);
            return Fitted::Pushed;
        }
        let (Some(_), Some(_)) = found else {
            return Fitted::Taken;
        };
        if old.y < t.rules.small_body_px {
            return Fitted::Refused;
        }
        let rise = t.body(crouched(self.pose.facing)).y.max(old.y) - old.y;
        self.position.y -= px(rise);
        Fitted::Crouched { rise }
    }

    /// `PoseFitAfterMoves`: one look by `by` whole px (read signed, downward positive) at size
    /// `r`. It doesn't move her, but what stops it writes her y subpixel as a stop would, and a
    /// stepped block below seats her (`SlopesCarry`). Returns the room it found where something
    /// stopped it: how far it got, in whole px.
    fn look(&mut self, t: &mut Tick, by: Px, r: Radius, probe: Probe) -> Option<Px> {
        let look = t.vmove(
            Bounds {
                at: self.position,
                r,
            },
            px(by),
            probe,
        );
        // What stopped the look writes her y subpixel (FFFF down, 0 up or against an enemy).
        self.take_result(&look);
        look.stop.map(|_| look.gone)
    }
}
