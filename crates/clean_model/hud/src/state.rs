//! What the HUD keeps from one frame to the next (§1): its tiles, the values last shown, the
//! selection and the explored cells. Nothing else carries over.

use crate::inputs::AMMO_KINDS;
use crate::item::SelectableItem;
use crate::map::MapCellSet;
use crate::pictures::HudGrid;

/// The values the HUD last showed: a value is drawn again only when it differs (§1).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Shown {
    /// Her energy.
    pub energy: u16,
    /// Her count of each kind of ammunition, in the order of `AMMO_KINDS`.
    pub ammo: [u16; AMMO_KINDS.len()],
    /// The item shown highlighted, or `None`.
    pub highlighted: Option<SelectableItem>,
}

/// The selected item, which she fires, and the one-shot item, which the weapons drop after one
/// shot of it (§1).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Selection {
    /// The selected item, or `None`.
    pub selected: Option<SelectableItem>,
    /// The one-shot item, or `None`.
    pub one_shot: Option<SelectableItem>,
}

impl Selection {
    /// Nothing selected and no one-shot item.
    pub const NONE: Self = Self {
        selected: None,
        one_shot: None,
    };
}

/// The HUD's state between frames. Each of the spec's rules is a step on it.
///
/// A new HUD has every tile blank, nothing selected or explored, and the values last shown 0
/// and none. Those zeroes are values it shows: a blank HUD stands for energy and counts of 0
/// and no highlight, so the first frame draws each value that differs from them, and a value of
/// 0 is drawn only once it changes. The HUD's real first contents are the load flow's and the
/// pause screen's to set.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct HudState {
    /// The HUD's tiles: the picture it draws.
    pub grid: HudGrid,
    /// The values last shown.
    pub shown: Shown,
    /// The selected and the one-shot item.
    pub selection: Selection,
    /// The cells of her map she has explored.
    pub explored: MapCellSet,
}
