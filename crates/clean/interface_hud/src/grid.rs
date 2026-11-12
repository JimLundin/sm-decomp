//! A grid of cells, each holding a value: the HUD's cells, her map's pictures and the sets of
//! her map's cells are each one. A place outside the grid has no cell: reading it gives nothing
//! and writing it changes nothing. Every cell keeps its value until a rule writes it.

use types::Pair;

/// A place on one of the HUD's grids, column first (Notation): `x` its column, counted across
/// from the left, `y` its row, counted down from the top, both from 0. It may lie outside the
/// grid.
pub type Place = Pair<i32>;

/// The place in column `x` and row `y`.
pub const fn at(x: i32, y: i32) -> Place {
    Pair { x, y }
}

/// The place `by` across and down from `place`.
pub const fn plus(place: Place, by: Place) -> Place {
    at(place.x.saturating_add(by.x), place.y.saturating_add(by.y))
}

/// A grid `COLUMNS` across by `ROWS` down, each cell holding a `T`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Grid<T, const COLUMNS: usize, const ROWS: usize> {
    rows: [[T; COLUMNS]; ROWS],
}

impl<T: Copy, const COLUMNS: usize, const ROWS: usize> Grid<T, COLUMNS, ROWS> {
    /// A grid holding `value` in every cell.
    #[must_use]
    pub const fn filled(value: T) -> Self {
        Self {
            rows: [[value; COLUMNS]; ROWS],
        }
    }

    /// What the cell at `place` holds, or `None` outside the grid.
    #[must_use]
    pub fn get(&self, place: Place) -> Option<T> {
        let (x, y) = index(place)?;
        self.rows.get(y)?.get(x).copied()
    }

    /// Puts `value` in the cell at `place`; outside the grid, nothing changes.
    pub fn set(&mut self, place: Place, value: T) {
        let Some((x, y)) = index(place) else {
            return;
        };
        if let Some(cell) = self.rows.get_mut(y).and_then(|row| row.get_mut(x)) {
            *cell = value;
        }
    }
}

/// A grid of `bool` is a set of its cells, each `true` in it.
impl<const COLUMNS: usize, const ROWS: usize> Grid<bool, COLUMNS, ROWS> {
    /// Whether the cell at `place` is in the set; a place outside the grid never is.
    #[must_use]
    pub fn contains(&self, place: Place) -> bool {
        self.get(place) == Some(true)
    }

    /// The cell at `place` joins the set; outside the grid, nothing changes.
    pub fn insert(&mut self, place: Place) {
        self.set(place, true);
    }
}

impl<T: Copy + Default, const COLUMNS: usize, const ROWS: usize> Default
    for Grid<T, COLUMNS, ROWS>
{
    /// A grid holding `T`'s default in every cell: for a set of cells, the empty set.
    fn default() -> Self {
        Self::filled(T::default())
    }
}

/// A place's column and row as indices, or `None` left of or above the grid.
fn index(place: Place) -> Option<(usize, usize)> {
    Some((
        usize::try_from(place.x).ok()?,
        usize::try_from(place.y).ok()?,
    ))
}
