//! Following her (Spec: §2): how positions compare, where the view aims, and how fast and how far
//! it moves there.

use clean_types::{Fixed, Pair, Parts, Px, below, px};

use crate::input::Frame;
use crate::rules::Rules;

/// Whether position `a` is before position `b`: bit 15 of a − b, mod 2^16, is set (Spec:
/// FollowingHer). So a position left of or above the room (FFF0) is before 0, and of two
/// positions 8000 apart each is before the other.
pub(crate) const fn before(a: Px, b: Px) -> bool {
    below(a.0, b.0)
}

/// Whether position `a` exceeds position `b`, both read as plain numbers from 0 to FFFF (Spec:
/// FollowingHer). So FFF0 exceeds every position in the room.
pub(crate) const fn exceeds(a: Px, b: Px) -> bool {
    a.0 > b.0
}

/// The spec's `max`, picking by `before`: `b` when `a` is before it, `a` otherwise.
pub(crate) const fn max_by_before(a: Px, b: Px) -> Px {
    if before(a, b) { b } else { a }
}

/// The spec's `min` by `exceeds`, picking as plain numbers: `b` when `a` exceeds it, `a`
/// otherwise.
pub(crate) const fn min_by_exceeds(a: Px, b: Px) -> Px {
    if exceeds(a, b) { b } else { a }
}

/// Which way the camera moves on an axis, towards its aim (Spec: FollowingHer).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Going {
    /// Left or up.
    Back,
    /// Right or down.
    Forward,
}

impl Going {
    /// The way from the camera's pixel part to its aim's: none when they are the same, back when
    /// the aim is before the camera (so an aim exactly 8000 away is back), forward otherwise.
    fn towards(camera: Px, aim: Px) -> Option<Self> {
        if aim == camera {
            None
        } else if before(aim, camera) {
            Some(Self::Back)
        } else {
            Some(Self::Forward)
        }
    }

    /// Where a move this way by `speed` takes the camera, carrying through the subpixels.
    fn by(self, camera: Fixed, speed: Fixed) -> Fixed {
        match self {
            Self::Back => camera - speed,
            Self::Forward => camera + speed,
        }
    }

    /// Whether a move this way that reached `reached` went beyond the aim: going forward, the aim
    /// is before it; going back, it is before the aim.
    const fn beyond(self, reached: Px, aim: Px) -> bool {
        match self {
            Self::Back => before(reached, aim),
            Self::Forward => before(aim, reached),
        }
    }
}

/// The camera's move on an axis, before the room limits it (Spec: FollowingHer, §3).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Move {
    /// Which way it moved.
    pub(crate) going: Going,
    /// The pixel part the move took it to, before the aim stopped it (the spec's cm).
    pub(crate) reached: Px,
    /// Where the move ends: where it took it, or on the aim, subpixel part 0, when it went
    /// beyond the aim.
    pub(crate) stopped: Fixed,
}

/// The point the view aims its top-left corner at (Spec: LeadingHer, AimingHeight): across, her
/// x less the lead the way she travels; down, her y less the room's height for whether she
/// rises.
pub(crate) fn aim(rules: &Rules, frame: &Frame) -> Pair<Px> {
    let samus = &frame.samus;
    // Going backwards turns the lead round, to the way she travels.
    let travel = if samus.backwards {
        samus.facing.back()
    } else {
        samus.facing
    };
    Pair {
        x: samus.position.x.pixel() - rules.lead(frame.lead, travel),
        y: samus.position.y.pixel() - frame.room.heights.height(samus.rising),
    }
}

/// The camera's speed on an axis: how far she moved from where it last saw her, `her` and `seen`
/// on that axis, on the whole P.S value mod 2^32, plus the extra pixel (Spec: FollowingHer). Its
/// way is read from the pixel parts alone: the seen value less hers when her pixel part is before
/// the seen one, hers less the seen one otherwise.
///
/// Spec: CreepReadsRightward. Reproduces the original's behaviour: a creep left or up inside a
/// pixel leaves her pixel part where it was seen, so it isn't before the seen one and the move
/// reads as hers less the seen one. That is under a pixel below 0, `FFFF.xxxx` mod 2^32, and the
/// extra pixel carries it round to `0.xxxx`: a speed under a pixel, its pixel part 0, so the view
/// settles 2 px, not 3.
pub(crate) fn speed(rules: &Rules, her: Fixed, seen: Fixed) -> Fixed {
    let moved = if before(her.pixel(), seen.pixel()) {
        seen - her
    } else {
        her - seen
    };
    moved + rules.extra
}

/// The camera's move towards its aim by `speed`, stopping on the aim when it would go beyond it
/// (Spec: FollowingHer).
///
/// None on a frame where the aim is the view's own pixel, its pixel part the camera's: the camera
/// stays as it is, subpixel part included, and, as she moved, it doesn't settle either
/// (C-at-aim-across-rising, C-at-aim-down, C-at-aim-down-held-across, C-at-aim-across).
pub(crate) fn chase(camera: Fixed, aim: Px, speed: Fixed) -> Option<Move> {
    let going = Going::towards(camera.pixel(), aim)?;
    let moved = going.by(camera, speed);
    let reached = moved.pixel();
    let stopped = if going.beyond(reached, aim) {
        px(aim)
    } else {
        moved
    };
    Some(Move {
        going,
        reached,
        stopped,
    })
}
