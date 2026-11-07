//! `PoseFitAfterMoves` and `ShrinkingLowersHer`: placing her body when the pose change grows or
//! shrinks it, by looks that test a move without making it.

use crate::collide::{Bounds, Mover, Probe};
use crate::{Change, Frame, Offset, Samus, crouched};
use clean_types::Heading::{Down, Up};
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
    /// With something on one side alone: the push away from it, `g` less the room on its side
    /// (down from what is above her, up from what is below her).
    const fn alone(self, g: u16) -> Option<Offset> {
        match self {
            Self {
                above: Some(room),
                below: None,
            } => Some(Offset {
                way: Down,
                px: g.wrapping_sub(room),
            }),
            Self {
                above: None,
                below: Some(room),
            } => Some(Offset {
                way: Up,
                px: g.wrapping_sub(room),
            }),
            Self { .. } => None,
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
            return Change::Chosen(None);
        }
        let mut found = Found::default();
        for (probe, far) in [(Probe::Enemies, new), (Probe::Blocks, old)] {
            let here = Found {
                above: self.look(m, Offset { way: Up, px: g }, old, probe),
                below: self.look(m, Offset { way: Down, px: g }, old, probe),
            };
            if let Some(push) = here.alone(g)
                && self.look(m, push, far, probe).is_some()
            {
                return Change::Refused;
            }
            found = Found {
                above: here.above.or(found.above),
                below: here.below.or(found.below),
            };
        }
        if let Some(push) = found.alone(g) {
            let (y, by) = (self.position.y, Fixed::px(push.px));
            self.position.y = if push.way == Up {
                y.minus(by)
            } else {
                y.plus(by)
            };
            return Change::Chosen(Some(push));
        }
        if found.above.is_none() {
            return Change::Chosen(None);
        }
        if old.y < m.rules.small_body_px {
            return Change::Refused;
        }
        let crouch = (frame.body)(crouched(self.pose));
        self.position.y = self
            .position
            .y
            .minus(Fixed::px(crouch.y.saturating_sub(old.y)));
        Change::Crouched
    }

    /// `PoseFitAfterMoves`: one look at size `r`. It doesn't move her, but what stops it writes
    /// her y subpixel as a stop would, and a stepped block below seats her (`SlopesCarry`).
    /// Returns the room it found where something stopped it: how far it got, in whole px.
    fn look(&mut self, m: &mut Mover, look: Offset, r: Radius, probe: Probe) -> Option<u16> {
        let at = self.position;
        let seen = m.look(Bounds { at, r }, look.way, look.px, probe);
        self.position.y = at.y.with_sub(seen.to.y.sub());
        if look.way == Down {
            self.seat_on(seen.stop);
        }
        seen.stop.map(|_| seen.to.y.pixel().abs_diff(at.y.pixel()))
    }

    /// `ShrinkingLowersHer`: she looks down `px` at her new size, a block look (solid enemies
    /// don't stop it), and she is lowered as far as it got; what stops it writes to her as a
    /// stop would.
    pub(crate) fn lower(&mut self, px: u16, m: &mut Mover) {
        let seen = m.look(self.bounds(), Down, px, Probe::Blocks);
        self.position = seen.to;
        self.seat_on(seen.stop);
    }
}
