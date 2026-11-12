//! The HUD: its tilemap, the item selection it shows and the values it last drew.
//!
//! Bank `$80` log, handle HUD tilemap `$80:9B44`: the HUD redraws a counter only when its
//! value differs from the one it last drew, so the State keeps those values beside the
//! tilemap they were drawn into. The HUD model (`interface_hud_bridge`) draws them.

use canonical::Canonical;
use data::map::MapCells;
use data::room::{Area, MapPosition};

use super::video::TilemapEntry;

/// The HUD tilemap's cells: three rows of a BG3 tilemap's 32.
pub const HUD_CELLS: usize = 3 * 32;

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
    pub selected: Option<HudItem>,
    /// The item that cancels itself automatically, whose icon flashes.
    pub auto_cancel: Option<HudItem>,
    /// What the HUD last drew.
    pub drawn: Drawn,
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
