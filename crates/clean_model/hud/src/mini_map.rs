//! The mini-map (§4): the 5 × 3 cells of her map around hers, the ones she has explored, her own
//! cell blinking, and the quirks at the map's first and last columns.

use clean_types::Pair;

use crate::inputs::{FrameCount, MiniMapInputs};
use crate::map::{MapCell, MapPicture, PicturePos};
use crate::pictures::{BLINK, CHARTED, EXPLORED, HER_TILE, HudPos, Picture, Tile, WINDOW_REACH};
use crate::rules::{Boss, Rules, Screen};
use crate::state::HudState;

/// Spec: Exploring. A screen is 256 px.
const SCREEN_PX: u16 = 256;
/// Spec: MiniMapStill. A block of her room is 16 px.
const BLOCK_PX: u16 = 16;
/// Her map's 64 columns (Notation), round which a room's screens wrap.
const MAP_COLUMNS: i32 = 64;
/// Spec: FirstColumnsShift. Her cell in the map's columns 0 and 1, up to this one.
const FIRST_COLUMNS_END: i32 = 1;
/// Spec: LastColumnsShift. Her cell in the map's columns 62 and 63, from this one.
const LAST_COLUMNS_START: i32 = 62;
/// Spec: DiagonalPicture. A picture's number is read mod 200h, its last nine bits.
const PICTURE_NUMBER_READ: u16 = 0x200;

/// The cell seen itself.
const HERE: Pair<i32> = Pair { x: 0, y: 0 };
/// Spec: FirstColumnsShift. 32 columns along and a row higher.
const ALONG_AND_UP: Pair<i32> = Pair { x: 32, y: -1 };
/// Spec: LastColumnsShift. 32 columns back and a row lower.
const BACK_AND_DOWN: Pair<i32> = Pair { x: -32, y: 1 };
/// Spec: DiagonalPicture. The cell over hers.
const ABOVE: Pair<i32> = Pair { x: 0, y: -1 };

/// A place on her map, across and down, which may lie outside it (§1.5).
type Point = Pair<i32>;

/// The point `by` across and down from `point`.
const fn plus(point: Point, by: Pair<i32>) -> Point {
    Pair {
        x: point.x.saturating_add(by.x),
        y: point.y.saturating_add(by.y),
    }
}

/// A room's screen `(sx, sy)` across and down from its top-left cell `(rx, ry)` is the map's cell
/// `((rx + sx) mod 64, ry + sy)`: the columns wrap round the map, the rows don't (spec:
/// Exploring, BossBlanksTheMap).
fn screen_cell(room: MapCell, screen: Screen) -> Point {
    Pair {
        x: i32::from(room.x())
            .saturating_add(i32::from(screen.x))
            .rem_euclid(MAP_COLUMNS),
        y: i32::from(room.y()).saturating_add(i32::from(screen.y)),
    }
}

/// The mini-map's window: each of its tiles with its offset from her cell, the top row first,
/// each row left to right.
fn window() -> impl Iterator<Item = (HudPos, Pair<i32>)> {
    let reach = WINDOW_REACH;
    (reach.y.saturating_neg()..=reach.y)
        .flat_map(move |y| (reach.x.saturating_neg()..=reach.x).map(move |x| Pair { x, y }))
        .filter_map(|offset| HER_TILE.offset(offset).map(|pos| (pos, offset)))
}

/// Which of her map's cells a window cell reads, by her cell's column.
#[derive(Clone, Copy)]
enum Reading {
    /// Spec: MiniMapWindow. The window cell `k` across and `j` down from hers shows the map's
    /// cell `(cx + k, cy + j)`, the map's columns running on from 31 to 32.
    Direct,
    /// Spec: FirstColumnsShift. With her cell in columns 0 and 1, each window cell shows the
    /// cell 32 columns along and a row higher: its picture and whether it is explored or
    /// charted.
    FirstColumnsShift,
    /// Spec: LastColumnsShift. With her cell in columns 62 and 63, each window cell takes its
    /// picture from its own cell, past column 63 a beyond picture, but whether it is explored or
    /// charted from the cell 32 columns back and a row lower.
    LastColumnsShift,
}

/// Where a window cell reads from, each an offset from the cell it sees.
struct ReadFrom {
    /// Whether it is explored or charted.
    state: Pair<i32>,
    /// Its picture.
    picture: Pair<i32>,
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

    /// Its row: where its window cells read from.
    const fn read_from(self) -> ReadFrom {
        match self {
            Self::Direct => ReadFrom {
                state: HERE,
                picture: HERE,
            },
            Self::FirstColumnsShift => ReadFrom {
                state: ALONG_AND_UP,
                picture: ALONG_AND_UP,
            },
            Self::LastColumnsShift => ReadFrom {
                state: BACK_AND_DOWN,
                picture: HERE,
            },
        }
    }
}

/// Spec: DiagonalPicture. Whether a picture is the diagonal one: its number mod 200h is the
/// diagonal picture's.
const fn is_diagonal(rules: &Rules, picture: Picture) -> bool {
    picture.number() % PICTURE_NUMBER_READ == rules.diagonal_picture.number() % PICTURE_NUMBER_READ
}

impl MiniMapInputs<'_> {
    /// Spec: MiniMapStill. Whether the mini-map updates this frame: it runs, it isn't off, and
    /// she lies within her room's blocks, her position read as 0 to 65,535 so that one left of
    /// or above the room reads as past it.
    pub(crate) const fn updates(&self) -> bool {
        let blocks = self.room.blocks;
        self.runs
            && !self.off
            && self.position.x / BLOCK_PX < blocks.x
            && self.position.y / BLOCK_PX < blocks.y
    }

    /// Spec: Exploring. Her cell: the screen of her room she is in, `(x div 256, y div 256)`
    /// from where she is.
    const fn her_screen(&self) -> Screen {
        Pair {
            x: self.position.x / SCREEN_PX,
            y: self.position.y / SCREEN_PX,
        }
    }
}

impl HudState {
    /// The mini-map's update, in the order §5 gives: her cell is explored, the window is drawn
    /// from the explored cells, the cell over a diagonal one is explored after, and her cell
    /// blinks last.
    pub(crate) fn update_mini_map(
        &mut self,
        rules: &Rules,
        inputs: &MiniMapInputs<'_>,
        frame: FrameCount,
    ) {
        let her = screen_cell(inputs.room.cell, inputs.her_screen());
        // Spec: Exploring.
        self.explore(her);
        let from = Reading::for_column(her.x).read_from();
        for (pos, offset) in window() {
            let tile = self.window_tile(inputs, &from, plus(her, offset));
            self.grid.set(pos, tile);
        }
        let her_tile = self.grid.get(HER_TILE);
        // Spec: DiagonalPicture. Her window cell shown explored, in the explored colours, with
        // the diagonal picture: the cell over hers joins the explored cells, shown from the next
        // frame.
        if her_tile.colours == EXPLORED && is_diagonal(rules, her_tile.picture) {
            self.explore(plus(her, ABOVE));
        }
        // Spec: HerBlink. For the first 8 frames of each 16, her window cell takes the blink
        // colours.
        if rules.her_blink.first_half(frame.0) {
            self.grid.set(HER_TILE, her_tile.in_colours(BLINK));
        }
    }

    /// Spec: BossBlanksTheMap. A boss turns the mini-map off: every window cell shows the map's
    /// blank in the charted colours, and each screen the boss names, from her room's top-left
    /// cell `room`, joins the explored cells.
    pub(crate) fn blank_mini_map(&mut self, rules: &Rules, boss: Boss, room: MapCell) {
        for (pos, _) in window() {
            self.grid.set(pos, MapPicture::BLANK.in_colours(CHARTED));
        }
        for &screen in rules.screens_of(boss) {
            self.explore(screen_cell(room, screen));
        }
    }

    /// Spec: Exploring. The cell at `point` joins the explored cells; off the map, nothing
    /// changes (§1.5).
    fn explore(&mut self, point: Point) {
        if let Some(cell) = MapCell::new(point.x, point.y) {
            self.explored.set(cell, true);
        }
    }

    /// Spec: MiniMapWindow. The window cell that sees `seen`, its state and picture read as
    /// `from` says: explored, its picture with its flips in the explored colours; not explored
    /// but charted by a map she has collected, in the charted colours; neither, or its state read
    /// outside the map, the map's blank in the charted colours. A picture read outside the map
    /// and its beyond pictures is the map's blank (§1.5).
    fn window_tile(&self, inputs: &MiniMapInputs<'_>, from: &ReadFrom, seen: Point) -> Tile {
        let state = plus(seen, from.state);
        let at = plus(seen, from.picture);
        let picture = PicturePos::new(at.x, at.y)
            .map_or(MapPicture::BLANK, |pos| inputs.map.pictures.get(pos));
        match MapCell::new(state.x, state.y) {
            Some(cell) if self.explored.get(cell) => picture.in_colours(EXPLORED),
            Some(cell) if inputs.map_collected && inputs.map.charted.get(cell) => {
                picture.in_colours(CHARTED)
            }
            Some(_) | None => MapPicture::BLANK.in_colours(CHARTED),
        }
    }
}
