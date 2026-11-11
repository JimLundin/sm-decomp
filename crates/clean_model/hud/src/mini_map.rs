//! The mini-map (§4): the 5 × 3 cells of her map around hers, the ones she has explored, her own
//! cell blinking, the bosses that blank it, and the quirks at the map's first and last columns.

use clean_types::{Look, Pair, Picture, Screen};

use crate::grid::{Place, at, plus};
use crate::inputs::{FrameCount, MiniMapInputs};
use crate::layout::{HER_CELL, WINDOW_REACH};
use crate::pictures::HudPictures;
use crate::rules::{Boss, Rules};
use crate::state::{HudState, MiniMapPower};
use crate::tile::{BLINK, CHARTED, EXPLORED, Tile};

/// Spec: Exploring. A screen is 256 px.
const SCREEN_PX: u16 = 256;
/// Spec: MiniMapStill. A block of her room is 16 px.
const BLOCK_PX: u16 = 16;
/// Her map's 64 columns (Notation), round which a room's screens wrap.
const MAP_COLUMNS: i32 = 64;
/// Spec: FirstColumnsShift. The last of the map's first columns, 0 and 1: this one included.
const LAST_OF_FIRST_COLUMNS: i32 = 1;
/// Spec: LastColumnsShift. The first of the map's last columns, 62 and 63: this one included.
const FIRST_OF_LAST_COLUMNS: i32 = 62;
/// Spec: DiagonalPicture. A picture's number is read mod 512.
const PICTURE_NUMBER_MODULUS: u16 = 512;

/// The cell seen itself.
const HERE: Place = at(0, 0);
/// Spec: FirstColumnsShift. 32 columns along and a row higher.
const ALONG_AND_UP: Place = at(32, -1);
/// Spec: LastColumnsShift. 32 columns back and a row lower.
const BACK_AND_DOWN: Place = at(-32, 1);
/// Spec: DiagonalPicture. The cell over hers.
const ABOVE: Place = at(0, -1);

/// Where a window cell reads her map, each an offset from the map's cell it sees.
#[derive(Clone, Copy)]
struct Reading {
    /// Where it reads whether the cell is explored or charted.
    state: Place,
    /// Where it reads the cell's picture.
    picture: Place,
}

/// Spec: MiniMapWindow. The window cell `k` across and `j` down from hers shows the map's cell
/// `(cx + k, cy + j)`, the map's columns running on from 31 to 32.
const DIRECT: Reading = Reading {
    state: HERE,
    picture: HERE,
};

/// Spec: FirstColumnsShift. Each window cell shows the cell 32 columns along and a row higher:
/// its picture and whether it is explored or charted.
const FIRST_COLUMNS_SHIFT: Reading = Reading {
    state: ALONG_AND_UP,
    picture: ALONG_AND_UP,
};

/// Spec: LastColumnsShift. Each window cell takes its picture from its own cell, past column 63
/// a beyond picture, but whether it is explored or charted from the cell 32 columns back and a
/// row lower.
const LAST_COLUMNS_SHIFT: Reading = Reading {
    state: BACK_AND_DOWN,
    picture: HERE,
};

/// The reading for her cell's column: `FIRST_COLUMNS_SHIFT` in columns 0 and 1,
/// `LAST_COLUMNS_SHIFT` in 62 and 63, `DIRECT` between.
const fn reading(column: i32) -> Reading {
    if column <= LAST_OF_FIRST_COLUMNS {
        FIRST_COLUMNS_SHIFT
    } else if column >= FIRST_OF_LAST_COLUMNS {
        LAST_COLUMNS_SHIFT
    } else {
        DIRECT
    }
}

/// What a window cell shows of the map's cell it reads (spec: MiniMapWindow).
#[derive(Clone, Copy)]
enum WindowCell {
    /// It is explored: its picture with its flips, in the explored colours.
    Explored(Look),
    /// Not explored, but charted by a map she has collected: its picture with its flips, in the
    /// charted colours.
    Charted(Look),
    /// Neither, or the cell it reads lies outside the map: the map's blank, in the charted
    /// colours.
    Blank,
}

impl WindowCell {
    /// The HUD's cell that shows it.
    const fn tile(self, pictures: &HudPictures) -> Tile {
        match self {
            Self::Explored(look) => Tile::new(look, EXPLORED),
            Self::Charted(look) => Tile::new(look, CHARTED),
            Self::Blank => Tile::new(pictures.map_blank, CHARTED),
        }
    }
}

/// A room's screen `(sx, sy)` across and down from its top-left cell `(rx, ry)` is the map's cell
/// `((rx + sx) mod 64, ry + sy)`: the columns wrap round the map, the rows don't (spec:
/// Exploring, BossBlanksTheMap).
fn screen_cell(room: Place, screen: Screen) -> Place {
    at(
        room.x
            .saturating_add(i32::from(screen.x))
            .rem_euclid(MAP_COLUMNS),
        room.y.saturating_add(i32::from(screen.y)),
    )
}

/// Spec: Exploring. The screen of her room she is in: `(x div 256, y div 256)` from where she is.
const fn her_screen(position: Pair<u16>) -> Screen {
    Screen {
        x: position.x / SCREEN_PX,
        y: position.y / SCREEN_PX,
    }
}

/// Spec: DiagonalPicture. Whether `picture` counts as the diagonal: its number mod 512 is the
/// diagonal's.
const fn is_diagonal(picture: Picture, diagonal: Picture) -> bool {
    picture.index() % PICTURE_NUMBER_MODULUS == diagonal.index() % PICTURE_NUMBER_MODULUS
}

/// The mini-map's window: each of its cells on the HUD with its offset from hers, the top row
/// first, each row left to right.
fn window() -> impl Iterator<Item = (Place, Place)> {
    let reach = WINDOW_REACH;
    (reach.y.saturating_neg()..=reach.y)
        .flat_map(move |y| (reach.x.saturating_neg()..=reach.x).map(move |x| at(x, y)))
        .map(|offset| (plus(HER_CELL, offset), offset))
}

impl HudState {
    /// Spec: MiniMapStill. Whether the mini-map updates this frame: it runs, it isn't off, and
    /// she lies within her room's blocks, her position read as 0 to 65,535 so that one left of
    /// or above the room reads as past it.
    pub(crate) fn mini_map_updates(&self, inputs: &MiniMapInputs<'_>) -> bool {
        let blocks = inputs.room.blocks;
        inputs.runs
            && self.mini_map == MiniMapPower::On
            && inputs.position.x / BLOCK_PX < blocks.x
            && inputs.position.y / BLOCK_PX < blocks.y
    }

    /// The mini-map's update, in the order §5 gives: her cell is explored, the window is drawn
    /// from the explored cells, the cell over a diagonal one is explored after, and her cell
    /// blinks last.
    pub(crate) fn explore_and_draw_mini_map(
        &mut self,
        rules: &Rules,
        pictures: &HudPictures,
        inputs: &MiniMapInputs<'_>,
        frame: FrameCount,
    ) {
        let her = screen_cell(inputs.room.cell, her_screen(inputs.position));
        // Spec: Exploring.
        self.explored.insert(her);
        let reading = reading(her.x);
        for (place, offset) in window() {
            let cell = self.window_cell(inputs, pictures, reading, plus(her, offset));
            self.grid.set(place, cell.tile(pictures));
        }
        let her_cell = self.window_cell(inputs, pictures, reading, her);
        // Spec: DiagonalPicture. Her window cell shown explored, with a picture that counts as
        // the diagonal: the cell over hers joins the explored cells, shown from the next frame.
        if let WindowCell::Explored(look) = her_cell
            && is_diagonal(look.picture, pictures.diagonal)
        {
            self.explored.insert(plus(her, ABOVE));
        }
        // Spec: HerBlink. The blink's first half shows her window cell in the blink colours; its
        // second half leaves it as drawn.
        let in_blink_colours = rules.her_blink.first_half(frame);
        if in_blink_colours {
            self.grid
                .set(HER_CELL, her_cell.tile(pictures).in_colours(BLINK));
        }
    }

    /// Spec: BossBlanksTheMap. A boss blanks the mini-map: every window cell shows the map's
    /// blank in the charted colours, each screen the boss names, from her room's top-left cell
    /// `room`, joins the explored cells, and the mini-map is off.
    pub(crate) fn blank_mini_map(
        &mut self,
        rules: &Rules,
        pictures: &HudPictures,
        boss: Boss,
        room: Place,
    ) {
        for (place, _) in window() {
            self.grid.set(place, WindowCell::Blank.tile(pictures));
        }
        for &screen in &rules.boss_screens[boss] {
            self.explored.insert(screen_cell(room, screen));
        }
        self.mini_map = MiniMapPower::Off;
    }

    /// Spec: MiniMapWindow. What the window cell that sees the map's cell `seen` shows, its state
    /// and picture read as `reading` says: explored, or charted by a map she has collected, or
    /// blank. A picture read outside the map and its beyond pictures is the map's blank (§1.5).
    fn window_cell(
        &self,
        inputs: &MiniMapInputs<'_>,
        pictures: &HudPictures,
        reading: Reading,
        seen: Place,
    ) -> WindowCell {
        let state = plus(seen, reading.state);
        let look = inputs
            .map
            .pictures
            .get(plus(seen, reading.picture))
            .unwrap_or(pictures.map_blank);
        if self.explored.contains(state) {
            WindowCell::Explored(look)
        } else if inputs.map_collected && inputs.map.charted.contains(state) {
            WindowCell::Charted(look)
        } else {
            WindowCell::Blank
        }
    }
}
