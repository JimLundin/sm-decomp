//! What the HUD keeps from one frame to the next (§1): its cells, the values last shown, the
//! selection, the explored cells and whether the mini-map is off. Nothing else carries over.

use enum_map::EnumMap;

use crate::item::{Ammo, SelectableItem};
use crate::map::MapCellSet;
use crate::tile::HudGrid;

/// The values the HUD last showed: a value is drawn again only when it differs (§1).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Shown {
    /// Her energy.
    pub energy: u16,
    /// Her count of each ammunition.
    pub ammo: EnumMap<Ammo, u16>,
    /// The item shown highlighted, or `None`.
    pub highlighted: Option<SelectableItem>,
}

/// The selected item, which she fires, and whether it is the one-shot item, which the weapons
/// drop after one shot of it (§1): the one-shot item is the selected item or none.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Selection {
    /// No item selected, and so no one-shot item.
    None,
    /// An item selected; there is no one-shot item.
    Item(SelectableItem),
    /// An item selected, and it is the one-shot item.
    OneShot(SelectableItem),
}

impl Selection {
    /// The selected item, or `None`.
    #[must_use]
    pub const fn item(self) -> Option<SelectableItem> {
        match self {
            Self::None => None,
            Self::Item(item) | Self::OneShot(item) => Some(item),
        }
    }

    /// The one-shot item, or `None`.
    #[must_use]
    pub const fn one_shot(self) -> Option<SelectableItem> {
        match self {
            Self::None | Self::Item(_) => None,
            Self::OneShot(item) => Some(item),
        }
    }
}

/// Whether the mini-map is off (§1): from when a boss turns it off until it is turned on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MiniMapPower {
    /// It updates on the frames it runs (spec: MiniMapStill).
    On,
    /// A boss turned it off: it doesn't update.
    Off,
}

/// The HUD's state between frames, each of the spec's rules a step on it.
///
/// A caller may set each part at any value the spec allows; the HUD's first contents are the
/// load flow's and the pause screen's to set.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct HudState {
    /// The HUD's cells: the picture it draws.
    pub grid: HudGrid,
    /// The values last shown.
    pub shown: Shown,
    /// The selected item, and whether it is the one-shot item.
    pub selection: Selection,
    /// The cells of her map she has explored.
    pub explored: MapCellSet,
    /// Whether the mini-map is off.
    pub mini_map: MiniMapPower,
}
