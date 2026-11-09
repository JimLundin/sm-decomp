//! What the HUD keeps from one frame to the next (§1): its tiles, the values last shown, the
//! selection and the explored cells. Nothing else carries over.

use crate::grid::HudGrid;
use crate::inputs::Ammo;
use crate::item::SelectableItem;
use crate::map::MapCellSet;
use crate::pictures::BLANK;

/// The values the HUD last showed: a value is drawn again only when it differs (§1).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Shown {
    /// Her energy.
    pub energy: u16,
    /// Her missiles.
    pub missiles: u16,
    /// Her super missiles.
    pub super_missiles: u16,
    /// Her power bombs.
    pub power_bombs: u16,
    /// The item whose icon is highlighted, or `None`.
    pub selection: Option<SelectableItem>,
}

impl Shown {
    /// The count last shown of one ammunition.
    #[must_use]
    pub const fn ammo(&self, ammo: Ammo) -> u16 {
        match ammo {
            Ammo::Missiles => self.missiles,
            Ammo::SuperMissiles => self.super_missiles,
            Ammo::PowerBombs => self.power_bombs,
        }
    }

    /// Sets the count last shown of one ammunition.
    pub const fn set_ammo(&mut self, ammo: Ammo, count: u16) {
        match ammo {
            Ammo::Missiles => self.missiles = count,
            Ammo::SuperMissiles => self.super_missiles = count,
            Ammo::PowerBombs => self.power_bombs = count,
        }
    }
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

/// The HUD's state between frames.
#[derive(Clone, PartialEq, Eq, Debug)]
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

impl Default for HudState {
    /// Every tile blank, nothing shown, selected or explored. The HUD's first contents are the
    /// load flow's and the pause screen's to set.
    fn default() -> Self {
        Self {
            grid: HudGrid::filled(BLANK),
            shown: Shown::default(),
            selection: Selection::NONE,
            explored: MapCellSet::EMPTY,
        }
    }
}
