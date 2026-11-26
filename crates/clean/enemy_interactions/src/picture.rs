//! Shaft figure requests, independent of art and sprite placement.

use std::cmp::Reverse;

use serde::{Deserialize, Serialize};

/// Whole-pixel position in the room, wrapping on both axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomPoint {
    /// Horizontal position.
    pub x: u16,
    /// Vertical position.
    pub y: u16,
}

impl RoomPoint {
    pub(crate) const fn relative_to(self, view: Self) -> ScreenPoint {
        ScreenPoint {
            x: i16::from_ne_bytes(self.x.wrapping_sub(view.x).to_ne_bytes()),
            y: i16::from_ne_bytes(self.y.wrapping_sub(view.y).to_ne_bytes()),
        }
    }
}

/// View before scrolling, used for overlay visits and effect lifetime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreScrollView(pub RoomPoint);

/// View after scrolling, used for placing the requested figures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawView(pub RoomPoint);

/// Signed whole-pixel displacement supplied by the actor or room quake.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Displacement {
    /// Horizontal displacement.
    pub dx: i16,
    /// Vertical displacement.
    pub dy: i16,
}

/// A signed whole-pixel figure origin, not a sprite piece position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenPoint {
    /// Horizontal origin.
    pub x: i16,
    /// Vertical origin.
    pub y: i16,
}

const PICTURE_MIN: i16 = -128;
const PICTURE_MAX: i16 = 383;

impl ScreenPoint {
    pub(crate) const fn shifted(self, offset: Displacement) -> Self {
        Self {
            x: self.x.wrapping_add(offset.dx),
            y: self.y.wrapping_add(offset.dy),
        }
    }

    /// Spec: `PictureWindow`. Vents and effects use the same inclusive window.
    pub(crate) fn in_picture_window(self) -> bool {
        (PICTURE_MIN..=PICTURE_MAX).contains(&self.x)
            && (PICTURE_MIN..=PICTURE_MAX).contains(&self.y)
    }
}

/// An opaque figure identity from the art owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FigureId(pub u16);

/// Spec: `EffectSlots`. Stable index into the room's effect pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EffectSlot(pub(crate) usize);

impl EffectSlot {
    /// Index of this slot, from zero through seventeen.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Spec: `ShaftPictureOrder`. The mounted vent's stable room placement slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlacementSlot(pub usize);

/// Spec: `ShaftPictureOrder`. Sort ascending by this key, inserting Samus's
/// normal picture after the effects and overlay but before the mounted vents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShaftPictureOrder {
    /// Stationary effects go first, in descending slot order.
    Effect(Reverse<EffectSlot>),
    /// The overlay follows all stationary effects.
    Overlay,
    /// Mounted vents follow Samus, in ascending placement order.
    Vent(PlacementSlot),
}

/// Spec: `PictureWindow`, `ShaftPictureOrder`. The exact renderer resolves the
/// figure into sprite pieces and clips and places those pieces itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FigureRequest {
    /// Figure to place.
    pub figure: FigureId,
    /// Screen origin for that figure.
    pub origin: ScreenPoint,
    /// Position among other shaft requests and Samus's normal picture.
    pub order: ShaftPictureOrder,
}
