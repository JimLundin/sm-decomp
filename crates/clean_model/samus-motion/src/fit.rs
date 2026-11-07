//! `PoseFitAfterMoves` and `ShrinkingLowersHer`: placing her body when the pose change grows or
//! shrinks it, by looks that test a move without making it.

use crate::collide::{Bounds, Mover, Probe};
use crate::{Change, Frame, Samus, crouched};
use Change::{Chosen, Crouched, Refused};
use Probe::{Blocks, Enemies};
use clean_types::{Facing, Fixed, Pose, Radius};

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
    const fn alone(self, g: u16) -> Option<Fixed> {
        match (self.above, self.below) {
            (Some(room), None) => Some(Fixed::px(g.wrapping_sub(room))),
            (None, Some(room)) => Some(Fixed::px(g.wrapping_sub(room)).negated()),
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

impl Samus {
    /// `PoseFitAfterMoves`: when the chosen pose differs from hers, doesn't face the screen and
    /// has a body taller by g px, she looks g px up and down at her old size, from where her
    /// moves left her: solid enemies up and down, then the enemy's far-side look (at her new
    /// size); blocks up and down, then the block's far-side look. A far-side look stopped
    /// refuses the change. With something on one side only she is pushed away from it (a
    /// block's room before an enemy's on the same side), keeping the subpixel her looks left;
    /// with something on both, a ball's change is refused, and any other body takes crouched
    /// `Still` instead, rising by the crouch's growth over her old body, her feet where they
    /// were.
    pub(crate) fn fit(&mut self, chosen: Pose, frame: &Frame, m: &mut Mover) -> Change {
        let (old, new) = (self.radius, (frame.body)(chosen));
        let g = new.y.saturating_sub(old.y);
        if chosen == self.pose || chosen.facing == Facing::Screen || g == 0 {
            return Chosen;
        }
        let (down, mut found) = (Fixed::px(g), Found::default());
        for (probe, far) in [(Enemies, new), (Blocks, old)] {
            let here = Found {
                above: self.look(m, down.negated(), old, probe),
                below: self.look(m, down, old, probe),
            };
            if let Some(push) = here.alone(g)
                && self.look(m, push, far, probe).is_some()
            {
                return Refused;
            }
            found = here.or(found);
        }
        let y = self.position.y;
        match found.alone(g) {
            Some(push) => self.position.y = y.plus(push),
            None if found.above.is_none() => {}
            None if old.y < m.rules.small_body_px => return Refused,
            None => {
                let crouch = (frame.body)(crouched(self.pose));
                self.position.y = y.minus(Fixed::px(crouch.y.saturating_sub(old.y)));
                return Crouched;
            }
        }
        Chosen
    }

    /// `PoseFitAfterMoves`: one look by `by` (signed, downward positive) at size `r`. It doesn't
    /// move her, but what stops it writes her y subpixel as a stop would, and a stepped block
    /// below seats her (`SlopesCarry`). Returns the room it found where something stopped it:
    /// how far it got, in whole px.
    fn look(&mut self, m: &mut Mover, by: Fixed, r: Radius, probe: Probe) -> Option<u16> {
        let at = self.position;
        let seen = m.look(Bounds { at, r }, by, probe);
        self.position.y = at.y.with_sub(seen.to.y.sub());
        if !by.negative() {
            self.seat_on(seen.stop);
        }
        seen.stop.map(|_| seen.to.y.pixel().abs_diff(at.y.pixel()))
    }

    /// `ShrinkingLowersHer`: she looks down `px` at her new size, a block look (solid enemies
    /// don't stop it), and she is lowered as far as it got; what stops it writes to her as a
    /// stop would.
    pub(crate) fn lower(&mut self, by: Fixed, m: &mut Mover) {
        let seen = m.look(self.bounds(), by, Blocks);
        self.position = seen.to;
        self.seat_on(seen.stop);
    }
}
