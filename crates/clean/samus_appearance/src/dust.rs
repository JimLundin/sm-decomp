//! The wall-jump puff advances on fresh ordinary room drawings, not on publication.

use core::num;

use enum_map::{Enum, EnumMap, enum_map};
use serde::{Deserialize, Serialize};
use types::{Pair, Px, Side};

/// Whole room pixels, wrapping at 65,536 (`WallJumpDust`, `WallJumpDustPicture`).
pub type RoomPoint = Pair<Px>;

/// Spec: `WallJumpDust`. What Movement supplied for this drawing visit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DustArrival {
    /// No new wall-jumping pose or dry touchdown.
    None,
    /// A dry touchdown graphics opportunity clears the puff.
    DryTouchdown,
    /// A newly accepted wall-jumping pose, not a continuing wall jump.
    WallJump {
        /// The accepted pose's facing.
        facing: Side,
        /// Samus's room centre at the change.
        centre: RoomPoint,
        /// The accepted pose's vertical half-size, not the collision size after pose fit.
        half_height: u16,
    },
}

/// Spec: `WallJumpDust`. The puff's four ordered looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum DustLook {
    /// The look of a new puff.
    First,
    /// The second look.
    Second,
    /// The third look.
    Third,
    /// The final look, after which the puff disappears.
    Fourth,
}

impl DustLook {
    const fn next(self) -> Option<Self> {
        match self {
            Self::First => Some(Self::Second),
            Self::Second => Some(Self::Third),
            Self::Third => Some(Self::Fourth),
            Self::Fourth => None,
        }
    }
}

/// Spec: `WallJumpDust`. The number of drawing visits each look lasts, including its first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DustClip(pub EnumMap<DustLook, num::NonZeroU16>);

const FIRST_VISITS: u16 = 2;
const SECOND_VISITS: u16 = 3;
const THIRD_VISITS: u16 = 4;
const FOURTH_VISITS: u16 = 5;

impl DustClip {
    /// The reference game's four look lengths.
    #[must_use]
    pub fn reference() -> Self {
        const FIRST: num::NonZeroU16 = num::NonZeroU16::new(FIRST_VISITS).unwrap();
        const SECOND: num::NonZeroU16 = num::NonZeroU16::new(SECOND_VISITS).unwrap();
        const THIRD: num::NonZeroU16 = num::NonZeroU16::new(THIRD_VISITS).unwrap();
        const FOURTH: num::NonZeroU16 = num::NonZeroU16::new(FOURTH_VISITS).unwrap();
        Self(enum_map! {
            DustLook::First => FIRST,
            DustLook::Second => SECOND,
            DustLook::Third => THIRD,
            DustLook::Fourth => FOURTH,
        })
    }
}

/// Spec: `WallJumpDust`. A live puff, kept by the caller with the rest of appearance State.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dust {
    /// The puff's current room centre.
    pub centre: RoomPoint,
    /// The look it currently shows.
    pub look: DustLook,
    /// Visits left, including the one that showed this State.
    pub visits_left: num::NonZeroU16,
}

/// Spec: `WallJumpDustPicture`. What to submit to the prepared ordinary room picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DustPlacement {
    /// Do not write or consume a sprite position.
    None,
    /// Write only the horizontal coordinate at the next empty position, without consuming
    /// a sprite entry.
    /// Its vertical byte stays zero until another placement uses it or empty entries are hidden.
    HorizontalOnly {
        /// The horizontal coordinate in screen pixels.
        left: u8,
    },
    /// Append one small sprite, using the imported art for this look.
    SmallSprite {
        /// The horizontal coordinate in screen pixels.
        left: u8,
        /// The sprite's top edge in screen pixels.
        top: u8,
        /// The puff's current look.
        look: DustLook,
    },
}

/// Spec: `WallJumpDust`, `WallJumpDustPicture`. A dust drawing opportunity.
///
/// A fresh draw reads the Movement arrival, the post-scroll view and the imported
/// dust clip, independently of whether protection hides Samus's body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DustDrawing {
    /// No new ordinary drawing work, including a turn that only publishes prepared work.
    Skipped,
    /// Prepare the dust placement after the timer, room effects and overlay, and before
    /// Samus and the mounted vents.
    Fresh {
        /// Movement's observation for this drawing.
        arrival: DustArrival,
        /// The room's left and top after scrolling.
        view: RoomPoint,
        /// The four supplied visit lengths.
        clip: DustClip,
    },
}

impl DustDrawing {
    pub(crate) fn prepare(self, before: Option<Dust>) -> (Option<Dust>, DustPlacement) {
        let Self::Fresh {
            arrival,
            view,
            clip,
        } = self
        else {
            return (before, DustPlacement::None);
        };
        let after = Dust::visit(before, arrival, clip);
        let placement = after.map_or(DustPlacement::None, |dust| dust.placement(view));
        (after, placement)
    }
}

const WALL_OFFSET: Px = num::Wrapping(6);
const ABOVE_FEET: Px = num::Wrapping(1);
const RISE: Px = num::Wrapping(1);
const HALF_SPRITE: Px = num::Wrapping(4);

impl Dust {
    fn spawn(mut centre: RoomPoint, facing: Side, half_height: u16, clip: DustClip) -> Self {
        let offset = match facing {
            Side::Left => WALL_OFFSET,
            Side::Right => -WALL_OFFSET,
        };
        centre.x += offset;
        centre.y += num::Wrapping(half_height) - ABOVE_FEET;
        Self {
            centre,
            look: DustLook::First,
            visits_left: clip.0[DustLook::First],
        }
    }

    fn spend_visit(mut self, clip: DustClip) -> Option<Self> {
        if let Some(left) = num::NonZeroU16::new(self.visits_left.get() - 1) {
            self.visits_left = left;
        } else {
            self.look = self.look.next()?;
            self.visits_left = clip.0[self.look];
        }
        Some(self)
    }

    /// Spec: `WallJumpDust`. Replacement starts with a full first look. Touchdown or expiry
    /// removes the puff; every surviving puff rises even if its picture is clipped.
    pub(crate) fn visit(
        before: Option<Self>,
        arrival: DustArrival,
        clip: DustClip,
    ) -> Option<Self> {
        let mut dust = match arrival {
            DustArrival::None => before?.spend_visit(clip)?,
            DustArrival::DryTouchdown => return None,
            DustArrival::WallJump {
                facing,
                centre,
                half_height,
            } => Self::spawn(centre, facing, half_height, clip),
        };
        dust.centre.y -= RISE;
        Some(dust)
    }

    /// Spec: `WallJumpDustPicture`. Project the raised centre against the post-scroll view,
    /// without shaft tilt. Room arithmetic wraps before either screen coordinate is clipped.
    pub(crate) fn placement(self, view: RoomPoint) -> DustPlacement {
        let Ok(left) = u8::try_from((self.centre.x - view.x - HALF_SPRITE).0) else {
            return DustPlacement::None;
        };
        let Ok(top) = u8::try_from((self.centre.y - view.y - HALF_SPRITE).0) else {
            return DustPlacement::HorizontalOnly { left };
        };
        DustPlacement::SmallSprite {
            left,
            top,
            look: self.look,
        }
    }
}
