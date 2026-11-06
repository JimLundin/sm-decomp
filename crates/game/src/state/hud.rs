//! The HUD: its tilemap, the item selection it shows and the values it last drew.
//!
//! Bank `$80` log, handle HUD tilemap `$80:9B44`: the HUD redraws a counter only when its
//! value differs from the one it last drew, so the State keeps those values beside the
//! tilemap they were drawn into.

use canonical::Canonical;
use data::map::MapCells;
use data::room::{Area, MapPosition};

use super::video::{BgPalette, TilemapEntry};

/// The HUD tilemap's rows.
pub const HUD_ROWS: usize = 3;
/// A BG3 tilemap row's cells.
pub const HUD_COLUMNS: usize = 32;
/// The HUD tilemap's cells.
pub const HUD_CELLS: usize = HUD_ROWS * HUD_COLUMNS;

/// A HUD tilemap cell, by row and column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    /// From the top, below the top row the HUD's initialisation uploads once.
    pub row: usize,
    /// From the left.
    pub column: usize,
}

impl Cell {
    /// The cell `rows` down and `columns` right of this one.
    #[must_use]
    pub const fn offset(self, rows: usize, columns: usize) -> Self {
        Self {
            row: self.row.wrapping_add(rows),
            column: self.column.wrapping_add(columns),
        }
    }

    /// The cell's index in the tilemap, row-major, when it is inside.
    const fn index(self) -> Option<usize> {
        if self.row < HUD_ROWS && self.column < HUD_COLUMNS {
            Some(self.row.wrapping_mul(HUD_COLUMNS).wrapping_add(self.column))
        } else {
            None
        }
    }
}

/// The HUD's three tilemap rows, which every gameplay frame queues to VRAM.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HudTilemap {
    /// The cells, row-major.
    pub cells: [TilemapEntry; HUD_CELLS],
}

impl Default for HudTilemap {
    fn default() -> Self {
        Self {
            cells: [TilemapEntry::default(); HUD_CELLS],
        }
    }
}

impl HudTilemap {
    /// The entry at `cell`, when it is inside the tilemap.
    #[must_use]
    pub fn get(&self, cell: Cell) -> Option<TilemapEntry> {
        cell.index()
            .and_then(|index| self.cells.get(index))
            .copied()
    }

    /// Set the entry at `cell`; a cell outside the tilemap is left alone.
    pub fn set(&mut self, cell: Cell, entry: TilemapEntry) {
        if let Some(slot) = cell.index().and_then(|index| self.cells.get_mut(index)) {
            *slot = entry;
        }
    }

    /// The tilemap as the words VRAM holds, row-major.
    #[must_use]
    pub fn words(&self) -> Vec<u16> {
        self.cells.iter().map(|entry| entry.word()).collect()
    }
}

/// An item the HUD can select, in the HUD's left-to-right order.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudItem {
    /// Missiles.
    Missiles,
    /// Super missiles.
    SuperMissiles,
    /// Power bombs.
    PowerBombs,
    /// The grapple beam.
    GrappleBeam,
    /// The X-ray scope.
    XRayScope,
}

/// The values the HUD last drew, which it compares against to decide what to redraw.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Drawn {
    /// Energy.
    pub energy: u16,
    /// Missiles.
    pub missiles: u16,
    /// Super missiles.
    pub super_missiles: u16,
    /// Power bombs.
    pub power_bombs: u16,
    /// The highlighted item.
    pub selection: Option<HudItem>,
}

/// The HUD.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Hud {
    /// Rows 1 to 3 of the BG3 tilemap.
    pub tilemap: HudTilemap,
    /// The item the player has selected.
    // open: the selection joins with weapons, which switch it on Select and fire it.
    pub selected: Option<HudItem>,
    /// The item that cancels itself automatically, whose icon flashes.
    pub auto_cancel: Option<HudItem>,
    /// What the HUD last drew.
    pub drawn: Drawn,
    /// The palette the last item highlight wrote.
    pub item_palette: BgPalette,
    /// The mini-map.
    pub mini_map: MiniMap,
}

/// What the mini-map draws from (bank $90 log, update mini-map `$90:A91B`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MiniMap {
    /// The current area's explored cells, which the mini-map marks as Samus moves.
    // open: the other areas' explored cells and the copy on area change join with `progress`.
    pub explored: MapCells,
    /// The current area.
    pub area: Area,
    /// The current room's top-left cell on the area map.
    pub room: MapPosition,
    /// Whether the current area's map has been collected, so the cells its station reveals
    /// show.
    pub map_collected: bool,
    /// Whether a boss room has turned the mini-map off (`$90:A7E2`).
    pub disabled: bool,
}
