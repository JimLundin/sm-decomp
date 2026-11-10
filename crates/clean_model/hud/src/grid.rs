//! A grid of cells, each holding a value: the HUD's cells, her map's pictures and the sets of
//! her map's cells are each one. A place outside the grid has no cell: reading it gives nothing
//! and writing it changes nothing. Every cell keeps its value until a rule writes it.

use clean_types::Pair;

/// A place on a grid: `x` across from the left, `y` down from the top, both from 0. It may lie
/// outside the grid.
pub type Point = Pair<i32>;

/// The place `x` across and `y` down.
pub const fn at(x: i32, y: i32) -> Point {
    Pair { x, y }
}

/// The place `by` across and down from `point`.
pub const fn plus(point: Point, by: Point) -> Point {
    at(point.x.saturating_add(by.x), point.y.saturating_add(by.y))
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

    /// What the cell at `point` holds, or `None` outside the grid.
    #[must_use]
    pub fn get(&self, point: Point) -> Option<T> {
        let (x, y) = index(point)?;
        self.rows.get(y)?.get(x).copied()
    }

    /// Puts `value` in the cell at `point`; outside the grid, nothing changes.
    pub fn set(&mut self, point: Point, value: T) {
        let Some((x, y)) = index(point) else {
            return;
        };
        if let Some(cell) = self.rows.get_mut(y).and_then(|row| row.get_mut(x)) {
            *cell = value;
        }
    }
}

/// A grid of `bool` is a set of its cells, each `true` in it.
impl<const COLUMNS: usize, const ROWS: usize> Grid<bool, COLUMNS, ROWS> {
    /// Whether the cell at `point` is in the set; a place outside the grid never is.
    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        self.get(point) == Some(true)
    }

    /// The cell at `point` joins the set; outside the grid, nothing changes.
    pub fn insert(&mut self, point: Point) {
        self.set(point, true);
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
fn index(point: Point) -> Option<(usize, usize)> {
    Some((
        usize::try_from(point.x).ok()?,
        usize::try_from(point.y).ok()?,
    ))
}
