//! The HUD as the engine meets it: the HUD model (`clean_model_hud`).
//!
//! The engine keeps the HUD as the game keeps it (`docs/behaviours/hud.decode.md`, "State"): the
//! HUD tilemap's three rows, the values it last drew, the selected and the auto-cancel item, the
//! explored cells and whether the mini-map is off. The model's `HudState` is built from those
//! each turn and written back after it. The HUD's pictures are the importer's looks; the area
//! maps' tilemap words become the model's maps once, when the content loads.
//!
//! The original's HUD runs at two turns of the gameplay pass that the PLM pass lies between, and
//! an item's message box called out of the PLM pass takes NMIs, so the frame count moves between
//! them. The mini-map's turn follows her moves (`mini_map_turn`); the HUD phase draws the status
//! bar and changes the selection (`hud_turn`). Each runs the model's frame and keeps its own part:
//! the mini-map's turn the window, the explored cells and the mini-map's off, the HUD phase the
//! rest, with the mini-map held as its turn left it.

use clean_model_hud::{
    Ammo, Ammunition, CancelButton, ColourSet, Controls, Energy, Equipped, Flip, FrameCount,
    FrameInputs, HerMap, HudGrid, HudPictures, HudState, Icons, MapCellSet, MapPictures,
    MiniMapInputs, MiniMapPower, Mute, Place, Reserve, ReserveMode, Room, Rules, SelectButton,
    SelectableItem, Selection, Shown, Status, Stock, Tile,
};
use clean_types::{Look, Pair, Picture};
use data::map::{MAP_COLUMNS, MAP_ROWS, MAP_TILEMAP_WORDS, MapCells};
use data::pose::transition::Control;
use data::room::Area;
use enum_map::enum_map;

use crate::exact::tilemap::TileQueue;
use crate::intent::TileWrite;
use crate::pose::{self, Content, FACING_FORWARD};
use crate::state::State;
use crate::state::hud::{Drawn, Hud, HudItem, HudTilemap};
use crate::state::samus::{Gauge, Item, MovementType, Point as Position, ReserveMode as Refill};
use crate::state::video::{BgPalette, TileNumber, TilemapEntry};
use crate::state::{Button, Controller};

/// The overlay's row the HUD's cells start at: its second (`$80:9CC1`).
const HUD_ROW: u16 = 1;
/// The mini-map's window (spec §1.2): rows 0 to 2, columns 26 to 30.
const WINDOW_COLUMNS: core::ops::RangeInclusive<i32> = 26..=30;
/// The HUD's cells across a row.
const HUD_COLUMNS: usize = 32;
/// The columns of her map's pictures: the map's, then the two beyond it the window reaches.
const PICTURE_COLUMNS: usize = 66;
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
    /// An auto-cancel item that isn't the selected one, which the original never holds.
    OneShotNotSelected,
    /// A HUD tilemap cell that is no model tile: a palette past the eight colour sets, or a
    /// tile past the 400h pictures.
    Cell(TilemapEntry),
}

/// The model's HUD pictures of the importer's, which hold the same looks by the same names.
const fn hud_pictures(imported: &data::hud::HudPictures) -> HudPictures {
    let icons = &imported.icons;
    HudPictures {
        blank: imported.blank,
        digits: imported.digits,
        full_tank: imported.full_tank,
        empty_tank: imported.empty_tank,
        reserve_sign: imported.reserve_sign,
        icons: Icons {
            missiles: icons.missiles,
            super_missiles: icons.super_missiles,
            power_bombs: icons.power_bombs,
            grapple_beam: icons.grapple_beam,
            x_ray_scope: icons.x_ray_scope,
        },
        map_blank: imported.map_blank,
        diagonal: imported.diagonal,
    }
}

/// The look of a tilemap entry: its picture and flips; its colours are the HUD's to choose.
fn look(entry: TilemapEntry) -> Option<Look> {
    Some(Look {
        picture: Picture::new(entry.tile.0)?,
        flip: match (entry.flip_x, entry.flip_y) {
            (false, false) => Flip::None,
            (true, false) => Flip::Across,
            (false, true) => Flip::Down,
            (true, true) => Flip::Both,
        },
    })
}

/// The place `column` across and `row` down a grid.
fn place(column: usize, row: usize) -> Place {
    let coordinate = |value: usize| i32::try_from(value).unwrap_or(i32::MAX);
    Pair {
        x: coordinate(column),
        y: coordinate(row),
    }
}

/// The model's pictures of `map`, `none` where it has no word: column `c` of row `y` is word
/// `c div 32 × 400h + y × 20h + c mod 32`, and the beyond words follow the map's two pages as a
/// third, so the map's columns 64 and 65 are that page's first two.
fn map_pictures(map: &data::map::AreaMap, none: Look) -> MapPictures {
    let mut pictures = MapPictures::filled(none);
    for row in 0..usize::from(MAP_ROWS) {
        for column in 0..PICTURE_COLUMNS {
            let word = (column / PAGE_CELLS)
                .wrapping_mul(PAGE_WORDS)
                .wrapping_add(row.wrapping_mul(ROW_WORDS))
                .wrapping_add(column % PAGE_CELLS);
            let read = word
                .checked_sub(MAP_TILEMAP_WORDS)
                .map_or_else(|| map.tiles.get(word), |beyond| map.beyond.get(beyond));
            if let Some(look) = read.and_then(|&word| look(TilemapEntry::from_word(word))) {
                pictures.set(place(column, row), look);
            }
        }
    }
    pictures
}

/// The HUD's content as the model reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Imported {
    maps: Vec<(Area, HerMap)>,
    /// The map of an area with none, picture 0 everywhere and nothing charted: the debug area's
    /// pointers name no map.
    blank: HerMap,
    pictures: HudPictures,
}

/// What the HUD draws with: every area's map, and the HUD's pictures; with nothing imported the
/// HUD has nothing to draw, and its turns leave the State as it is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HudContent(Option<Imported>);

impl HudContent {
    /// The model's content of the importer's maps and pictures.
    #[must_use]
    pub fn import(maps: &data::map::AreaMaps, pictures: &data::hud::HudPictures) -> Self {
        let Some(none) = Picture::new(0).map(Look::unflipped) else {
            return Self::default();
        };
        let her_map = |map: &data::map::AreaMap| HerMap {
            pictures: map_pictures(map, none),
            charted: explored(&map.station),
        };
        Self(Some(Imported {
            maps: maps.0.iter().map(|map| (map.area, her_map(map))).collect(),
            blank: HerMap {
                pictures: MapPictures::filled(none),
                charted: MapCellSet::default(),
            },
            pictures: hud_pictures(pictures),
        }))
    }

    /// `area`'s map, a blank one that charts nothing for an area with none, and the HUD's
    /// pictures; `None` with nothing imported.
    #[must_use]
    pub fn of(&self, area: Area) -> Option<(&HerMap, &HudPictures)> {
        let imported = self.0.as_ref()?;
        let map = imported
            .maps
            .iter()
            .find(|(each, _)| *each == area)
            .map_or(&imported.blank, |(_, map)| map);
        Some((map, &imported.pictures))
    }
}

/// The model's set of the engine's cells: row `y`'s word holds column `x` at bit `x`.
#[must_use]
pub fn explored(cells: &MapCells) -> MapCellSet {
    let mut set = MapCellSet::default();
    for (row, word) in cells.0.iter().enumerate() {
        for column in (0..MAP_COLUMNS).filter(|&column| (word >> column) & 1 != 0) {
            set.insert(place(usize::from(column), row));
        }
    }
    set
}

/// The engine's set of the model's cells.
fn engine_cells(set: &MapCellSet) -> MapCells {
    let mut cells = MapCells::default();
    for (row, word) in cells.0.iter_mut().enumerate() {
        for column in
            (0..MAP_COLUMNS).filter(|&column| set.contains(place(usize::from(column), row)))
        {
            *word |= 1 << column;
        }
    }
    cells
}

/// Whether the HUD cell at `place` is the mini-map's window's.
fn in_window(place: Place) -> bool {
    WINDOW_COLUMNS.contains(&place.x)
}

/// The model's tile for a tilemap cell. The HUD draws every word above the background, so its
/// priority is not the model's.
///
/// # Errors
///
/// [`HudFault::Cell`] for a palette past the eight colour sets or a tile past the pictures.
pub fn tile(entry: TilemapEntry) -> Result<Tile, HudFault> {
    look(entry)
        .zip(ColourSet::new(entry.palette.0))
        .map(|(look, colours)| Tile::new(look, colours))
        .ok_or(HudFault::Cell(entry))
}

/// The tilemap cell for the model's tile, above the background.
const fn entry(tile: Tile) -> TilemapEntry {
    let flip = tile.look.flip;
    TilemapEntry {
        tile: TileNumber(tile.look.picture.index()),
        palette: BgPalette(tile.colours.number()),
        priority: true,
        flip_x: matches!(flip, Flip::Across | Flip::Both),
        flip_y: matches!(flip, Flip::Down | Flip::Both),
    }
}

/// The model's item for the engine's.
const fn item(item: HudItem) -> SelectableItem {
    match item {
        HudItem::Missiles => SelectableItem::Ammo(Ammo::Missiles),
        HudItem::SuperMissiles => SelectableItem::Ammo(Ammo::SuperMissiles),
        HudItem::PowerBombs => SelectableItem::Ammo(Ammo::PowerBombs),
        HudItem::GrappleBeam => SelectableItem::GrappleBeam,
        HudItem::XRayScope => SelectableItem::XRayScope,
    }
}

/// The engine's item for the model's.
const fn hud_item(item: SelectableItem) -> HudItem {
    match item {
        SelectableItem::Ammo(Ammo::Missiles) => HudItem::Missiles,
        SelectableItem::Ammo(Ammo::SuperMissiles) => HudItem::SuperMissiles,
        SelectableItem::Ammo(Ammo::PowerBombs) => HudItem::PowerBombs,
        SelectableItem::GrappleBeam => HudItem::GrappleBeam,
        SelectableItem::XRayScope => HudItem::XRayScope,
    }
}

/// The model's selection of the engine's selected and auto-cancel items.
///
/// # Errors
///
/// [`HudFault::OneShotNotSelected`] for an auto-cancel item other than the selected one.
pub fn selection(
    selected: Option<HudItem>,
    auto_cancel: Option<HudItem>,
) -> Result<Selection, HudFault> {
    match (selected, auto_cancel) {
        (None, None) => Ok(Selection::None),
        (Some(selected), None) => Ok(Selection::Item(item(selected))),
        (Some(selected), Some(auto_cancel)) if selected == auto_cancel => {
            Ok(Selection::OneShot(item(selected)))
        }
        (None | Some(_), Some(_)) => Err(HudFault::OneShotNotSelected),
    }
}

/// The model's HUD of the engine's.
fn model(hud: &Hud) -> Result<HudState, HudFault> {
    // Every cell is set from the tilemap below.
    let mut grid = HudGrid::filled(tile(TilemapEntry::default())?);
    for (row, cells) in hud
        .tilemap
        .cells
        .as_chunks::<HUD_COLUMNS>()
        .0
        .iter()
        .enumerate()
    {
        for (column, &cell) in cells.iter().enumerate() {
            grid.set(place(column, row), tile(cell)?);
        }
    }
    Ok(HudState {
        grid,
        shown: Shown {
            energy: hud.drawn.energy,
            ammo: enum_map! {
                Ammo::Missiles => hud.drawn.missiles,
                Ammo::SuperMissiles => hud.drawn.super_missiles,
                Ammo::PowerBombs => hud.drawn.power_bombs,
            },
            highlighted: hud.drawn.selection.map(item),
        },
        selection: selection(hud.selected, hud.auto_cancel)?,
        explored: explored(&hud.mini_map.explored),
        mini_map: if hud.mini_map.disabled {
            MiniMapPower::Off
        } else {
            MiniMapPower::On
        },
    })
}

/// Writes the tiles `keep` names that the turn changed back to the engine's tilemap, leaving
/// every other cell's word as it was.
fn write_tiles(
    tilemap: &mut HudTilemap,
    before: &HudState,
    after: &HudState,
    keep: fn(Place) -> bool,
) {
    let rows = tilemap.cells.as_chunks_mut::<HUD_COLUMNS>().0;
    for (row, cells) in rows.iter_mut().enumerate() {
        for (column, cell) in cells.iter_mut().enumerate() {
            let at = place(column, row);
            let drawn = after.grid.get(at);
            if let Some(drawn) = drawn.filter(|_| keep(at) && drawn != before.grid.get(at)) {
                *cell = entry(drawn);
            }
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

/// What the model reads this turn from the engine's State over her `map`; the mini-map runs
/// when `runs`.
///
/// She can switch while the normal current state handler runs her (the elevator's locked
/// handler doesn't) and she doesn't face the screen. The engine fires no missiles, super
/// missiles or power bombs, so no shot cancels the selection; its one item, the morph ball,
/// draws no icon. Nothing in the engine turns the mini-map on or off: it enters no room by a
/// door or a load, meets no boss, and Mother Brain doesn't fall.
fn inputs<'map>(
    state: &State,
    map: &'map HerMap,
    runs: bool,
) -> Result<FrameInputs<'map>, HudFault> {
    let samus = &state.samus;
    let equipment = &samus.equipment;
    let ammunition = Ammunition::new(enum_map! {
        Ammo::Missiles => stock(equipment.missiles),
        Ammo::SuperMissiles => stock(equipment.super_missiles),
        Ammo::PowerBombs => stock(equipment.power_bombs),
    })
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
    let at = Position::from(samus.motion.position);
    Ok(FrameInputs {
        frame: FrameCount(state.clock.counters.requested_byte),
        status,
        controls,
        collected: None,
        mother_brain_falls: false,
        mini_map: MiniMapInputs {
            runs,
            turned_on: false,
            position: Pair {
                x: at.x.pixels,
                y: at.y.pixels,
            },
            room: Room {
                // Rooms start a row down: the map's row 0 is padding.
                cell: Pair {
                    x: i32::from(mini_map.room.x),
                    y: i32::from(mini_map.room.y).wrapping_add(1),
                },
                blocks: Pair {
                    x: width,
                    y: blocks.checked_div(width).unwrap_or(0),
                },
            },
            map,
            map_collected: mini_map.map_collected,
        },
    })
}

/// The model's HUD before and after its frame over the engine's State, the mini-map running when
/// `runs`; `None` with no HUD content.
fn frame(
    state: &State,
    content: &Content,
    runs: bool,
) -> Result<Option<(HudState, HudState)>, HudFault> {
    let Some((map, pictures)) = content.hud.of(state.hud.mini_map.area) else {
        return Ok(None);
    };
    let before = model(&state.hud)?;
    let inputs = inputs(state, map, runs)?;
    let after = before
        .clone()
        .frame(&Rules::reference(), pictures, &inputs)
        .state;
    Ok(Some((before, after)))
}

/// The mini-map's turn, after her moves (`$90:E734`): it marks her cell explored and draws the
/// window around her.
///
/// # Errors
///
/// A [`HudFault`] when the State holds what the model isn't defined on.
pub fn mini_map_turn(state: &mut State, content: &Content) -> Result<(), HudFault> {
    let Some((before, after)) = frame(state, content, true)? else {
        return Ok(());
    };
    let hud = &mut state.hud;
    write_tiles(&mut hud.tilemap, &before, &after, in_window);
    if after.explored != before.explored {
        hud.mini_map.explored = engine_cells(&after.explored);
    }
    hud.mini_map.disabled = after.mini_map == MiniMapPower::Off;
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
    let Some((before, after)) = frame(state, content, false)? else {
        return Ok(());
    };
    let hud = &mut state.hud;
    write_tiles(&mut hud.tilemap, &before, &after, |_| true);
    let Shown {
        energy,
        ammo,
        highlighted,
    } = after.shown;
    hud.drawn = Drawn {
        energy,
        missiles: ammo[Ammo::Missiles],
        super_missiles: ammo[Ammo::SuperMissiles],
        power_bombs: ammo[Ammo::PowerBombs],
        selection: highlighted.map(hud_item),
    };
    hud.selected = after.selection.item().map(hud_item);
    hud.auto_cancel = after.selection.one_shot().map(hud_item);
    let write = TileWrite::Overlay {
        row: HUD_ROW,
        cells: hud.tilemap.cells.to_vec(),
    };
    TileQueue::new(&mut state.video.batch).write(&state.room.camera, &write);
    Ok(())
}

#[cfg(test)]
mod tests;
