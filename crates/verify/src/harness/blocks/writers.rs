//! The other parts' cell writers, the masks' single source (`writers.toml`): each PLM of bank
//! $84 outside the blocks that writes a cell other than its own, with the cells it writes from
//! its own cell. A writer left out (a room's own cell, a PLM that moves, a setup that frees its
//! slot) masks its own cell only, like every PLM the table doesn't name, so a cell it writes past
//! that fails loudly as a mismatch.

use clean_types::{Cell, Pair};
use serde::Deserialize;

/// The table.
const TABLE: &str = include_str!("writers.toml");

/// A cell a writer writes, as an offset from its own cell: x rightward, y downward.
type Offset = Pair<i16>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Table {
    writer: Vec<Row>,
}

/// One writer of the table.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    plms: Vec<u16>,
    cells: Vec<Cells>,
    reached: Reached,
    left_out: Option<LeftOut>,
}

/// What the cached rows do with a writer.
#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Reached {
    /// A cell of its reach other than its own changes while it is live.
    Writes,
    /// A row holds it, but no other cell of its reach changes there.
    Live,
    /// No row holds it.
    No,
}

/// Why a writer's cells aren't all relative to a cell it keeps.
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum LeftOut {
    /// It writes a cell of the room wherever it sits.
    RoomAbsolute,
    /// It moves between its draws.
    Moves,
    /// Its setup writes and frees its slot in one call, so no record shows it.
    FreesItsSlot,
}

/// A point `[dx, dy]`, or a box.
#[derive(Clone, Copy, Deserialize)]
#[serde(untagged)]
enum Cells {
    Point([i16; 2]),
    Box(Span),
}

/// A box's columns and rows, each `[from, to]` with both ends included.
#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct Span {
    x: [i16; 2],
    y: [i16; 2],
}

impl Cells {
    /// Its columns and rows; `None` for a box whose span runs backwards.
    const fn span(self) -> Option<Span> {
        let span = match self {
            Self::Point([x, y]) => Span {
                x: [x, x],
                y: [y, y],
            },
            Self::Box(span) => span,
        };
        let ([left, right], [top, bottom]) = (span.x, span.y);
        if left <= right && top <= bottom {
            Some(span)
        } else {
            None
        }
    }
}

impl Span {
    fn offsets(self) -> impl Iterator<Item = Offset> {
        let ([left, right], [top, bottom]) = (self.x, self.y);
        (top..=bottom).flat_map(move |y| (left..=right).map(move |x| Pair { x, y }))
    }
}

/// The writers the harness masks by.
pub(super) struct Writers(Vec<(u16, Vec<Offset>)>);

impl Writers {
    /// The table's writers.
    ///
    /// # Errors
    ///
    /// A table that doesn't read (a key it doesn't know included), that holds a box that runs
    /// backwards, or that leaves out a writer the cached rows see write.
    pub(super) fn load() -> Result<Self, String> {
        Self::parse(TABLE)
    }

    fn parse(text: &str) -> Result<Self, String> {
        let table: Table =
            toml::from_str(text).map_err(|error| format!("the writers' table: {error}"))?;
        let mut writers = Vec::new();
        for row in table.writer {
            if row.left_out.is_some() {
                if row.reached == Reached::Writes {
                    return Err(format!("{:04X?}: left out, but the rows see it", row.plms));
                }
                continue;
            }
            let spans: Option<Vec<Span>> = row.cells.into_iter().map(Cells::span).collect();
            let Some(spans) = spans else {
                return Err(format!("{:04X?}: a box that runs backwards", row.plms));
            };
            let cells: Vec<Offset> = spans.into_iter().flat_map(Span::offsets).collect();
            writers.extend(row.plms.into_iter().map(|plm| (plm, cells.clone())));
        }
        Ok(Self(writers))
    }

    /// The cells PLM `plm` on `at` writes: its row's, or its own cell only, with the cells past
    /// the room's top or left edge left out.
    pub(super) fn cells(&self, plm: u16, at: Cell) -> impl Iterator<Item = Cell> {
        const OWN: &[Offset] = &[Pair { x: 0, y: 0 }];
        let reach = self
            .0
            .iter()
            .find_map(|(id, cells)| (*id == plm).then_some(cells.as_slice()))
            .unwrap_or(OWN);
        reach.iter().filter_map(move |offset| {
            Some(Cell {
                x: at.x.checked_add_signed(offset.x)?,
                y: at.y.checked_add_signed(offset.y)?,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{TABLE, Table, Writers};
    use clean_types::Cell;

    /// The table reads: its keys, its boxes, and no writer left out that the rows see write.
    #[test]
    fn the_table_reads() {
        assert!(Writers::load().is_ok());
    }

    /// A box with a key it doesn't know, or one that runs backwards, is refused.
    #[test]
    fn a_box_with_an_unknown_key_or_backwards_is_refused() {
        let table = |cells: &str| {
            format!(
                "[[writer]]
plms = [1]
cells = [{cells}]
reached = \"no\"
"
            )
        };
        assert!(Writers::parse(&table("{ x = [0, 1], y = [0, 3] }")).is_ok());
        assert!(Writers::parse(&table("{ x = [0, 1], y = [0, 3], z = [0, 0] }")).is_err());
        assert!(Writers::parse(&table("{ x = [1, 0], y = [0, 3] }")).is_err());
        assert!(Writers::parse(&table("{ x = [0, 1], y = [3, 0] }")).is_err());
    }

    /// Every writer left out has its reason, and one the cached rows see write is refused, so it
    /// can't hide.
    #[test]
    fn a_left_out_writer_has_its_reason_and_the_rows_never_see_it() {
        let table = "[[writer]]\nplms = [1]\ncells = [[0, 0]]\nreached = \"writes\"\n";
        assert!(Writers::parse(table).is_ok());
        let left_out = format!("{table}left_out = \"moves\"\n");
        assert!(Writers::parse(&left_out).is_err());
        assert!(Writers::parse(&left_out.replace("writes", "no")).is_ok());
        assert!(Writers::parse(&format!("{table}left_out = \"for no reason\"\n")).is_err());
    }

    /// No PLM is named twice, so each has one reach.
    #[test]
    fn every_plm_is_named_once() {
        let table: Table = toml::from_str(TABLE).unwrap();
        let mut plms: Vec<u16> = table.writer.into_iter().flat_map(|row| row.plms).collect();
        let count = plms.len();
        plms.sort_unstable();
        plms.dedup();
        assert_eq!(plms.len(), count);
    }

    /// The forms give the cells the table names: a column, points and a row, two points either
    /// side, a box reaching left and up, a run, a writer left out and a PLM the table doesn't name.
    #[test]
    fn the_forms_give_the_cells_the_table_names() {
        let writers = Writers::load().unwrap();
        let at = Cell { x: 10, y: 10 };
        let cells = |plm| {
            let mut cells: Vec<_> = writers
                .cells(plm, at)
                .map(|cell| (cell.x, cell.y))
                .collect();
            cells.sort_unstable();
            cells
        };
        assert_eq!(cells(0xC842), [(10, 10), (10, 11), (10, 12), (10, 13)]);
        assert_eq!(
            cells(0xB70B),
            [(10, 10), (10, 11), (11, 11), (12, 11), (13, 10), (13, 11)]
        );
        assert_eq!(cells(0xB6D7), [(7, 10), (10, 10)]);
        assert_eq!(cells(0xD6EA).len(), 16);
        assert_eq!(cells(0xB6B7).len(), 5);
        assert_eq!(cells(0xB6A3), cells(0xB67B));
        assert_eq!(cells(0xD6D6), [(10, 10)]);
        assert_eq!(cells(0xB8EB), [(10, 10)]);
    }
}
