//! The gameplay pass's HUD phase: redraw what changed into the HUD tilemap, then queue the
//! tilemap's upload.
//!
//! Bank `$80` log, handle HUD tilemap `$80:9B44`, in its order: the AUTO reserve sign, the
//! energy tanks and digits, the missile, super missile and power bomb counts, the selected
//! item's highlight, the auto-cancel item's flash, then the queued transfer of the tilemap
//! to VRAM.

use data::dma::HUD_TILEMAP_BUFFER;

use crate::state::State;
use crate::state::hud::{Cell, Hud, HudItem, HudTilemap};
use crate::state::samus::{Equipment, Gauge, ReserveMode};
use crate::state::video::{
    BgPalette, TileNumber, TilemapEntry, VramSource, VramStep, VramTransfer, VramWord,
};

/// Where the queued tilemap lands: the HUD's second BG3 row (`$80:9CC1`).
pub const HUD_TILEMAP_VRAM: VramWord = VramWord(0x5820);

/// A HUD tile in `palette`, drawn above the background, unflipped.
const fn hud_tile(tile: u16, palette: u8) -> TilemapEntry {
    TilemapEntry {
        tile: TileNumber(tile),
        palette: BgPalette(palette),
        priority: true,
        flip_x: false,
        flip_y: false,
    }
}

/// The blank HUD tile; an item icon cell holding it keeps it when highlighted (`$80:9CF9`).
pub const BLANK: TilemapEntry = hud_tile(0x0F, 3);

/// The digit tiles, from 0 to 9 (`$80:9DBF` for energy, `$80:9DD3` for ammunition: the two
/// tables are identical).
pub const DIGITS: [TilemapEntry; 10] = [
    hud_tile(0x09, 3),
    hud_tile(0x00, 3),
    hud_tile(0x01, 3),
    hud_tile(0x02, 3),
    hud_tile(0x03, 3),
    hud_tile(0x04, 3),
    hud_tile(0x05, 3),
    hud_tile(0x06, 3),
    hud_tile(0x07, 3),
    hud_tile(0x08, 3),
];

/// A full energy tank (`$80:9BDC`).
pub const FULL_TANK: TilemapEntry = hud_tile(0x31, 2);
/// An empty energy tank (`$80:9BD3`).
pub const EMPTY_TANK: TilemapEntry = hud_tile(0x30, 5);

/// The energy tank cells in fill order: the bottom row left to right, then the top row
/// (`$80:9CCE`).
pub const TANK_CELLS: [Cell; 14] = [
    Cell { row: 1, column: 1 },
    Cell { row: 1, column: 2 },
    Cell { row: 1, column: 3 },
    Cell { row: 1, column: 4 },
    Cell { row: 1, column: 5 },
    Cell { row: 1, column: 6 },
    Cell { row: 1, column: 7 },
    Cell { row: 0, column: 1 },
    Cell { row: 0, column: 2 },
    Cell { row: 0, column: 3 },
    Cell { row: 0, column: 4 },
    Cell { row: 0, column: 5 },
    Cell { row: 0, column: 6 },
    Cell { row: 0, column: 7 },
];

/// The energy's two digits (`$80:9BF3`).
const ENERGY_DIGITS: Cell = Cell { row: 2, column: 6 };
/// The counts' first digits (`$80:9C10`, `$80:9C26`, `$80:9C4F`).
const MISSILE_DIGITS: Cell = Cell { row: 2, column: 10 };
const SUPER_MISSILE_DIGITS: Cell = Cell { row: 2, column: 14 };
const POWER_BOMB_DIGITS: Cell = Cell { row: 2, column: 17 };

/// The AUTO reserve sign's top-left cell (`$80:9B64`); it is two cells wide, three high.
const RESERVE_SIGN: Cell = Cell { row: 0, column: 8 };

/// A sign's tile flipped top to bottom.
const fn flipped_y(entry: TilemapEntry) -> TilemapEntry {
    TilemapEntry {
        flip_y: true,
        ..entry
    }
}

/// The AUTO sign with energy in the reserve tanks, row by row (`$80:998B`).
pub const AUTO_RESERVE: [TilemapEntry; 6] = [
    hud_tile(0x33, 7),
    hud_tile(0x46, 7),
    hud_tile(0x47, 7),
    hud_tile(0x48, 7),
    flipped_y(hud_tile(0x33, 7)),
    flipped_y(hud_tile(0x46, 7)),
];
/// The AUTO sign with the reserve tanks empty (`$80:9997`).
pub const EMPTY_AUTO_RESERVE: [TilemapEntry; 6] = [
    hud_tile(0x33, 3),
    hud_tile(0x46, 3),
    hud_tile(0x47, 3),
    hud_tile(0x48, 3),
    flipped_y(hud_tile(0x33, 3)),
    flipped_y(hud_tile(0x46, 3)),
];

/// The highlighted item's palette (`$80:9C5D`).
pub const HIGHLIGHTED: BgPalette = BgPalette(4);
/// Every other item's palette (`$80:9C66`).
pub const PLAIN: BgPalette = BgPalette(5);

/// The auto-cancel flash's half-period: the icon highlights while this bit of the 8-bit
/// requested NMI count is set (`$80:9C9C`).
const FLASH_BIT: u8 = 0x10;

/// An item icon's top-left cell (`$80:9D6E`).
const fn icon_cell(item: HudItem) -> Cell {
    let column = match item {
        HudItem::Missiles => 10,
        HudItem::SuperMissiles => 14,
        HudItem::PowerBombs => 17,
        HudItem::GrappleBeam => 20,
        HudItem::XRayScope => 23,
    };
    Cell { row: 0, column }
}

/// An item icon's cells, as row and column offsets from its top-left: missiles are three
/// tiles wide, every other icon two (`$80:9CEA`).
const fn icon_offsets(item: HudItem) -> &'static [(usize, usize)] {
    const TWO_WIDE: [(usize, usize); 4] = [(0, 0), (0, 1), (1, 0), (1, 1)];
    const THREE_WIDE: [(usize, usize); 6] = [(0, 0), (0, 1), (1, 0), (1, 1), (0, 2), (1, 2)];
    match item {
        HudItem::Missiles => &THREE_WIDE,
        HudItem::SuperMissiles
        | HudItem::PowerBombs
        | HudItem::GrappleBeam
        | HudItem::XRayScope => &TWO_WIDE,
    }
}

/// The gameplay pass's HUD phase.
pub fn phase(state: &mut State) {
    let requested = state.clock.counters.requested_byte;
    draw(&mut state.hud, &state.samus.equipment, requested);
    state.video.batch.vram.push(VramTransfer {
        to: HUD_TILEMAP_VRAM,
        step: VramStep::Row,
        source: VramSource::Words {
            from: HUD_TILEMAP_BUFFER,
            words: state.hud.tilemap.words(),
        },
    });
}

/// Redraw what changed since the HUD last drew it. `requested` is the 8-bit requested NMI
/// count the auto-cancel flash reads.
pub fn draw(hud: &mut Hud, equipment: &Equipment, requested: u8) {
    let tilemap = &mut hud.tilemap;
    if equipment.reserve_mode == ReserveMode::Automatic {
        let sign = if equipment.reserve.current == 0 {
            &EMPTY_AUTO_RESERVE
        } else {
            &AUTO_RESERVE
        };
        for (index, &entry) in sign.iter().enumerate() {
            tilemap.set(RESERVE_SIGN.offset(index / 2, index % 2), entry);
        }
    }
    let drawn = &mut hud.drawn;
    if equipment.energy.current != drawn.energy {
        drawn.energy = equipment.energy.current;
        draw_tanks(tilemap, equipment.energy);
        draw_two_digits(tilemap, ENERGY_DIGITS, equipment.energy.current % 100);
    }
    if let Some(missiles) = changed(equipment.missiles, &mut drawn.missiles) {
        draw_three_digits(tilemap, MISSILE_DIGITS, missiles);
    }
    // open: with the debug options' bits `1F40h` set the super missiles draw three digits
    // (`$80:9C2C`); vanilla never sets them and the State doesn't hold them.
    if let Some(supers) = changed(equipment.super_missiles, &mut drawn.super_missiles) {
        draw_two_digits(tilemap, SUPER_MISSILE_DIGITS, supers);
    }
    if let Some(bombs) = changed(equipment.power_bombs, &mut drawn.power_bombs) {
        draw_two_digits(tilemap, POWER_BOMB_DIGITS, bombs);
    }
    if hud.selected != drawn.selection {
        highlight(tilemap, &mut hud.item_palette, hud.selected, HIGHLIGHTED);
        highlight(tilemap, &mut hud.item_palette, drawn.selection, PLAIN);
        drawn.selection = hud.selected;
        // open: the switch click (sound `39h`, unless spin or wall jumping, grappling or
        // with time frozen, `$80:9C72`) joins with the sound domain (C11).
    }
    let flash = if requested & FLASH_BIT == 0 {
        PLAIN
    } else {
        HIGHLIGHTED
    };
    highlight(tilemap, &mut hud.item_palette, hud.auto_cancel, flash);
}

/// The gauge's count when it has a ceiling and differs from the one last drawn, which it
/// then becomes (`$80:9C00`).
const fn changed(gauge: Gauge, drawn: &mut u16) -> Option<u16> {
    if gauge.max == 0 || gauge.current == *drawn {
        return None;
    }
    *drawn = gauge.current;
    Some(gauge.current)
}

/// One tank per 100 of the ceiling, up to fourteen, full while whole hundreds of energy
/// remain (`$80:9BB3`).
pub fn draw_tanks(tilemap: &mut HudTilemap, energy: Gauge) {
    let full = usize::from(energy.current / 100);
    let tanks = usize::from(energy.max / 100);
    for (index, &cell) in TANK_CELLS.iter().take(tanks).enumerate() {
        tilemap.set(cell, if index < full { FULL_TANK } else { EMPTY_TANK });
    }
}

/// The digit's tile.
// open: the original indexes its table with whatever the division gives, so a count past
// what the cells hold (two digits past 99, three past 999) reads the timer code after the table
// (`$80:9DE7`); no vanilla count gets there, and the cell keeps its tile here.
fn digit(value: u16) -> Option<TilemapEntry> {
    DIGITS.get(usize::from(value)).copied()
}

/// Draw `value`'s tens and units from `cell` rightward (`$80:9D98`).
pub fn draw_two_digits(tilemap: &mut HudTilemap, cell: Cell, value: u16) {
    for (offset, place) in [(0, value / 10), (1, value % 10)] {
        if let Some(entry) = digit(place) {
            tilemap.set(cell.offset(0, offset), entry);
        }
    }
}

/// Draw `value`'s hundreds, then its last two digits (`$80:9D78`).
pub fn draw_three_digits(tilemap: &mut HudTilemap, cell: Cell, value: u16) {
    if let Some(entry) = digit(value / 100) {
        tilemap.set(cell, entry);
    }
    draw_two_digits(tilemap, cell.offset(0, 1), value % 100);
}

/// Set `item`'s icon to `palette`, leaving its blank cells blank; the palette is kept even
/// when there is no item (`$80:9CEA`).
pub fn highlight(
    tilemap: &mut HudTilemap,
    item_palette: &mut BgPalette,
    item: Option<HudItem>,
    palette: BgPalette,
) {
    *item_palette = palette;
    let Some(item) = item else {
        return;
    };
    let corner = icon_cell(item);
    for &(rows, columns) in icon_offsets(item) {
        let cell = corner.offset(rows, columns);
        if let Some(entry) = tilemap.get(cell)
            && entry != BLANK
        {
            tilemap.set(cell, entry.in_palette(palette));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BLANK, Cell, DIGITS, EMPTY_TANK, FULL_TANK, HIGHLIGHTED, PLAIN, TANK_CELLS, draw,
        draw_tanks, draw_three_digits, draw_two_digits, highlight,
    };
    use crate::state::hud::{Hud, HudItem, HudTilemap};
    use crate::state::samus::{Equipment, Gauge};
    use crate::state::video::{BgPalette, TilemapEntry};

    fn blank() -> HudTilemap {
        HudTilemap {
            cells: [BLANK; crate::state::hud::HUD_CELLS],
        }
    }

    fn word(tilemap: &HudTilemap, row: usize, column: usize) -> u16 {
        tilemap
            .get(Cell { row, column })
            .map_or(0, TilemapEntry::word)
    }

    #[test]
    fn the_tiles_are_the_originals_words() {
        let digits: Vec<u16> = DIGITS.iter().map(|entry| entry.word()).collect();
        assert_eq!(
            digits,
            [
                0x2C09, 0x2C00, 0x2C01, 0x2C02, 0x2C03, 0x2C04, 0x2C05, 0x2C06, 0x2C07, 0x2C08
            ]
        );
        assert_eq!(FULL_TANK.word(), 0x2831);
        assert_eq!(EMPTY_TANK.word(), 0x3430);
        assert_eq!(BLANK.word(), 0x2C0F);
        let auto: Vec<u16> = super::AUTO_RESERVE.iter().map(|e| e.word()).collect();
        assert_eq!(auto, [0x3C33, 0x3C46, 0x3C47, 0x3C48, 0xBC33, 0xBC46]);
        let empty: Vec<u16> = super::EMPTY_AUTO_RESERVE.iter().map(|e| e.word()).collect();
        assert_eq!(empty, [0x2C33, 0x2C46, 0x2C47, 0x2C48, 0xAC33, 0xAC46]);
    }

    #[test]
    fn every_tilemap_word_round_trips() {
        for word in 0..=u16::MAX {
            assert_eq!(TilemapEntry::from_word(word).word(), word);
        }
    }

    #[test]
    fn digits_draw_from_a_value() {
        let mut tilemap = blank();
        draw_two_digits(&mut tilemap, Cell { row: 2, column: 6 }, 47);
        assert_eq!(word(&tilemap, 2, 6), 0x2C03);
        assert_eq!(word(&tilemap, 2, 7), 0x2C06);
        draw_three_digits(&mut tilemap, Cell { row: 2, column: 10 }, 105);
        assert_eq!(word(&tilemap, 2, 10), 0x2C00);
        assert_eq!(word(&tilemap, 2, 11), 0x2C09);
        assert_eq!(word(&tilemap, 2, 12), 0x2C04);
    }

    #[test]
    fn tanks_fill_bottom_row_first() {
        let mut tilemap = blank();
        draw_tanks(
            &mut tilemap,
            Gauge {
                current: 299,
                max: 899,
            },
        );
        let drawn: Vec<u16> = TANK_CELLS
            .iter()
            .map(|&cell| tilemap.get(cell).map_or(0, TilemapEntry::word))
            .collect();
        let mut expected = vec![
            0x2831, 0x2831, 0x3430, 0x3430, 0x3430, 0x3430, 0x3430, 0x3430,
        ];
        expected.extend([0x2C0F; 6]);
        assert_eq!(drawn, expected);
    }

    #[test]
    fn energy_redraws_only_when_it_changes() {
        let mut hud = Hud {
            tilemap: blank(),
            ..Hud::default()
        };
        let mut equipment = Equipment {
            energy: Gauge {
                current: 99,
                max: 99,
            },
            ..Equipment::default()
        };
        draw(&mut hud, &equipment, 0);
        assert_eq!(hud.drawn.energy, 99);
        assert_eq!(word(&hud.tilemap, 2, 6), 0x2C08);
        hud.tilemap = blank();
        draw(&mut hud, &equipment, 0);
        assert_eq!(word(&hud.tilemap, 2, 6), 0x2C0F);
        equipment.energy.current = 30;
        draw(&mut hud, &equipment, 0);
        assert_eq!(word(&hud.tilemap, 2, 6), 0x2C02);
        assert_eq!(word(&hud.tilemap, 2, 7), 0x2C09);
    }

    #[test]
    fn a_highlight_keeps_blank_cells_and_the_palette() {
        let mut tilemap = blank();
        let icon = TilemapEntry::from_word(0x344B);
        tilemap.set(Cell { row: 0, column: 10 }, icon);
        let mut palette = BgPalette::default();
        highlight(
            &mut tilemap,
            &mut palette,
            Some(HudItem::Missiles),
            HIGHLIGHTED,
        );
        assert_eq!(word(&tilemap, 0, 10), 0x304B);
        assert_eq!(word(&tilemap, 0, 11), 0x2C0F);
        assert_eq!(palette, HIGHLIGHTED);
        highlight(&mut tilemap, &mut palette, None, PLAIN);
        assert_eq!(palette, PLAIN);
        assert_eq!(word(&tilemap, 0, 10), 0x304B);
    }
}
