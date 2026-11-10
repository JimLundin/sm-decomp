//! The other parts' cell writers, read from the decode notes' table ("The other parts' cell
//! writers", `docs/behaviours/blocks.decode.md`), so the masks have one source: the observers'
//! table of the cells each PLM of bank $84 writes from its own cell.
//!
//! A row's cells are read in the table's forms: a point `(dx, dy)`, and a box `x a..b, y c..d`
//! or `x a..b at y c..d`, joined by `, ` or `, and `, with a note after `:` or ` (`; a `;` gives
//! each of the row's PLMs its own. A row in any other form (a room's own cell, a PLM that moves
//! between its draws) is left out: its PLMs mask their own cell only, so a cell they write past
//! it is a mismatch, failing loudly, as the notes' "What the harness needs for them" says.

use core::ops::RangeInclusive;

use clean_types::{Cell, Pair};

/// The decode notes, which hold the table.
const NOTES: &str = include_str!("../../../../../docs/behaviours/blocks.decode.md");
/// The table's heading.
const HEADING: &str = "## The other parts' cell writers";

/// A cell a writer writes, as an offset from its own cell: x rightward, y downward.
type Offset = Pair<i16>;

/// One row of the table.
pub(super) struct Row {
    /// Its PLMs, each a run of header ids (one id, or the table's `$a..$b`).
    pub(super) plms: Vec<RangeInclusive<u16>>,
    /// The cells each of its PLMs writes, in the order of `plms`; `None` for a row in a form the
    /// harness doesn't read.
    pub(super) cells: Option<Vec<Vec<Offset>>>,
}

/// The table's lines, a row each.
fn table() -> impl Iterator<Item = &'static str> {
    NOTES
        .split_once(HEADING)
        .map_or("", |(_, after)| after)
        .lines()
        .skip_while(|line| !line.starts_with('|'))
        .take_while(|line| line.starts_with('|'))
        // The header and its rule.
        .skip(2)
}

/// The table's rows.
pub(super) fn rows() -> Vec<Row> {
    table().filter_map(row).collect()
}

/// A row of the table, from its columns: part, PLMs, what, cells written, rows.
fn row(line: &str) -> Option<Row> {
    let mut columns = line.split('|').map(str::trim).skip(1);
    let (_part, plms, _what, cells) = (
        columns.next()?,
        columns.next()?,
        columns.next()?,
        columns.next()?,
    );
    let plms = ids(plms);
    let forms = forms(cells);
    let cells = match forms.as_slice() {
        [one] => form(one).map(|cells| vec![cells; plms.len()]),
        each if each.len() == plms.len() => each.iter().map(|text| form(text)).collect(),
        _ => None,
    };
    Some(Row { plms, cells })
}

/// The writers the harness masks by: the rows it reads.
pub(super) struct Writers(Vec<(RangeInclusive<u16>, Vec<Offset>)>);

impl Writers {
    /// The table's writers, every row in a form the harness reads.
    pub(super) fn from_notes() -> Self {
        Self(
            rows()
                .into_iter()
                .filter_map(|row| Some(row.plms.into_iter().zip(row.cells?)))
                .flatten()
                .collect(),
        )
    }

    /// The cells PLM `plm` on `at` writes: its row's, or its own cell only, with the cells past
    /// the room's top or left edge left out.
    pub(super) fn cells(&self, plm: u16, at: Cell) -> impl Iterator<Item = Cell> {
        const OWN: &[Offset] = &[Pair { x: 0, y: 0 }];
        let reach = self
            .0
            .iter()
            .find_map(|(ids, cells)| ids.contains(&plm).then_some(cells.as_slice()))
            .unwrap_or(OWN);
        reach.iter().filter_map(move |offset| {
            Some(Cell {
                x: at.x.checked_add_signed(offset.x)?,
                y: at.y.checked_add_signed(offset.y)?,
            })
        })
    }
}

/// The PLM header ids a column names: each `` `$XXXX` ``, and each `` `$a`..`$b` `` as the run
/// from a to b.
fn ids(column: &str) -> Vec<RangeInclusive<u16>> {
    let mut ids = Vec::new();
    let mut rest = column;
    while let Some((_, after)) = rest.split_once("`$") {
        let Some((from, after)) = hex(after) else {
            rest = after;
            continue;
        };
        let (to, after) = after
            .strip_prefix("..`$")
            .and_then(hex)
            .unwrap_or((from, after));
        ids.push(from..=to);
        rest = after;
    }
    ids
}

/// A header id, four hex digits and the closing backtick, and what follows it.
fn hex(text: &str) -> Option<(u16, &str)> {
    let (digits, rest) = text.split_once('`')?;
    let id = u16::from_str_radix(digits, 16).ok()?;
    (digits.len() == 4).then_some((id, rest))
}

/// A column's forms: split at each `;` outside a note's parentheses.
fn forms(column: &str) -> Vec<&str> {
    let mut forms = Vec::new();
    let mut depth = 0_u32;
    let mut start = 0;
    for (at, character) in column.char_indices() {
        match character {
            '(' => depth = depth.saturating_add(1),
            ')' => depth = depth.saturating_sub(1),
            ';' if depth == 0 => {
                forms.extend(column.get(start..at).map(str::trim));
                start = at.saturating_add(1);
            }
            _ => {}
        }
    }
    forms.extend(column.get(start..).map(str::trim));
    forms
}

/// The cells of a form: its items joined by `, ` or `, and `, then nothing or a note.
fn form(text: &str) -> Option<Vec<Offset>> {
    let mut cells = Vec::new();
    let mut rest = item(text, &mut cells)?;
    loop {
        if rest.is_empty() || rest.starts_with(':') || rest.starts_with(" (") {
            return Some(cells);
        }
        let next = rest
            .strip_prefix(", and ")
            .or_else(|| rest.strip_prefix(", "))?;
        rest = item(next, &mut cells)?;
    }
}

/// One item, its cells added to `cells`: a point `(dx, dy)`, or a box `x a..b, y c..d` or
/// `x a..b at y c..d`. Returns what follows it.
fn item<'a>(text: &'a str, cells: &mut Vec<Offset>) -> Option<&'a str> {
    if let Some(rest) = text.strip_prefix('(') {
        let (x, rest) = number(rest)?;
        let (y, rest) = number(rest.strip_prefix(", ")?)?;
        cells.push(Pair { x, y });
        return rest.strip_prefix(')');
    }
    let (across, rest) = span(text.strip_prefix("x ")?)?;
    let rest = rest
        .strip_prefix(", y ")
        .or_else(|| rest.strip_prefix(" at y "))?;
    let (down, rest) = span(rest)?;
    cells.extend(down.flat_map(|y| across.clone().map(move |x| Pair { x, y })));
    Some(rest)
}

/// A span `a` or `a..b`, and what follows it.
fn span(text: &str) -> Option<(RangeInclusive<i16>, &str)> {
    let (from, rest) = number(text)?;
    let Some(rest) = rest.strip_prefix("..") else {
        return Some((from..=from, rest));
    };
    let (to, rest) = number(rest)?;
    Some((from..=to, rest))
}

/// A whole number, its minus sign `−` or `-`, and what follows it.
fn number(text: &str) -> Option<(i16, &str)> {
    let (negative, rest) = text
        .strip_prefix('−')
        .or_else(|| text.strip_prefix('-'))
        .map_or((false, text), |rest| (true, rest));
    let end = rest
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(rest.len());
    let magnitude: i16 = rest.get(..end)?.parse().ok()?;
    let value = if negative {
        magnitude.checked_neg()?
    } else {
        magnitude
    };
    Some((value, rest.get(end..)?))
}

#[cfg(test)]
mod tests {
    use super::{NOTES, Writers, row, rows, table};
    use clean_types::Cell;

    /// Every writer a cached row sees write past its own cell is in a form the harness reads, so
    /// none of today's rows meets a writer masked by its own cell only.
    #[test]
    fn every_writer_the_cached_rows_see_write_is_read() {
        assert!(rows().len() > 40, "the table has {} rows", rows().len());
        let writes = table().filter(|line| {
            line.split('|')
                .nth(5)
                .is_some_and(|rows| rows.trim().starts_with("writes:"))
        });
        for line in writes {
            assert!(row(line).is_some_and(|row| row.cells.is_some()), "{line}");
        }
    }

    /// A row the harness doesn't read is one the notes list under "What the harness needs for
    /// them", by its PLMs.
    #[test]
    fn every_row_left_out_is_one_the_notes_say_fails_loudly() {
        let needs = NOTES
            .split_once("**What the harness needs for them.**")
            .and_then(|(_, after)| after.split_once("\n## "))
            .map_or("", |(needs, _)| needs);
        for row in rows().iter().filter(|row| row.cells.is_none()) {
            for plms in &row.plms {
                let named = format!("`${:04X}`", plms.start());
                assert!(needs.contains(&named), "{named}");
            }
        }
    }

    /// The forms read as the table gives them: a column, a list of points and a row, and a
    /// box reaching left and up.
    #[test]
    fn the_forms_give_the_cells_the_table_names() {
        let writers = Writers::from_notes();
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
        assert_eq!(cells(0xB8EB), [(10, 10)]);
    }
}
