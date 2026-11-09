//! `PoseFitAfterMoves`: placing her body when the pose change grows it, by looks that test a
//! move without making it, from her box and her poses alone.

use crate::collide::{Bounds, Probe};
use crate::samus::Samus;
use crate::tick::Tick;
use Probe::{Blocks, Enemies};
use clean_types::Action::Still;
use clean_types::{Aim, Body, Facing, Gun, Legs, Pose, Px, Radius, px};
use core::num::Wrapping;

/// `PoseFitAfterMoves`: crouched `Still`, aim `Forward`, facing `facing` (she takes it facing
/// as before the change).
const fn crouched(facing: Facing) -> Pose {
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

/// `PoseFitAfterMoves`: what pose fit made of the change.
pub struct Fitted {
    /// The pose she takes instead of the chosen one, which brings none of the chosen pose's
    /// effects: the one she had where the change is refused, or crouched `Still`. `None` where
    /// she takes the chosen pose.
    pub instead: Option<Pose>,
    /// Pose fit moved her: a push, or the crouch taken instead raising her (§1: it places her).
    pub moved: bool,
}

/// `PoseFitAfterMoves`: she takes the chosen pose where her moves left her.
const TAKEN: Fitted = Fitted {
    instead: None,
    moved: false,
};

impl Samus {
    /// `PoseFitAfterMoves`: when the chosen pose differs from hers, doesn't face the screen and
    /// has a body taller by g px, she looks g px up and down from where her moves left her, at
    /// her old size: solid enemies up and down, then the enemy's far-side look (at her new size);
    /// blocks up and down, then the block's far-side look. A far-side look stopped refuses the
    /// change. With something on one side only she is pushed away from it (a block's room before
    /// an enemy's on the same side), keeping the subpixel her looks left; with something on both,
    /// a ball's change is refused, and any other body takes crouched `Still` instead, rising by
    /// the crouch's growth over her old body, her feet where they were.
    pub(crate) fn fit(&mut self, t: &mut Tick, chosen: Pose) -> Fitted {
        let (old, new) = (self.radius, t.body(chosen));
        if chosen == self.pose || chosen.facing == Facing::Screen || new.y <= old.y {
            return TAKEN;
        }
        let g = new.y - old.y;
        let refused = Fitted {
            instead: Some(self.pose),
            moved: false,
        };
        let mut found: Rooms = (None, None);
        for (probe, far) in [(Enemies, new), (Blocks, old)] {
            let mut look = |by, r| self.look(t, by, r, probe);
            let here = (look(-g, old), look(g, old));
            if alone(here, g).is_some_and(|push| look(push, far).is_some()) {
                return refused;
            }
            found = (here.0.or(found.0), here.1.or(found.1));
        }
        if let Some(push) = alone(found, g) {
            self.position.y += px(push);
            return Fitted {
                instead: None,
                moved: true,
            };
        }
        let (Some(_), Some(_)) = found else {
            return TAKEN;
        };
        if old.y < t.rules.small_body_px {
            return refused;
        }
        let crouch = crouched(self.pose.facing);
        let rise = t.body(crouch).y.max(old.y) - old.y;
        self.position.y -= px(rise);
        Fitted {
            instead: Some(crouch),
            moved: rise != Wrapping(0),
        }
    }

    /// `PoseFitAfterMoves`: one look by `by` whole px (read signed, downward positive) at size
    /// `r`. It doesn't move her, but what stops it writes her subpixel as a stop would, and a
    /// stepped block below seats her (`SlopesCarry`). Returns the room it found where something
    /// stopped it: how far it got, in whole px.
    fn look(&mut self, t: &mut Tick, by: Px, r: Radius, probe: Probe) -> Option<Px> {
        let seen = t.vmove(
            Bounds {
                at: self.position,
                r,
            },
            px(by),
            probe,
        );
        self.reach(&seen);
        seen.stop.map(|_| seen.gone)
    }
}
