//! The status bar (§2): the energy and its tanks, the ammunition counts, the reserve sign, and the
//! items' icons with the selection's highlight.

use crate::grid::{ColourSet, GridPos, HudGrid, Tile};
use crate::inputs::{Ammo, Energy, FrameCount, FrameInputs, Reserve, ReserveMode, Status};
use crate::item::SelectableItem;
use crate::pictures::{
    BLANK, EMPTY_TANK, ENERGY_DIGITS, FULL_TANK, HIGHLIGHTED, PLAIN, RESERVE_SIGN, SIGN_DIM,
    SIGN_LIT, TANKS, ammo_digits, digit, icon,
};
use crate::rules::Rules;
use crate::state::{Selection, Shown};

/// Spec: EnergyGauge. The energy one tank stands for.
const ENERGY_PER_TANK: u16 = 100;
/// The digits are decimal.
const DECIMAL: u16 = 10;

/// Draws the status bar from the frame's inputs and the selection as it now is, in the order §5
/// gives: the energy, the counts and the sign, then a pickup's icon, then the highlight, then the
/// one-shot item's flash. Returns whether the highlight moved (spec: `SelectionHighlight`'s first
/// rule).
pub fn draw_status_bar(
    rules: &Rules,
    inputs: &FrameInputs<'_>,
    selection: Selection,
    grid: &mut HudGrid,
    shown: &mut Shown,
) -> bool {
    energy_gauge(grid, shown, inputs.status.energy);
    ammo_counts(grid, shown, &inputs.status);
    reserve_sign(grid, inputs.status.reserve);
    if let Some(item) = inputs.collected {
        icon_appears(grid, item);
    }
    let moved = highlight(grid, shown, selection.selected);
    if let Some(item) = selection.one_shot {
        one_shot_flash(rules, inputs.frame, grid, item);
    }
    moved
}

/// Spec: OneShotFlash. After the highlight, the one-shot item's icon takes the plain colours for
/// the first 16 frames of each 32 and the highlighted ones for the next 16, though it is the
/// selected item.
fn one_shot_flash(rules: &Rules, frame: FrameCount, grid: &mut HudGrid, item: SelectableItem) {
    let colours = if rules.one_shot_flash.first_half(frame.0) {
        PLAIN
    } else {
        HIGHLIGHTED
    };
    colour_icon(grid, item, colours);
}

/// Spec: EnergyGauge. When her energy isn't the energy last shown, it becomes the one shown: one
/// tank per 100 of her capacity, as many as fit (14), full while whole hundreds remain and empty
/// after, and her energy's last two digits. Tiles past her capacity keep what they show.
fn energy_gauge(grid: &mut HudGrid, shown: &mut Shown, energy: Energy) {
    if energy.now == shown.energy {
        return;
    }
    shown.energy = energy.now;
    let tanks = usize::from(energy.capacity / ENERGY_PER_TANK);
    let full = usize::from(energy.now / ENERGY_PER_TANK);
    for (tank, &pos) in TANKS.iter().enumerate().take(tanks) {
        let tile = if tank < full { FULL_TANK } else { EMPTY_TANK };
        grid.set(pos, tile);
    }
    draw_digits(grid, &ENERGY_DIGITS, energy.now);
}

/// Spec: AmmoCounts. A count whose capacity isn't 0 and isn't the count last shown becomes the
/// one shown, in its digits. With a capacity of 0 nothing is drawn and the count last shown
/// stays.
fn ammo_counts(grid: &mut HudGrid, shown: &mut Shown, status: &Status) {
    for ammo in Ammo::ALL {
        let stock = status.ammunition.stock(ammo);
        if stock.capacity != 0 && stock.count != shown.ammo(ammo) {
            shown.set_ammo(ammo, stock.count);
            draw_digits(grid, ammo_digits(ammo), stock.count);
        }
    }
}

/// Draws the last digits of `value`, one per tile, the lowest in the last tile.
fn draw_digits(grid: &mut HudGrid, tiles: &[GridPos], value: u16) {
    let mut rest = value;
    for &pos in tiles.iter().rev() {
        grid.set(pos, digit(rest % DECIMAL));
        rest /= DECIMAL;
    }
}

/// Spec: AutoReserveSign. While the reserve tanks refill on their own, the sign shows, lit with
/// energy in them and dim when they are empty. Set to refill by hand, its tiles keep what they
/// show.
fn reserve_sign(grid: &mut HudGrid, reserve: Reserve) {
    if reserve.mode == ReserveMode::Manual {
        return;
    }
    let colours = if reserve.energy == 0 {
        SIGN_DIM
    } else {
        SIGN_LIT
    };
    for placed in RESERVE_SIGN {
        grid.set(placed.pos, Tile::new(placed.picture, colours, placed.flip));
    }
}

/// Spec: IconsAppear. A pickup draws its item's icon in the plain colours when the icon's
/// top-left tile shows the blank's picture, whatever its colours; otherwise its tiles keep what
/// they show.
fn icon_appears(grid: &mut HudGrid, item: SelectableItem) {
    let tiles = icon(item);
    let blank = tiles
        .first()
        .is_some_and(|top_left| grid.tile(top_left.pos).picture == BLANK.picture);
    if !blank {
        return;
    }
    for placed in tiles {
        grid.set(placed.pos, Tile::new(placed.picture, PLAIN, placed.flip));
    }
}

/// Spec: SelectionHighlight. When the selected item isn't the one last shown, its icon takes the
/// highlighted colours, the last shown's takes the plain ones, and the selected item becomes the
/// one shown. Returns whether it moved.
fn highlight(grid: &mut HudGrid, shown: &mut Shown, selected: Option<SelectableItem>) -> bool {
    if selected == shown.selection {
        return false;
    }
    if let Some(item) = selected {
        colour_icon(grid, item, HIGHLIGHTED);
    }
    if let Some(item) = shown.selection {
        colour_icon(grid, item, PLAIN);
    }
    shown.selection = selected;
    true
}

/// Spec: SelectionHighlight. An icon takes `colours`: each of its tiles keeps its picture and
/// flips in the new colours, and a tile showing the blank keeps it.
fn colour_icon(grid: &mut HudGrid, item: SelectableItem, colours: ColourSet) {
    for placed in icon(item) {
        let tile = grid.tile(placed.pos);
        if tile != BLANK {
            grid.set(placed.pos, tile.in_colours(colours));
        }
    }
}
