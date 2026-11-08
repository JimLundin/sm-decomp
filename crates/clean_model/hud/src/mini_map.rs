//! The mini-map (§4): the 5 × 3 cells of her map around hers, the ones she has explored, her own
//! cell blinking, and the quirks at the map's first and last columns.

use clean_types::{CellSet, MapCell};

use crate::grid::{ColourSet, HudGrid, Picture, Tile};
use crate::inputs::{FrameCount, MiniMapInputs};
use crate::map::{AreaMap, BeyondCell, MapPicture};
use crate::pictures::{BLINK, CHARTED, EXPLORED, HER_WINDOW_TILE, WINDOW_REACH};
use crate::rules::{Boss, Rules};

/// Spec: Exploring. A screen is 256 px: her cell is her room's cell plus her position over 256.
const SCREEN_PX: u16 = 256;
/// Spec: MiniMapStill. A block of her room is 16 px.
const BLOCK_PX: u16 = 16;
/// Spec: FirstColumnsShift, LastColumnsShift. The window's other reading lies 32 columns away.
const HALF_MAP_COLUMNS: i32 = 32;
/// Spec: FirstColumnsShift. Her cell in the map's columns 0 and 1, up to this one.
const FIRST_COLUMNS_END: i32 = 1;
/// Spec: LastColumnsShift. Her cell in the map's columns 62 and 63, from this one.
const LAST_COLUMNS_START: i32 = 62;
/// Spec: DiagonalPicture. A picture's number is read mod 200h, its last nine bits.
const PICTURE_NUMBER_READ: u16 = 0x200;
/// Her map is 64 columns across (Notation); columns wrap mod 64 where a rule says so.
const MAP_COLUMNS: i32 = 64;

/// A place on her map's grid of cells, which may lie outside its 64 columns and 32 rows.
#[derive(Clone, Copy, PartialEq, Eq)]
struct MapPoint {
    column: i32,
    row: i32,
}

impl MapPoint {
    /// The place `columns` right and `rows` down of this one.
    const fn offset(self, columns: i32, rows: i32) -> Self {
        Self {
            column: self.column.saturating_add(columns),
            row: self.row.saturating_add(rows),
        }
    }

    /// The map's cell here, or `None` outside the map (§1.5).
    fn cell(self) -> Option<MapCell> {
        MapCell::new(
            u8::try_from(self.column).ok()?,
            u8::try_from(self.row).ok()?,
        )
    }

    /// The map's picture here; outside the map, its blank (§1.5).
    fn picture(self, map: &AreaMap) -> MapPicture {
        self.cell()
            .map_or(MapPicture::BLANK, |cell| map.picture(cell))
    }
}

/// Spec: Exploring. Her cell: column `(rx + x div 256) mod 64` and row `ry + y div 256` of her
/// map, from her room's top-left cell `(rx, ry)` and where she is `(x, y)`.
fn her_cell(mini_map: &MiniMapInputs<'_>) -> MapPoint {
    let room = mini_map.room.cell;
    let across = i32::from(mini_map.position.x / SCREEN_PX);
    let down = i32::from(mini_map.position.y / SCREEN_PX);
    MapPoint {
        column: i32::from(room.column())
            .saturating_add(across)
            .rem_euclid(MAP_COLUMNS),
        row: i32::from(room.row()).saturating_add(down),
    }
}

/// Spec: MiniMapStill. Whether the mini-map updates this frame: it runs, it isn't off, and she
/// lies within her room's blocks, her position read as 0 to 65,535 so that one left of or above
/// the room reads as past it.
pub const fn updates(mini_map: &MiniMapInputs<'_>) -> bool {
    let blocks = mini_map.room.blocks;
    mini_map.runs
        && !mini_map.off
        && mini_map.position.x / BLOCK_PX < blocks.x
        && mini_map.position.y / BLOCK_PX < blocks.y
}

/// Which of her map's cells each window cell reads, by her cell's column.
#[derive(Clone, Copy)]
enum Reading {
    /// Spec: MiniMapWindow. The window cell `k` columns and `j` rows from hers shows the map's
    /// cell `(cx + k, cy + j)`, the map's columns running on from 31 to 32.
    Direct,
    /// Spec: FirstColumnsShift. With her cell in columns 0 and 1, each window cell shows the
    /// cell 32 columns along and a row higher: its picture and whether it is explored or
    /// charted.
    FirstColumnsShift,
    /// Spec: LastColumnsShift. With her cell in columns 62 and 63, each window cell takes its
    /// picture from its own cell, a beyond picture past column 63, but whether it is explored or
    /// charted from the cell 32 columns back and a row lower.
    LastColumnsShift,
}

impl Reading {
    /// The reading for her cell's column.
    const fn for_column(column: i32) -> Self {
        if column <= FIRST_COLUMNS_END {
            Self::FirstColumnsShift
        } else if column >= LAST_COLUMNS_START {
            Self::LastColumnsShift
        } else {
            Self::Direct
        }
    }

    /// The cell whose explored and charted state the window cell over `seen` shows.
    const fn state_of(self, seen: MapPoint) -> MapPoint {
        match self {
            Self::Direct => seen,
            Self::FirstColumnsShift => seen.offset(HALF_MAP_COLUMNS, -1),
            Self::LastColumnsShift => seen.offset(HALF_MAP_COLUMNS.saturating_neg(), 1),
        }
    }

    /// The picture the window cell over `seen` shows.
    fn picture_of(self, map: &AreaMap, seen: MapPoint) -> MapPicture {
        match self {
            Self::Direct => seen.picture(map),
            Self::FirstColumnsShift => seen.offset(HALF_MAP_COLUMNS, -1).picture(map),
            Self::LastColumnsShift => beyond_or_own(map, seen),
        }
    }
}

/// Spec: LastColumnsShift. Past column 63 the picture is the map's beyond picture
/// `(column − 64, row)`; otherwise the cell's own.
fn beyond_or_own(map: &AreaMap, seen: MapPoint) -> MapPicture {
    let past = seen.column.saturating_sub(MAP_COLUMNS);
    if past < 0 {
        return seen.picture(map);
    }
    let beyond = u8::try_from(past)
        .ok()
        .zip(u8::try_from(seen.row).ok())
        .and_then(|(column, row)| BeyondCell::new(column, row));
    beyond.map_or(MapPicture::BLANK, |cell| map.beyond(cell))
}

/// How a window cell shows.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shows {
    /// Explored: its picture in the explored colours.
    Explored,
    /// Charted by the collected map: its picture in the charted colours.
    Charted,
    /// Neither: the map's blank in the charted colours.
    Blank,
}

/// Spec: MiniMapWindow. A window cell: explored, its picture with its flips in the explored
/// colours; not explored but charted by a map she has collected, in the charted colours; neither,
/// or its state read outside the map, the map's blank in the charted colours.
fn window_tile(
    map: &AreaMap,
    explored: &CellSet,
    reading: Reading,
    seen: MapPoint,
) -> (Tile, Shows) {
    let shows = match reading.state_of(seen).cell() {
        Some(cell) if explored.contains(cell) => Shows::Explored,
        Some(cell) if map.collected() && map.charts(cell) => Shows::Charted,
        Some(_) | None => Shows::Blank,
    };
    let tile = match shows {
        Shows::Explored => in_colours(reading.picture_of(map, seen), EXPLORED),
        Shows::Charted => in_colours(reading.picture_of(map, seen), CHARTED),
        Shows::Blank => in_colours(MapPicture::BLANK, CHARTED),
    };
    (tile, shows)
}

/// A map picture as a HUD tile in `colours`.
const fn in_colours(picture: MapPicture, colours: ColourSet) -> Tile {
    Tile::new(picture.picture, colours, picture.flip)
}

/// Spec: DiagonalPicture. Whether a picture is the diagonal one: its number mod 200h is the
/// diagonal picture's.
const fn is_diagonal(rules: &Rules, picture: Picture) -> bool {
    picture.number() % PICTURE_NUMBER_READ == rules.diagonal_picture.number() % PICTURE_NUMBER_READ
}

/// The mini-map's update, in the order §5 gives: her cell is explored, the window is drawn from
/// the explored cells, the cell over a diagonal one is explored after, and her cell blinks last.
pub fn update(
    rules: &Rules,
    mini_map: &MiniMapInputs<'_>,
    frame: FrameCount,
    grid: &mut HudGrid,
    explored: &mut CellSet,
) {
    let her = her_cell(mini_map);
    // Spec: Exploring. Her cell, when it lies on the map, joins the explored cells.
    if let Some(cell) = her.cell() {
        explored.insert(cell);
    }
    let reading = Reading::for_column(her.column);
    for (columns, rows) in WINDOW_REACH.offsets() {
        let (tile, _) = window_tile(mini_map.map, explored, reading, her.offset(columns, rows));
        if let Some(pos) = HER_WINDOW_TILE.offset(rows, columns) {
            grid.set(pos, tile);
        }
    }
    let (her_tile, her_shows) = window_tile(mini_map.map, explored, reading, her);
    // Spec: DiagonalPicture. Her window cell shown explored with the diagonal picture: the cell
    // over hers joins the explored cells, shown from the next frame.
    if her_shows == Shows::Explored
        && is_diagonal(rules, her_tile.picture)
        && let Some(above) = her.offset(0, -1).cell()
    {
        explored.insert(above);
    }
    // Spec: HerBlink. For the first 8 frames of each 16, her window cell takes the blink colours.
    if rules.her_blink.first_half(frame.0) {
        grid.set(HER_WINDOW_TILE, her_tile.in_colours(BLINK));
    }
}

/// Spec: BossBlanksTheMap. A boss turns the mini-map off: every window cell shows the map's blank
/// in the charted colours, and each screen `(sx, sy)` the boss names joins the explored cells as
/// the map's cell `((rx + sx) mod 64, ry + sy)` from her room's top-left cell `(rx, ry)`, when
/// that lies on the map.
pub fn boss_turns_it_off(
    rules: &Rules,
    boss: Boss,
    room: MapCell,
    grid: &mut HudGrid,
    explored: &mut CellSet,
) {
    let blank = in_colours(MapPicture::BLANK, CHARTED);
    for (columns, rows) in WINDOW_REACH.offsets() {
        if let Some(pos) = HER_WINDOW_TILE.offset(rows, columns) {
            grid.set(pos, blank);
        }
    }
    for screen in rules.screens_of(boss) {
        let point = MapPoint {
            column: i32::from(room.column())
                .saturating_add(i32::from(screen.across))
                .rem_euclid(MAP_COLUMNS),
            row: i32::from(room.row()).saturating_add(i32::from(screen.down)),
        };
        if let Some(cell) = point.cell() {
            explored.insert(cell);
        }
    }
}
