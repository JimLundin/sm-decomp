//! A grid of cells, each holding a value: the HUD's tiles, her map's pictures and the sets of her
//! map's cells are each one. A position lies within its grid, so a cell read or written is
//! always there; every cell keeps its value until a rule writes it.

use clean_types::Pair;

/// A cell's place in a grid `COLUMNS` across by `ROWS` down: `x` across from the left, `y` down
/// from the top, both from 0.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pos<const COLUMNS: usize, const ROWS: usize> {
    x: u8,
    y: u8,
}

impl<const COLUMNS: usize, const ROWS: usize> Pos<COLUMNS, ROWS> {
    /// The cell `x` across and `y` down, or `None` outside the grid. Every position worked out
    /// while the game runs is checked here.
    #[must_use]
    pub fn new(x: i32, y: i32) -> Option<Self> {
        let x = u8::try_from(x).ok().filter(|&x| usize::from(x) < COLUMNS)?;
        let y = u8::try_from(y).ok().filter(|&y| usize::from(y) < ROWS)?;
        Some(Self { x, y })
    }

    /// A place the spec's fixed layout gives, which lies in the grid.
    pub(crate) const fn at(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    /// Its column, across from the left.
    #[must_use]
    pub const fn x(self) -> u8 {
        self.x
    }

    /// Its row, down from the top.
    #[must_use]
    pub const fn y(self) -> u8 {
        self.y
    }

    /// The place `by` across and down from this one, if it lies in the grid.
    pub(crate) fn offset(self, by: Pair<i32>) -> Option<Self> {
        Self::new(
            i32::from(self.x).saturating_add(by.x),
            i32::from(self.y).saturating_add(by.y),
        )
    }

    /// Every place of the grid, the top row first, each row left to right.
    pub fn all() -> impl Iterator<Item = Self> {
        (0..=u8::MAX)
            .take(ROWS)
            .flat_map(|y| (0..=u8::MAX).take(COLUMNS).map(move |x| Self { x, y }))
    }
}

/// A grid `COLUMNS` across by `ROWS` down, each cell holding a `T`; a new one holds `T`'s
/// default in every cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Grid<T, const COLUMNS: usize, const ROWS: usize> {
    rows: [[T; COLUMNS]; ROWS],
}

impl<T: Copy + Default, const COLUMNS: usize, const ROWS: usize> Default
    for Grid<T, COLUMNS, ROWS>
{
    fn default() -> Self {
        Self {
            rows: [[T::default(); COLUMNS]; ROWS],
        }
    }
}

impl<T: Copy + Default, const COLUMNS: usize, const ROWS: usize> Grid<T, COLUMNS, ROWS> {
    /// What the cell at `pos` holds.
    #[must_use]
    pub fn get(&self, pos: Pos<COLUMNS, ROWS>) -> T {
        // A position lies within its grid, so the default is never read.
        self.rows
            .get(usize::from(pos.y))
            .and_then(|row| row.get(usize::from(pos.x)))
            .copied()
            .unwrap_or_default()
    }

    /// Puts `value` in the cell at `pos`.
    pub fn set(&mut self, pos: Pos<COLUMNS, ROWS>, value: T) {
        if let Some(cell) = self
            .rows
            .get_mut(usize::from(pos.y))
            .and_then(|row| row.get_mut(usize::from(pos.x)))
        {
            *cell = value;
        }
    }
}
