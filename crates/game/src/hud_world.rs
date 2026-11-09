//! The HUD as the engine meets it: the HUD model (`clean_model_hud`).
//!
//! The engine keeps the HUD as the game keeps it (`docs/behaviours/hud.decode.md`, "State"): the
//! HUD tilemap's three rows, the values it last drew, the selected and the auto-cancel item, and
//! the explored cells. The model's `HudState` is built from those each turn and written back after
//! it; the area maps are the model's, converted once from the ROM's when the content loads.
//!
//! The original's HUD runs at two turns of the gameplay pass that the PLM pass lies between, and
//! an item's message box called out of the PLM pass takes NMIs, so the frame count moves between
//! them. The mini-map's turn follows her moves (`mini_map_turn`); the HUD phase draws the status
//! bar and changes the selection (`hud_turn`). Each runs the model's frame and keeps its own part:
//! the mini-map's turn the window and the explored cells, the HUD phase the rest, with the
//! mini-map held as its turn left it.

use clean_model_hud::{
    Ammunition, CancelButton, ColourSet, Controls, Energy, Equipped, Flip, FrameCount, FrameInputs,
    HerMap, HudPos, HudState, MapCell, MapCellSet, MapPicture, MiniMapInputs, Mute, Picture,
    PicturePos, Reserve, ReserveMode, Room, Rules, SelectButton, SelectableItem, Selection, Shown,
    Status, Stock, Tile,
};
use clean_types::Pair;
use data::dma::HUD_TILEMAP_BUFFER;
use data::map::{MAP_TILEMAP_WORDS, MapCells};
use data::pose::transition::Control;
use data::room::Area;

use crate::pose::{self, Content, FACING_FORWARD};
use crate::state::State;
use crate::state::hud::{Drawn, Hud, HudItem, HudTilemap};
use crate::state::samus::{Gauge, Item, MovementType, Point, ReserveMode as Refill};
use crate::state::video::{
    BgPalette, TileNumber, TilemapEntry, VramSource, VramStep, VramTransfer, VramWord,
};
use crate::state::{Button, Controller};

/// Where the queued tilemap lands: the HUD's second BG3 row (`$80:9CC1`).
pub const HUD_TILEMAP_VRAM: VramWord = VramWord(0x5820);
/// The mini-map's window (spec §1.2): rows 0 to 2, columns 26 to 30.
const WINDOW_COLUMNS: core::ops::RangeInclusive<u8> = 26..=30;
/// A map tilemap page's columns and words, and a row's words.
const PAGE_CELLS: usize = 32;
const PAGE_WORDS: usize = 0x400;
const ROW_WORDS: usize = 0x20;

/// Why the HUD can't run a turn over the engine's State.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudFault {
    /// A count or capacity past what the HUD's cells hold: 999 missiles, 99 super missiles or
    /// power bombs (spec §1.5; the original reads past its digit table, docs/LEGACY.md).
    CountPastCells,
    /// Her room's top-left cell lies off the 64 by 32 map.
    RoomOffMap,
    /// A HUD tilemap cell's palette is past the eight colour sets.
    Colours(BgPalette),
}

/// Every area's map as the model reads it, and the blank map of an area with none: the debug
/// area's pointers name no map.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AreaMaps {
    maps: Vec<(Area, HerMap)>,
    blank: HerMap,
}

/// A map tilemap word's picture and flips; the mini-map chooses its colours.
const fn picture(word: u16) -> MapPicture {
    let entry = TilemapEntry::from_word(word);
    MapPicture {
        picture: Picture::new(entry.tile.0),
        flip: flip(entry.flip_x, entry.flip_y),
    }
}

/// The model's flips from a tilemap entry's.
const fn flip(flip_x: bool, flip_y: bool) -> Flip {
    match (flip_x, flip_y) {
        (false, false) => Flip::None,
        (true, false) => Flip::Horizontal,
        (false, true) => Flip::Vertical,
        (true, true) => Flip::Both,
    }
}

/// The ROM's word for the picture at `pos`. The beyond words follow the map's two pages as a
/// third, so the map's columns 64 and 65 are that page's first two.
fn picture_word(map: &data::map::AreaMap, pos: PicturePos) -> Option<u16> {
    let (column, row) = (usize::from(pos.x()), usize::from(pos.y()));
    let word = (column / PAGE_CELLS)
        .wrapping_mul(PAGE_WORDS)
        .wrapping_add(row.wrapping_mul(ROW_WORDS))
        .wrapping_add(column % PAGE_CELLS);
    word.checked_sub(MAP_TILEMAP_WORDS)
        .map_or_else(|| map.tiles.get(word), |beyond| map.beyond.get(beyond))
        .copied()
}

/// The model's map of `map`.
fn import(map: &data::map::AreaMap) -> HerMap {
    let mut model = HerMap::default();
    for pos in PicturePos::all() {
        if let Some(word) = picture_word(map, pos) {
            model.pictures.set(pos, picture(word));
        }
    }
    model.charted = explored(&map.station);
    model
}

impl AreaMaps {
    /// The model's maps of the ROM's.
    #[must_use]
    pub fn import(maps: &data::map::AreaMaps) -> Self {
        Self {
            maps: maps.0.iter().map(|map| (map.area, import(map))).collect(),
            blank: HerMap::default(),
        }
    }

    /// `area`'s map; a blank map that charts nothing for an area with none.
    #[must_use]
    pub fn of(&self, area: Area) -> &HerMap {
        self.maps
            .iter()
            .find(|(each, _)| *each == area)
            .map_or(&self.blank, |(_, map)| map)
    }
}

/// Whether the engine's set holds `cell`.
fn contains(cells: &MapCells, cell: MapCell) -> bool {
    cells
        .0
        .get(usize::from(cell.y()))
        .and_then(|word| word.checked_shr(u32::from(cell.x())))
        .is_some_and(|word| word & 1 != 0)
}

/// The model's set of the engine's cells.
#[must_use]
pub fn explored(cells: &MapCells) -> MapCellSet {
    let mut set = MapCellSet::default();
    for cell in MapCell::all().filter(|&cell| contains(cells, cell)) {
        set.set(cell, true);
    }
    set
}

/// The engine's set of the model's cells.
fn engine_cells(set: &MapCellSet) -> MapCells {
    let mut cells = MapCells::default();
    for cell in MapCell::all().filter(|&cell| set.get(cell)) {
        if let (Some(word), Some(bit)) = (
            cells.0.get_mut(usize::from(cell.y())),
            1_u64.checked_shl(u32::from(cell.x())),
        ) {
            *word |= bit;
        }
    }
    cells
}

/// Whether the tile at `pos` is the mini-map's window's.
fn in_window(pos: HudPos) -> bool {
    WINDOW_COLUMNS.contains(&pos.x())
}

/// The model's tile for a tilemap cell. The HUD draws every word above the background, so its
/// priority is not the model's.
///
/// # Errors
///
/// [`HudFault::Colours`] for a palette past the eight colour sets.
pub fn tile(entry: TilemapEntry) -> Result<Tile, HudFault> {
    let colours = ColourSet::new(entry.palette.0).ok_or(HudFault::Colours(entry.palette))?;
    Ok(Tile::new(
        Picture::new(entry.tile.0),
        colours,
        flip(entry.flip_x, entry.flip_y),
    ))
}

/// The tilemap cell for the model's tile, above the background.
const fn entry(tile: Tile) -> TilemapEntry {
    TilemapEntry {
        tile: TileNumber(tile.picture.number()),
        palette: BgPalette(tile.colours.number()),
        priority: true,
        flip_x: tile.flip.horizontal(),
        flip_y: tile.flip.vertical(),
    }
}

/// The model's item for the engine's.
const fn item(item: HudItem) -> SelectableItem {
    match item {
        HudItem::Missiles => SelectableItem::Missiles,
        HudItem::SuperMissiles => SelectableItem::SuperMissiles,
        HudItem::PowerBombs => SelectableItem::PowerBombs,
        HudItem::GrappleBeam => SelectableItem::GrappleBeam,
        HudItem::XRayScope => SelectableItem::XRayScope,
    }
}

/// The engine's item for the model's.
const fn hud_item(item: SelectableItem) -> HudItem {
    match item {
        SelectableItem::Missiles => HudItem::Missiles,
        SelectableItem::SuperMissiles => HudItem::SuperMissiles,
        SelectableItem::PowerBombs => HudItem::PowerBombs,
        SelectableItem::GrappleBeam => HudItem::GrappleBeam,
        SelectableItem::XRayScope => HudItem::XRayScope,
    }
}

/// The model's HUD of the engine's.
fn model(hud: &Hud) -> Result<HudState, HudFault> {
    let mut state = HudState {
        shown: Shown {
            energy: hud.drawn.energy,
            ammo: [
                hud.drawn.missiles,
                hud.drawn.super_missiles,
                hud.drawn.power_bombs,
            ],
            highlighted: hud.drawn.selection.map(item),
        },
        selection: Selection {
            selected: hud.selected.map(item),
            one_shot: hud.auto_cancel.map(item),
        },
        explored: explored(&hud.mini_map.explored),
        ..HudState::default()
    };
    for (pos, &cell) in HudPos::all().zip(&hud.tilemap.cells) {
        state.grid.set(pos, tile(cell)?);
    }
    Ok(state)
}

/// Writes the tiles `keep` names that the turn changed back to the engine's tilemap, leaving
/// every other cell's word as it was.
fn write_tiles(
    tilemap: &mut HudTilemap,
    before: &HudState,
    after: &HudState,
    keep: fn(HudPos) -> bool,
) {
    for (pos, cell) in HudPos::all().zip(&mut tilemap.cells) {
        let drawn = after.grid.get(pos);
        if keep(pos) && drawn != before.grid.get(pos) {
            *cell = entry(drawn);
        }
    }
}

/// The ammunition's stock in the engine's gauge.
const fn stock(gauge: Gauge) -> Stock {
    Stock {
        count: gauge.current,
        capacity: gauge.max,
    }
}

/// Item select this frame: its default binding, Select.
fn select(controller: &Controller) -> SelectButton {
    if controller.pressed.contains(Button::Select) {
        SelectButton::Pressed
    } else {
        SelectButton::NotPressed
    }
}

/// Item cancel this frame, under the controls' bindings.
fn cancel(controller: &Controller) -> CancelButton {
    if pose::controls(controller.pressed).contains(Control::ItemCancel) {
        CancelButton::Pressed
    } else if pose::controls(controller.held).contains(Control::ItemCancel) {
        CancelButton::Held
    } else {
        CancelButton::NotPressed
    }
}

/// Why a switch makes no click: she spin jumps or wall jumps.
// open: the grapple beam out and time stopped join with the grapple and the x-ray scope.
fn muted_by(movement: MovementType) -> Option<Mute> {
    if movement == MovementType::SpinJumping {
        Some(Mute::SpinJump)
    } else if movement == MovementType::WallJumping {
        Some(Mute::WallJump)
    } else {
        None
    }
}

/// What the model reads this turn from the engine's State; the mini-map runs when `runs`.
///
/// She can switch while the normal current state handler runs her (the elevator's locked
/// handler doesn't) and she doesn't face the screen. The engine fires no missiles, super
/// missiles or power bombs, so no shot cancels the selection; its one item, the morph ball,
/// draws no icon, and it has no boss that turns the mini-map off in a frame.
fn inputs<'map>(
    state: &State,
    maps: &'map AreaMaps,
    runs: bool,
) -> Result<FrameInputs<'map>, HudFault> {
    let samus = &state.samus;
    let equipment = &samus.equipment;
    let ammunition = Ammunition::new([
        stock(equipment.missiles),
        stock(equipment.super_missiles),
        stock(equipment.power_bombs),
    ])
    .ok_or(HudFault::CountPastCells)?;
    let status = Status {
        energy: Energy {
            now: equipment.energy.current,
            capacity: equipment.energy.max,
        },
        reserve: Reserve {
            energy: equipment.reserve.current,
            mode: match equipment.reserve_mode {
                Refill::Automatic => ReserveMode::Auto,
                Refill::None | Refill::Manual => ReserveMode::Manual,
            },
        },
        ammunition,
        equipped: Equipped {
            grapple_beam: equipment.equipped.contains(Item::GrappleBeam),
            x_ray_scope: equipment.equipped.contains(Item::XRayScope),
        },
    };
    let controls = Controls {
        select: select(&state.controller),
        cancel: cancel(&state.controller),
        can_switch: !pose::carried(samus, state.elevator) && !FACING_FORWARD.contains(&samus.pose),
        shot_cancels: false,
        muted_by: muted_by(samus.movement),
    };
    let mini_map = &state.hud.mini_map;
    let room = &state.room;
    let width = room.width.0;
    let blocks = u16::try_from(room.blocks.len()).unwrap_or(u16::MAX);
    let at = Point::from(samus.motion.position);
    // Rooms start a row down: the map's row 0 is padding.
    let cell = MapCell::new(
        i32::from(mini_map.room.x),
        i32::from(mini_map.room.y).wrapping_add(1),
    )
    .ok_or(HudFault::RoomOffMap)?;
    Ok(FrameInputs {
        frame: FrameCount(state.clock.counters.requested_byte),
        status,
        controls,
        collected: None,
        boss: None,
        mini_map: MiniMapInputs {
            runs,
            off: mini_map.disabled,
            position: Pair {
                x: at.x.pixels,
                y: at.y.pixels,
            },
            room: Room {
                cell,
                blocks: Pair {
                    x: width,
                    y: blocks.checked_div(width).unwrap_or(0),
                },
            },
            map: maps.of(mini_map.area),
            map_collected: mini_map.map_collected,
        },
    })
}

/// The mini-map's turn, after her moves (`$90:E734`): it marks her cell explored and draws the
/// window around her.
///
/// # Errors
///
/// A [`HudFault`] when the State holds what the model isn't defined on.
pub fn mini_map_turn(state: &mut State, content: &Content) -> Result<(), HudFault> {
    let before = model(&state.hud)?;
    let inputs = inputs(state, &content.area_maps, true)?;
    let after = before.clone().frame(&Rules::reference(), &inputs).state;
    let hud = &mut state.hud;
    write_tiles(&mut hud.tilemap, &before, &after, in_window);
    if after.explored != before.explored {
        hud.mini_map.explored = engine_cells(&after.explored);
    }
    Ok(())
}

/// The HUD phase (`$80:9B44`): the selection changes, the status bar is drawn, and the tilemap's
/// upload is queued. The mini-map has had its turn.
///
/// # Errors
///
/// A [`HudFault`] when the State holds what the model isn't defined on.
// open: the weapons and her look read `Switched`, the sound domain the click; the engine has
// neither yet, so the events go unread.
pub fn hud_turn(state: &mut State, content: &Content) -> Result<(), HudFault> {
    let before = model(&state.hud)?;
    let inputs = inputs(state, &content.area_maps, false)?;
    let after = before.clone().frame(&Rules::reference(), &inputs).state;
    let hud = &mut state.hud;
    write_tiles(&mut hud.tilemap, &before, &after, |_| true);
    let Shown {
        energy,
        ammo: [missiles, super_missiles, power_bombs],
        highlighted,
    } = after.shown;
    hud.drawn = Drawn {
        energy,
        missiles,
        super_missiles,
        power_bombs,
        selection: highlighted.map(hud_item),
    };
    hud.selected = after.selection.selected.map(hud_item);
    hud.auto_cancel = after.selection.one_shot.map(hud_item);
    let words = hud.tilemap.cells.iter().map(|entry| entry.word()).collect();
    state.video.batch.vram.push(VramTransfer {
        to: HUD_TILEMAP_VRAM,
        step: VramStep::Row,
        source: VramSource::Words {
            from: HUD_TILEMAP_BUFFER,
            words,
        },
    });
    Ok(())
}

#[cfg(test)]
mod tests;
