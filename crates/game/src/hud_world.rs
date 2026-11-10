//! The HUD as the engine meets it: the HUD model (`clean_model_hud`).
//!
//! The engine keeps the HUD as the game keeps it (`docs/behaviours/hud.decode.md`, "State"): the
//! HUD tilemap's three rows, the values it last drew, the selected and the auto-cancel item, the
//! explored cells and whether the mini-map is off. The model's `HudState` is built from those
//! each turn and written back after it; the area maps and the HUD's pictures are the model's,
//! converted once from the importer's when the content loads.
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
    MiniMapInputs, MiniMapPower, Mute, Picture, Point, Reserve, ReserveMode, Room, Rules,
    SelectButton, SelectableItem, Selection, Shown, Status, Stock, Tile,
};
use clean_types::{Look, Pair};
use data::dma::HUD_TILEMAP_BUFFER;
use data::map::{MAP_COLUMNS, MAP_ROWS, MAP_TILEMAP_WORDS, MapCells};
use data::pose::transition::Control;
use data::room::Area;
use enum_map::enum_map;

use crate::pose::{self, Content, FACING_FORWARD};
use crate::state::State;
use crate::state::hud::{Drawn, Hud, HudItem, HudTilemap};
use crate::state::samus::{Gauge, Item, MovementType, Point as Position, ReserveMode as Refill};
use crate::state::video::{
    BgPalette, TileNumber, TilemapEntry, VramSource, VramStep, VramTransfer, VramWord,
};
use crate::state::{Button, Controller};

/// Where the queued tilemap lands: the HUD's second BG3 row (`$80:9CC1`).
pub const HUD_TILEMAP_VRAM: VramWord = VramWord(0x5820);
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
/// A picture no map draws, which the model's map holds where the importer gives none.
const NO_PICTURE: Picture = Picture::new(0, Flip::None);

/// Why the HUD can't run a turn over the engine's State.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudFault {
    /// A count or capacity past what the HUD's cells hold: 999 missiles, 99 super missiles or
    /// power bombs (spec §1.5; the original reads past its digit table, docs/LEGACY.md).
    CountPastCells,
    /// An auto-cancel item that isn't the selected one, which the original never holds.
    OneShotNotSelected,
    /// A HUD tilemap cell's palette is past the eight colour sets.
    Colours(BgPalette),
}

/// The model's picture of a look.
const fn picture(look: Look) -> Picture {
    Picture::new(look.picture.index(), look.flip)
}

/// The model's pictures of a table of looks.
fn pictures<const ACROSS: usize, const DOWN: usize>(
    looks: [[Look; ACROSS]; DOWN],
) -> [[Picture; ACROSS]; DOWN] {
    looks.map(|row| row.map(picture))
}

/// The model's HUD pictures of the importer's.
fn hud_pictures(imported: &data::hud::HudPictures) -> HudPictures {
    let icons = &imported.icons;
    HudPictures {
        blank: picture(imported.blank),
        digits: imported.digits.map(picture),
        full_tank: picture(imported.full_tank),
        empty_tank: picture(imported.empty_tank),
        reserve_sign: pictures(imported.reserve_sign),
        icons: Icons {
            missiles: pictures(icons.missiles),
            super_missiles: pictures(icons.super_missiles),
            power_bombs: pictures(icons.power_bombs),
            grapple_beam: pictures(icons.grapple_beam),
            x_ray_scope: pictures(icons.x_ray_scope),
        },
        map_blank: picture(imported.map_blank),
        diagonal: picture(imported.diagonal),
    }
}

/// The model's flips of a tilemap entry's.
const fn flip(flip_x: bool, flip_y: bool) -> Flip {
    match (flip_x, flip_y) {
        (false, false) => Flip::None,
        (true, false) => Flip::Across,
        (false, true) => Flip::Down,
        (true, true) => Flip::Both,
    }
}

/// A tilemap word's picture and flips; its colours are the HUD's to choose.
const fn word_picture(word: u16) -> Picture {
    let entry = TilemapEntry::from_word(word);
    Picture::new(entry.tile.0, flip(entry.flip_x, entry.flip_y))
}

/// The place `column` across and `row` down a grid.
fn place(column: usize, row: usize) -> Point {
    let coordinate = |value: usize| i32::try_from(value).unwrap_or(i32::MAX);
    Pair {
        x: coordinate(column),
        y: coordinate(row),
    }
}

/// The model's pictures of `map`: column `c` of row `y` is word `c div 32 × 400h + y × 20h +
/// c mod 32`, and the beyond words follow the map's two pages as a third, so the map's columns
/// 64 and 65 are that page's first two.
fn map_pictures(map: &data::map::AreaMap) -> MapPictures {
    let mut pictures = MapPictures::filled(NO_PICTURE);
    for row in 0..usize::from(MAP_ROWS) {
        for column in 0..PICTURE_COLUMNS {
            let word = (column / PAGE_CELLS)
                .wrapping_mul(PAGE_WORDS)
                .wrapping_add(row.wrapping_mul(ROW_WORDS))
                .wrapping_add(column % PAGE_CELLS);
            let read = word
                .checked_sub(MAP_TILEMAP_WORDS)
                .map_or_else(|| map.tiles.get(word), |beyond| map.beyond.get(beyond));
            if let Some(&word) = read {
                pictures.set(place(column, row), word_picture(word));
            }
        }
    }
    pictures
}

/// An area's map as the model reads it, before and after she collects it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Maps {
    /// The map she hasn't collected.
    not_collected: HerMap,
    /// The same map collected.
    collected: HerMap,
}

impl Maps {
    /// Both of a map's, with its `pictures` and `charted` cells.
    const fn of(pictures: &MapPictures, charted: &MapCellSet) -> Self {
        let (pictures, charted) = (*pictures, *charted);
        Self {
            not_collected: HerMap {
                pictures,
                charted,
                collected: false,
            },
            collected: HerMap {
                pictures,
                charted,
                collected: true,
            },
        }
    }
}

/// What the HUD draws with: every area's map, and the HUD's pictures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HudContent {
    maps: Vec<(Area, Maps)>,
    /// The maps of an area with none: the debug area's pointers name no map.
    blank: Maps,
    pictures: HudPictures,
}

impl HudContent {
    /// The model's content of the importer's maps and pictures.
    #[must_use]
    pub fn import(maps: &data::map::AreaMaps, pictures: &data::hud::HudPictures) -> Self {
        Self {
            maps: maps
                .0
                .iter()
                .map(|map| {
                    let maps = Maps::of(&map_pictures(map), &explored(&map.station));
                    (map.area, maps)
                })
                .collect(),
            pictures: hud_pictures(pictures),
            ..Self::default()
        }
    }

    /// `area`'s map, collected or not; a blank map that charts nothing for an area with none.
    #[must_use]
    pub fn map(&self, area: Area, collected: bool) -> &HerMap {
        let maps = self
            .maps
            .iter()
            .find(|(each, _)| *each == area)
            .map_or(&self.blank, |(_, maps)| maps);
        if collected {
            &maps.collected
        } else {
            &maps.not_collected
        }
    }

    /// The HUD's pictures.
    #[must_use]
    pub const fn pictures(&self) -> &HudPictures {
        &self.pictures
    }
}

impl Default for HudContent {
    /// The content with nothing imported: no area's map, and every picture the first, unflipped.
    fn default() -> Self {
        Self {
            maps: Vec::new(),
            blank: Maps::of(&MapPictures::filled(NO_PICTURE), &MapCellSet::default()),
            pictures: HudPictures {
                blank: NO_PICTURE,
                digits: [NO_PICTURE; 10],
                full_tank: NO_PICTURE,
                empty_tank: NO_PICTURE,
                reserve_sign: [[NO_PICTURE; 2]; 3],
                icons: Icons {
                    missiles: [[NO_PICTURE; 3]; 2],
                    super_missiles: [[NO_PICTURE; 2]; 2],
                    power_bombs: [[NO_PICTURE; 2]; 2],
                    grapple_beam: [[NO_PICTURE; 2]; 2],
                    x_ray_scope: [[NO_PICTURE; 2]; 2],
                },
                map_blank: NO_PICTURE,
                diagonal: NO_PICTURE,
            },
        }
    }
}

/// The place of column `column` of row `row` on her map.
fn map_cell(column: u8, row: usize) -> Point {
    Pair {
        x: i32::from(column),
        y: i32::try_from(row).unwrap_or(i32::MAX),
    }
}

/// The model's set of the engine's cells: row `y`'s word holds column `x` at bit `x`.
#[must_use]
pub fn explored(cells: &MapCells) -> MapCellSet {
    let mut set = MapCellSet::default();
    for (row, word) in cells.0.iter().enumerate() {
        for column in (0..MAP_COLUMNS).filter(|&column| (word >> column) & 1 != 0) {
            set.insert(map_cell(column, row));
        }
    }
    set
}

/// The engine's set of the model's cells.
fn engine_cells(set: &MapCellSet) -> MapCells {
    let mut cells = MapCells::default();
    for (row, word) in cells.0.iter_mut().enumerate() {
        for column in (0..MAP_COLUMNS).filter(|&column| set.contains(map_cell(column, row))) {
            *word |= 1 << column;
        }
    }
    cells
}

/// Whether the HUD cell at `point` is the mini-map's window's.
fn in_window(point: Point) -> bool {
    WINDOW_COLUMNS.contains(&point.x)
}

/// The model's tile for a tilemap cell. The HUD draws every word above the background, so its
/// priority is not the model's.
///
/// # Errors
///
/// [`HudFault::Colours`] for a palette past the eight colour sets.
pub fn tile(entry: TilemapEntry) -> Result<Tile, HudFault> {
    let colours = ColourSet::new(entry.palette.0).ok_or(HudFault::Colours(entry.palette))?;
    Ok(Picture::new(entry.tile.0, flip(entry.flip_x, entry.flip_y)).in_colours(colours))
}

/// The tilemap cell for the model's tile, above the background.
const fn entry(tile: Tile) -> TilemapEntry {
    let flip = tile.picture.flip;
    TilemapEntry {
        tile: TileNumber(tile.picture.number),
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
    keep: fn(Point) -> bool,
) {
    let rows = tilemap.cells.as_chunks_mut::<HUD_COLUMNS>().0;
    for (row, cells) in rows.iter_mut().enumerate() {
        for (column, cell) in cells.iter_mut().enumerate() {
            let point = place(column, row);
            let drawn = after.grid.get(point);
            if let Some(drawn) = drawn.filter(|_| keep(point) && drawn != before.grid.get(point)) {
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

/// What the model reads this turn from the engine's State; the mini-map runs when `runs`.
///
/// She can switch while the normal current state handler runs her (the elevator's locked
/// handler doesn't) and she doesn't face the screen. The engine fires no missiles, super
/// missiles or power bombs, so no shot cancels the selection; its one item, the morph ball,
/// draws no icon. Nothing in the engine turns the mini-map on or off: it enters no room by a
/// door or a load, meets no boss, and Mother Brain doesn't fall.
fn inputs<'map>(
    state: &State,
    content: &'map HudContent,
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
            map: content.map(mini_map.area, mini_map.map_collected),
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
    let inputs = inputs(state, &content.hud, true)?;
    let after = before
        .clone()
        .frame(&Rules::reference(), content.hud.pictures(), &inputs)
        .state;
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
    let before = model(&state.hud)?;
    let inputs = inputs(state, &content.hud, false)?;
    let after = before
        .clone()
        .frame(&Rules::reference(), content.hud.pictures(), &inputs)
        .state;
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
