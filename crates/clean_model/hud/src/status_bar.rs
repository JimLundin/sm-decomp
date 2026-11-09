//! The status bar (§2): the energy and its tanks, the ammunition counts, the reserve sign, and the
//! items' icons with the selection's highlight.

use crate::inputs::{Ammunition, Energy, FrameCount, Reserve, ReserveMode};
use crate::item::SelectableItem;
use crate::pictures::{
    BLANK, ColourSet, EMPTY_TANK, ENERGY_DIGITS, FULL_TANK, HIGHLIGHTED, HudGrid, HudPos, PLAIN,
    RESERVE_SIGN, SIGN_DIM, SIGN_LIT, TANKS, Tile, digit, icon,
};
use crate::rules::Rules;
use crate::state::HudState;

/// Spec: EnergyGauge. The energy one tank stands for.
const ENERGY_PER_TANK: u16 = 100;
/// The digits are decimal.
const DECIMAL: u16 = 10;

impl HudState {
    /// Spec: EnergyGauge. When her energy isn't the energy last shown, it becomes the one shown:
    /// one tank per 100 of her capacity, as many as fit (14), full while whole hundreds remain
    /// and empty after, and her energy's last two digits. Tiles past her capacity keep what they
    /// show.
    pub(crate) fn energy_gauge(&mut self, energy: Energy) {
        if energy.now == self.shown.energy {
            return;
        }
        self.shown.energy = energy.now;
        let tanks = usize::from(energy.capacity / ENERGY_PER_TANK);
        let full = usize::from(energy.now / ENERGY_PER_TANK);
        for (tank, &pos) in TANKS.iter().enumerate().take(tanks) {
            let tile = if tank < full { FULL_TANK } else { EMPTY_TANK };
            self.grid.set(pos, tile);
        }
        draw_digits(&mut self.grid, &ENERGY_DIGITS, energy.now);
    }

    /// Spec: AmmoCounts. Each count whose capacity isn't 0 and isn't the count last shown
    /// becomes the one shown, in its kind's digits. With a capacity of 0 nothing is drawn and
    /// the count last shown stays.
    pub(crate) fn ammo_counts(&mut self, ammunition: &Ammunition) {
        for ((kind, stock), shown) in ammunition.kinds().zip(&mut self.shown.ammo) {
            if stock.capacity != 0 && stock.count != *shown {
                *shown = stock.count;
                draw_digits(&mut self.grid, kind.digits, stock.count);
            }
        }
    }

    /// Spec: AutoReserveSign. While the reserve tanks refill on their own, the sign shows, lit
    /// with energy in them and dim when they are empty. Set to refill by hand, its tiles keep
    /// what they show.
    pub(crate) fn reserve_sign(&mut self, reserve: Reserve) {
        if reserve.mode == ReserveMode::Manual {
            return;
        }
        let colours = if reserve.energy == 0 {
            SIGN_DIM
        } else {
            SIGN_LIT
        };
        for placed in RESERVE_SIGN {
            let tile = Tile::new(placed.picture, colours, placed.flip);
            self.grid.set(placed.pos, tile);
        }
    }

    /// Spec: IconsAppear. A pickup draws its item's icon in the plain colours when the icon's
    /// top-left tile shows the blank's picture, whatever its colours; otherwise its tiles keep
    /// what they show.
    pub(crate) fn icon_appears(&mut self, item: SelectableItem) {
        let tiles = icon(item);
        let blank = tiles
            .first()
            .is_some_and(|top_left| self.grid.get(top_left.pos).picture == BLANK.picture);
        if !blank {
            return;
        }
        for placed in tiles {
            let tile = Tile::new(placed.picture, PLAIN, placed.flip);
            self.grid.set(placed.pos, tile);
        }
    }

    /// Spec: SelectionHighlight. When the selected item isn't the item shown highlighted, its
    /// icon takes the highlighted colours, the one shown takes the plain ones, and the selected
    /// item becomes the one shown. Returns whether the item shown changed; from an item to none
    /// is a change like any other.
    pub(crate) fn highlight(&mut self) -> bool {
        let selected = self.selection.selected;
        if selected == self.shown.highlighted {
            return false;
        }
        if let Some(item) = selected {
            self.colour_icon(item, HIGHLIGHTED);
        }
        if let Some(item) = self.shown.highlighted {
            self.colour_icon(item, PLAIN);
        }
        self.shown.highlighted = selected;
        true
    }

    /// Spec: OneShotFlash. After the highlight, the one-shot item's icon takes the plain colours
    /// for the first 16 frames of each 32 and the highlighted ones for the next 16, though it is
    /// the selected item.
    pub(crate) fn one_shot_flash(&mut self, rules: &Rules, frame: FrameCount) {
        let Some(item) = self.selection.one_shot else {
            return;
        };
        let colours = if rules.one_shot_flash.first_half(frame.0) {
            PLAIN
        } else {
            HIGHLIGHTED
        };
        self.colour_icon(item, colours);
    }

    /// Spec: SelectionHighlight. An icon takes `colours`: each of its tiles keeps its picture and
    /// flips in the new colours, and a tile showing the blank keeps it.
    fn colour_icon(&mut self, item: SelectableItem, colours: ColourSet) {
        for placed in icon(item) {
            let tile = self.grid.get(placed.pos);
            if tile != BLANK {
                self.grid.set(placed.pos, tile.in_colours(colours));
            }
        }
    }
}

/// Draws the last digits of `value`, one per tile, the lowest in the last tile.
fn draw_digits(grid: &mut HudGrid, tiles: &[HudPos], value: u16) {
    let mut rest = value;
    for &pos in tiles.iter().rev() {
        grid.set(pos, digit(rest % DECIMAL));
        rest /= DECIMAL;
    }
}
