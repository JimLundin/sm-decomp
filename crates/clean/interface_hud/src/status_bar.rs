//! The status bar (§2): the energy and its tanks, the ammunition counts, the reserve sign, and the
//! items' icons with the selection's highlight.

use std::{iter, mem};

use types::Look;

use crate::grid::Place;
use crate::inputs::{Ammunition, Energy, FrameCount, Reserve, ReserveMode};
use crate::item::SelectableItem;
use crate::layout::{ENERGY_DIGITS, RESERVE_SIGN, TANKS, ammo_digits, icon_top_left, laid};
use crate::pictures::{HudPictures, rows};
use crate::rules::Rules;
use crate::state::HudState;
use crate::tile::{
    ColourSet, DIGIT_COLOURS, EMPTY_TANK_COLOURS, FULL_TANK_COLOURS, HIGHLIGHTED, PLAIN, SIGN_DIM,
    SIGN_LIT, Tile,
};

/// Spec: EnergyGauge. The energy one tank stands for.
const ENERGY_PER_TANK: u16 = 100;
/// The digits are decimal.
const DECIMAL: u16 = 10;

impl HudState {
    /// Spec: EnergyGauge. When her energy isn't the energy last shown, it becomes the one shown:
    /// one tank per 100 of her capacity, as many as fit (14), full while whole hundreds remain
    /// and empty after, and her energy's last two digits. Cells past her capacity keep what they
    /// show.
    pub(crate) fn energy_gauge(&mut self, pictures: &HudPictures, energy: Energy) {
        if mem::replace(&mut self.shown.energy, energy.now) == energy.now {
            return;
        }
        self.draw_number(pictures, &ENERGY_DIGITS, energy.now);
        let full_tank = Tile::new(pictures.full_tank, FULL_TANK_COLOURS);
        let empty_tank = Tile::new(pictures.empty_tank, EMPTY_TANK_COLOURS);
        let tanks = usize::from(energy.capacity / ENERGY_PER_TANK);
        let full = usize::from(energy.now / ENERGY_PER_TANK);
        for (tank, &place) in TANKS.iter().enumerate().take(tanks) {
            self.grid
                .set(place, if tank < full { full_tank } else { empty_tank });
        }
    }

    /// Spec: AmmoCounts. Each count whose capacity isn't 0 and isn't the count last shown
    /// becomes the one shown, in its digits. With a capacity of 0 nothing is drawn and the count
    /// last shown stays.
    pub(crate) fn ammo_counts(&mut self, pictures: &HudPictures, ammunition: &Ammunition) {
        for (ammo, stock) in ammunition.stocks() {
            if stock.capacity != 0
                && mem::replace(&mut self.shown.ammo[ammo], stock.count) != stock.count
            {
                self.draw_number(pictures, ammo_digits(ammo), stock.count);
            }
        }
    }

    /// Spec: EnergyGauge, AmmoCounts. Draws `value`'s last digits in `places`, one per cell, the
    /// lowest in the last, digit `d`'s picture the `d`-th.
    fn draw_number(&mut self, pictures: &HudPictures, places: &[Place], value: u16) {
        let digits = iter::successors(Some(value), |rest| Some(rest / DECIMAL))
            .map(|rest| usize::from(rest % DECIMAL));
        let cells = places
            .iter()
            .rev()
            .zip(digits)
            .filter_map(|(&place, digit)| Some((place, *pictures.digits.get(digit)?)));
        self.write(cells, DIGIT_COLOURS);
    }

    /// Spec: AutoReserveSign. While the reserve tanks refill on their own, the sign shows, lit
    /// with energy in them and dim when they are empty. Set to refill by hand, its cells keep
    /// what they show.
    pub(crate) fn reserve_sign(&mut self, pictures: &HudPictures, reserve: Reserve) {
        if reserve.mode == ReserveMode::Manual {
            return;
        }
        let colours = if reserve.energy == 0 {
            SIGN_DIM
        } else {
            SIGN_LIT
        };
        self.write(laid(RESERVE_SIGN, rows(&pictures.reserve_sign)), colours);
    }

    /// Spec: IconsAppear. A pickup draws its item's icon in the plain colours when the icon's
    /// top-left cell shows the blank's picture, whatever its colours and flips: the picture
    /// alone is tested. Otherwise its cells keep what they show.
    pub(crate) fn icon_appears(&mut self, pictures: &HudPictures, item: SelectableItem) {
        let top_left = icon_top_left(item);
        let shows_blank_picture = self
            .grid
            .get(top_left)
            .is_some_and(|tile| tile.look.picture == pictures.blank);
        if shows_blank_picture {
            self.write(laid(top_left, pictures.icons.of(item)), PLAIN);
        }
    }

    /// Writes each picture, with its flips, in the cell at its place, all in `colours`.
    fn write(&mut self, cells: impl IntoIterator<Item = (Place, Look)>, colours: ColourSet) {
        for (place, look) in cells {
            self.grid.set(place, Tile::new(look, colours));
        }
    }

    /// Spec: SelectionHighlight. When the selected item isn't the item shown highlighted, its
    /// icon takes the highlighted colours, the one shown takes the plain ones, and the selected
    /// item becomes the one shown.
    pub(crate) fn highlight(&mut self, pictures: &HudPictures) {
        let selected = self.selection.item();
        if selected == self.shown.highlighted {
            return;
        }
        for (icon, colours) in [(selected, HIGHLIGHTED), (self.shown.highlighted, PLAIN)] {
            if let Some(item) = icon {
                self.colour_icon(pictures, item, colours);
            }
        }
        self.shown.highlighted = selected;
    }

    /// Spec: OneShotFlash. After the highlight, the one-shot item's icon takes the plain colours
    /// in the first half of the flash's cycle and the highlighted ones in the second, though it
    /// is the selected item.
    pub(crate) fn one_shot_flash(
        &mut self,
        rules: &Rules,
        pictures: &HudPictures,
        frame: FrameCount,
    ) {
        let Some(item) = self.selection.one_shot() else {
            return;
        };
        let shown_plain = rules.one_shot_flash.first_half(frame);
        let colours = if shown_plain { PLAIN } else { HIGHLIGHTED };
        self.colour_icon(pictures, item, colours);
    }

    /// Spec: SelectionHighlight. An icon takes `colours`: each of its cells keeps its picture and
    /// flips in the new colours, but a cell showing the blank cell, the whole cell tested, keeps
    /// it.
    fn colour_icon(&mut self, pictures: &HudPictures, item: SelectableItem, colours: ColourSet) {
        let blank_cell = pictures.blank_cell();
        for (place, _) in laid(icon_top_left(item), pictures.icons.of(item)) {
            if let Some(tile) = self.grid.get(place).filter(|&tile| tile != blank_cell) {
                self.grid.set(place, tile.in_colours(colours));
            }
        }
    }
}
