//! `PoseFitAfterMoves`: placing her body when the pose change grows it, by looks that test a
//! move without making it, from her box and her poses alone.

use crate::collide::{Bounds, Probe};
use crate::samus::Samus;
use crate::tick::Tick;
use Probe::{Blocks, Enemies};
use clean_types::Action::Still;
use clean_types::{Aim, Body, Facing, Gun, Legs, Pose, Px, Radius, px};

/// `PoseFitAfterMoves`: crouched `Still`, aim `Forward`; she takes it facing as before the
/// change.
const CROUCHED: Pose = Pose {
    body: Body::Crouched,
    action: Still,
    facing: Facing::Right,
    aim: Aim::Forward,
    gun: Gun::Stowed,
    legs: Legs::Together,
};

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

impl Samus {
    /// `PoseFitAfterMoves`: when the chosen pose differs from hers, doesn't face the screen and
    /// has a body taller by g px, she looks g px up and down from where her moves left her, at
    /// her old size: solid enemies up and down, then the enemy's far-side look (at her new size);
    /// blocks up and down, then the block's far-side look. A far-side look stopped refuses the
    /// change. With something on one side only she is pushed away from it (a block's room before
    /// an enemy's on the same side), keeping the subpixel her looks left; with something on both,
    /// a ball's change is refused, and any other body takes crouched `Still` instead, rising by
    /// the crouch's growth over her old body, her feet where they were. Returns the pose she
    /// takes instead of the chosen one, refused or crouched, which brings none of its effects.
    pub(crate) fn fit(&mut self, t: &mut Tick, chosen: Pose) -> Option<Pose> {
        let (old, new) = (self.radius, t.body(chosen));
        if chosen == self.pose || chosen.facing == Facing::Screen || new.y <= old.y {
            return None;
        }
        let g = new.y - old.y;
        let mut found: Rooms = (None, None);
        for (probe, far) in [(Enemies, new), (Blocks, old)] {
            let mut look = |by, r| self.look(t, by, r, probe);
            let here = (look(-g, old), look(g, old));
            if alone(here, g).is_some_and(|push| look(push, far).is_some()) {
                return Some(self.pose);
            }
            found = (here.0.or(found.0), here.1.or(found.1));
        }
        if let Some(push) = alone(found, g) {
            self.position.y += px(push);
        }
        let (Some(_), Some(_)) = found else {
            return None;
        };
        if old.y < t.rules.small_body_px {
            return Some(self.pose);
        }
        let crouch = Pose {
            facing: self.pose.facing,
            ..CROUCHED
        };
        self.position.y -= px(t.body(crouch).y.max(old.y) - old.y);
        Some(crouch)
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
