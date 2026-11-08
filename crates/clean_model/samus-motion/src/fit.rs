//! `PoseFitAfterMoves`: placing her body when the pose change grows it, by looks that test a
//! move without making it, from her box and her poses alone.

use crate::collide::{Bounds, Probe, Stop};
use crate::tick::Tick;
use Change::{Chosen, Crouched, Refused};
use Probe::{Blocks, Enemies};
use clean_types::Action::Still;
use clean_types::{Aim, Body, Facing, Fixed, Gun, Legs, Parts, Pose, Position, Radius, px};

/// `PoseFitAfterMoves`: how the pose the frame ends in came about.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// The chosen pose.
    Chosen,
    /// The change refused: she keeps the pose she had.
    Refused,
    /// Crouched `Still` taken instead.
    Crouched,
}

/// `PoseFitAfterMoves`: what pose fit decided, and where its looks and its push leave her.
pub struct Fit {
    /// How the pose came about.
    pub change: Change,
    /// The pose she takes.
    pub pose: Pose,
    /// Her position.
    pub at: Position,
    /// `SlopesCarry`: a look down stopped by a stepped block seated her.
    pub seated: bool,
}

/// `PoseFitAfterMoves`: crouched `Still`, aim `Forward`, facing as before the change.
const fn crouched(p: Pose) -> Pose {
    Pose {
        body: Body::Crouched,
        action: Still,
        aim: Aim::Forward,
        gun: Gun::Stowed,
        legs: Legs::Together,
        ..p
    }
}

/// `PoseFitAfterMoves`: the room the looks found above her and below her, where something
/// stopped them, in whole px.
#[derive(Clone, Copy, Default)]
struct Found {
    /// Above.
    above: Option<u16>,
    /// Below.
    below: Option<u16>,
}

impl Found {
    /// With something on one side alone: the push away from it, `g` less the room on its side,
    /// signed, downward positive (down from what is above her, up from what is below her).
    fn alone(self, g: u16) -> Option<Fixed> {
        match (self.above, self.below) {
            (Some(room), None) => Some(px(g.wrapping_sub(room))),
            (None, Some(room)) => Some(-px(g.wrapping_sub(room))),
            (Some(_), Some(_)) | (None, None) => None,
        }
    }

    /// The rooms found, these looks' before `earlier`'s on the same side.
    fn or(self, earlier: Self) -> Self {
        Self {
            above: self.above.or(earlier.above),
            below: self.below.or(earlier.below),
        }
    }
}

/// `PoseFitAfterMoves`: when the chosen pose differs from hers (`pose`, her box `from` where her
/// moves left her), doesn't face the screen and has a body taller by g px, she looks g px up and
/// down at her old size: solid enemies up and down, then the enemy's far-side look (at her new
/// size); blocks up and down, then the block's far-side look. A far-side look stopped refuses
/// the change. With something on one side only she is pushed away from it (a block's room
/// before an enemy's on the same side), keeping the subpixel her looks left; with something on
/// both, a ball's change is refused, and any other body takes crouched `Still` instead, rising
/// by the crouch's growth over her old body, her feet where they were.
pub fn fit(t: &mut Tick, from: Bounds, pose: Pose, chosen: Pose) -> Fit {
    let (old, new) = (from.r, t.body(chosen));
    let g = new.y.saturating_sub(old.y);
    let mut fit = Fit {
        change: Chosen,
        pose: chosen,
        at: from.at,
        seated: false,
    };
    if chosen == pose || chosen.facing == Facing::Screen || g == 0 {
        return fit;
    }
    let (down, mut found) = (px(g), Found::default());
    for (probe, far) in [(Enemies, new), (Blocks, old)] {
        let here = Found {
            above: fit.look(t, -down, old, probe),
            below: fit.look(t, down, old, probe),
        };
        if let Some(push) = here.alone(g)
            && fit.look(t, push, far, probe).is_some()
        {
            return fit.taking(Refused, pose);
        }
        found = here.or(found);
    }
    match found.alone(g) {
        Some(push) => fit.at.y += push,
        None if found.above.is_none() => {}
        None if old.y < t.rules.small_body_px => return fit.taking(Refused, pose),
        None => {
            let crouch = crouched(pose);
            fit.at.y -= px(t.body(crouch).y.saturating_sub(old.y));
            return fit.taking(Crouched, crouch);
        }
    }
    fit
}

impl Fit {
    /// `PoseFitAfterMoves`: the change refused, or crouched `Still` taken instead: this pose.
    const fn taking(self, change: Change, pose: Pose) -> Self {
        Self {
            change,
            pose,
            ..self
        }
    }

    /// `PoseFitAfterMoves`: one look by `by` (signed, downward positive) at size `r`. It doesn't
    /// move her, but what stops it writes her y subpixel as a stop would, and a stepped block
    /// below seats her (`SlopesCarry`). Returns the room it found where something stopped it:
    /// how far it got, in whole px.
    fn look(&mut self, t: &mut Tick, by: Fixed, r: Radius, probe: Probe) -> Option<u16> {
        let at = self.at;
        let seen = t.look(Bounds { at, r }, by, probe);
        self.at.y = at.y.with_sub(seen.to.y.sub());
        self.seated |= !by.negative() && seen.stop == Some(Stop::Stepped);
        seen.stop.map(|_| seen.to.y.pixel().abs_diff(at.y.pixel()))
    }
}
