//! The escape's steam: vents in the station's floors, walls and ceilings that, while in view,
//! wait a moment each, then blow a puff over 21 runs, rest unseen, and puff again. A puff that
//! touches her once her protection is spent throws her back as a hit does, taking no energy.
//!
//! A caller holds one [`Vent`] for each vent the room holds, places each as the room starts
//! ([`Vent::place`]), and runs them all together, in the room's order of its enemies, on each
//! frame the enemies run ([`run_vents`]); the run answers with the frame's one hit.

use core::num;
use core::num::Wrapping;

use types::ceres_shaft::{ShaftView, VentDrawOffset};
use types::{Hit, Pair, PushFrames, Px, Side, part_below};

use crate::her::Her;
use crate::puff::{Puff, PuffFigure, VentFacing};
use crate::reach::Reach;
use crate::rules::Rules;

/// `VentPlaced`, `VentWaits`: the longest wait before a vent's first puff, 32 runs; a vent's
/// placing draws its first puff's wait from 1 to it.
const LONGEST_FIRST_PUFF_WAIT: u8 = 32;
/// `VentInView`: the window reaches 8 px left of the view's left edge.
const WINDOW_LEFT: Px = Wrapping(8);
/// `VentInView`: and 264 px right of it.
const WINDOW_RIGHT: Px = Wrapping(264);
/// `VentInView`: the window reaches 8 px above the view's top.
const WINDOW_TOP: Px = Wrapping(8);
/// `VentInView`: and 248 px below it.
const WINDOW_BOTTOM: Px = Wrapping(248);
/// `PuffTouches`: the push time of a puff's hit, in frames.
const PUSH_FRAMES: u8 = 5;
/// `PuffTouches`: the push time of a puff's hit, 5 frames, within the longest push the shared
/// type allows (checked as the crate compiles).
const PUSH: PushFrames = match PushFrames::new(PUSH_FRAMES) {
    Some(push) => push,
    None => panic!("the puff's push is within the longest push"),
};

/// A vent's wait for its first puff: the runs before it blows it, 1 to 32.
///
/// It is the `w` of `Placed(w)` and of the spec's `Waiting(w)`, [`VentPhase::AwaitingFirstPuff`].
/// It belongs to a steam vent alone, and is not
/// Ridley's wait in the dark ([`crate::RidleyWait`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirstPuffWait(u8);

impl FirstPuffWait {
    /// The wait of this many runs, or none outside 1 to 32.
    #[must_use]
    pub const fn new(runs: u8) -> Option<Self> {
        if runs >= 1 && runs <= LONGEST_FIRST_PUFF_WAIT {
            Some(Self(runs))
        } else {
            None
        }
    }

    /// Its runs, 1 to 32.
    #[must_use]
    pub const fn runs(self) -> u8 {
        self.0
    }

    /// `VentPlaced`: the wait the placing's random number `r` draws, `1 + (r mod 32)`: only its
    /// low five bits count, so its low byte mod 32 is `r mod 32`.
    const fn drawn(random: u16) -> Self {
        let [low, _] = random.to_le_bytes();
        Self(low.wrapping_rem(LONGEST_FIRST_PUFF_WAIT).saturating_add(1))
    }

    /// `VentWaits`: the wait one run less; none once its last run is over.
    const fn less_one(self) -> Option<Self> {
        Self::new(self.0.saturating_sub(1))
    }
}

/// Where a vent is in its cycle, with its count. Listed alphabetically; the order means
/// nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VentPhase {
    /// No puff yet, so many runs before its first; the spec's `Waiting`.
    AwaitingFirstPuff(FirstPuffWait),
    /// As the room placed it, never run, with its first puff's wait: nothing is drawn, but it
    /// reaches the one pixel at its position.
    Placed(FirstPuffWait),
    /// Its puff out, showing a figure for so many more runs.
    Puffing(Puff),
    /// No puff, resting for so many more runs, 1 to 255.
    Resting(num::NonZeroU8),
}

impl VentPhase {
    /// `VentWaits`, `VentPuffs`, `VentRests`: the phase after one run. Its first run starts its
    /// first puff's wait; that wait or a rest counts down a run at a time, and as it runs out a puff comes; a
    /// puff shows its figures in turn, and once the last has shown the vent rests for `rest`.
    const fn stepped(self, rest: num::NonZeroU8) -> Self {
        match self {
            Self::Placed(wait) => Self::AwaitingFirstPuff(wait),
            Self::AwaitingFirstPuff(wait) => match wait.less_one() {
                Some(wait) => Self::AwaitingFirstPuff(wait),
                None => Self::Puffing(Puff::COMES),
            },
            Self::Puffing(puff) => match puff.ran() {
                Some(puff) => Self::Puffing(puff),
                None => Self::Resting(rest),
            },
            Self::Resting(runs) => match num::NonZeroU8::new(runs.get().saturating_sub(1)) {
                Some(runs) => Self::Resting(runs),
                None => Self::Puffing(Puff::COMES),
            },
        }
    }
}

/// A steam vent: the state a caller holds from one frame to the next. Its facing and position
/// never change; its phase is where it is in its cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vent {
    /// The way its puff blows.
    pub facing: VentFacing,
    /// Where it is, in whole pixels: where its puff is drawn and reaches from.
    pub position: Pair<Px>,
    /// Where it is in its cycle.
    pub phase: VentPhase,
}

impl Vent {
    /// `VentPlaced`: the room places a vent, facing so at its position, with the random number
    /// its placing draws `r`: `Placed`, its first puff's wait `1 + (r mod 32)`.
    #[must_use]
    pub const fn place(facing: VentFacing, position: Pair<Px>, random: u16) -> Self {
        Self {
            facing,
            position,
            phase: VentPhase::Placed(FirstPuffWait::drawn(random)),
        }
    }

    /// `VentPuffs`: this vent's displayed figure, an output of the vent for drawing.
    ///
    /// It is the puff figure, 1 to 7, its `Puffing(k, n)` phase shows (`k`), drawn for the
    /// vent's facing; none in any other phase. It is the vent's puff alone, and says nothing of
    /// Ridley or his wait.
    #[must_use]
    pub const fn shown_puff_figure(&self) -> Option<PuffFigure> {
        match self.phase {
            VentPhase::Puffing(puff) => Some(puff.figure()),
            VentPhase::AwaitingFirstPuff(_) | VentPhase::Placed(_) | VentPhase::Resting(_) => None,
        }
    }

    /// `VentPuffs`: whether this vent's puff is shown, an output of the vent for drawing: while
    /// its puff is out, so while it has a displayed figure.
    #[must_use]
    pub const fn shown(&self) -> bool {
        self.shown_puff_figure().is_some()
    }

    /// `VentInView`: whether it is inside the window around the view whose top left is
    /// `camera`: from 8 px left of the view to 264 px right of its left edge, and from 8 px above
    /// it to 248 px below its top, each end included, each difference read signed.
    fn in_view(self, camera: Pair<Px>) -> bool {
        let Pair { x, y } = self.position;
        !part_below((x + WINDOW_LEFT).0, camera.x.0)
            && !part_below((camera.x + WINDOW_RIGHT).0, x.0)
            && !part_below((y + WINDOW_TOP).0, camera.y.0)
            && !part_below((camera.y + WINDOW_BOTTOM).0, y.0)
    }

    /// `PuffReach`: where it reaches in its phase: a puff's box for its first five figures, the
    /// one pixel at its position never run, and nothing else.
    fn reach(self) -> Option<Reach> {
        match self.phase {
            VentPhase::Placed(_) => Some(Reach::of_placed(self.position)),
            VentPhase::Puffing(puff) => Reach::of_puff(puff.figure(), self.facing, self.position),
            VentPhase::AwaitingFirstPuff(_) | VentPhase::Resting(_) => None,
        }
    }

    /// `PuffTouches`: its hit on her: her protection spent and its reach touching her body. The
    /// push lasts 5 frames, and takes her away from the vent: `Right` when her x is the vent's or
    /// right of it, read signed, else `Left`. It takes no energy: the worth samus-health is
    /// handed with it is 0.
    fn hit(self, her: &Her) -> Option<Hit> {
        let touched = her.unprotected() && self.reach()?.touches(her);
        let right = !part_below(her.position.x.0, self.position.x.0);
        touched.then_some(Hit {
            side: if right { Side::Right } else { Side::Left },
            push: PUSH,
        })
    }

    /// One run of the vent: in view (`VentInView`), it first meets her with the phase it
    /// started the run in (`PuffTouches`), then steps (`VentWaits`, `VentPuffs`, `VentRests`);
    /// out of view nothing of it changes and it gives no hit.
    fn run(&mut self, camera: Pair<Px>, her: Option<&Her>, rules: &Rules) -> Option<Hit> {
        if !self.in_view(camera) {
            return None;
        }
        self.run_in_view(her, rules)
    }

    fn run_in_view(&mut self, her: Option<&Her>, rules: &Rules) -> Option<Hit> {
        let hit = her.and_then(|her| self.hit(her));
        self.phase = self.phase.stepped(rules.steam_rest);
        hit
    }
}

/// Spec: `VentDrawFollowsShaft`. The extra drawing state carried by a mounted vent.
/// Save `offset` alongside the vent's ordinary phase. Contact still uses the room anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MountedVent {
    /// Ordinary vent state at the mount, independent of its projected drawing offset.
    pub vent: Vent,
    /// The last offset for drawing the regular puff at the mount.
    pub offset: VentDrawOffset,
}

impl MountedVent {
    /// Mount a newly placed vent with no displacement yet.
    #[must_use]
    pub const fn new(vent: Vent) -> Self {
        Self {
            vent,
            offset: VentDrawOffset::ZERO,
        }
    }

    /// Spec: `VentDrawFollowsShaft`, `VentInView`. Call on an actor turn with the view as
    /// it stands before this vent runs. Outside the camera window neither the regular puff
    /// nor its offset changes. As with ordinary vents, omit `her` after an earlier hit.
    pub fn run(
        &mut self,
        camera: Pair<Px>,
        view: ShaftView,
        her: Option<&Her>,
        rules: &Rules,
    ) -> Option<Hit> {
        if !self.vent.in_view(camera) {
            return None;
        }
        self.offset = projected_vent_offset(self.vent.position, view);
        self.vent.run_in_view(her, rules)
    }
}

/// Spec: `VentDrawFollowsShaft`. Project the drawing origin around the incoming pivot.
/// Round each product down separately, including negative products.
fn projected_vent_offset(anchor: Pair<Px>, view: ShaftView) -> VentDrawOffset {
    const AFFINE_UNIT: i32 = 256;
    // Both pixel differences and every unrounded product fit in a signed 32-bit integer.
    let right = i32::from(anchor.x.0).wrapping_sub(i32::from(view.pivot.x.0));
    let up = i32::from(view.pivot.y.0).wrapping_sub(i32::from(anchor.y.0));
    let [diagonal, cross, ..] = view.matrix.map(i32::from);
    let scaled = |coefficient: i32, distance: i32| {
        coefficient.wrapping_mul(distance).div_euclid(AFFINE_UNIT)
    };
    VentDrawOffset(Pair {
        x: scaled(diagonal, right)
            .wrapping_add(scaled(cross, up))
            .wrapping_sub(right),
        y: up
            .wrapping_sub(scaled(cross.wrapping_neg(), right))
            .wrapping_sub(scaled(diagonal, up)),
    })
}

/// One run of the room's vents, on a frame the enemies run, each in the room's order.
///
/// It reads the camera as the last frame left it and her as the frame starts, if she is there
/// to reach, and answers with the frame's hit, the shared hit her energy and her motion read:
/// the first vent's in the room's order to touch her (`PuffTouches`).
pub fn run_vents(
    vents: &mut [Vent],
    camera: Pair<Px>,
    her: Option<&Her>,
    rules: &Rules,
) -> Option<Hit> {
    vents.iter_mut().fold(None, |hit, vent| {
        let touch = vent.run(camera, her, rules);
        hit.or(touch)
    })
}
