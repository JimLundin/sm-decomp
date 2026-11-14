//! CSV acquisition never exposes captured row contents in a diagnostic.
use super::RoomSeed;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ErrorKind {
    MissingColumn(&'static str),
    MissingField,
    InvalidNumber,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct CsvError {
    pub line: usize,
    pub column: usize,
    pub kind: ErrorKind,
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CSV line {}, column {}: {:?}",
            self.line, self.column, self.kind
        )
    }
}

pub(super) fn room_seeds(text: &str) -> Result<Vec<RoomSeed>, CsvError> {
    let mut lines = text.lines();
    let header: Vec<_> = lines.next().unwrap_or_default().split(',').collect();
    let column = |name: &'static str| {
        header
            .iter()
            .position(|&field| field == name)
            .ok_or(CsvError {
                line: 1,
                column: header.len() + 1,
                kind: ErrorKind::MissingColumn(name),
            })
    };
    let (index, room, frame) = (
        column("room_index")?,
        column("room")?,
        column("movie_frame")?,
    );
    lines
        .enumerate()
        .filter(|(_, line)| !line.is_empty())
        .map(|(offset, line)| {
            let fields: Vec<_> = line.split(',').collect();
            let field = |column| {
                fields.get(column).copied().ok_or(CsvError {
                    line: offset + 2,
                    column: column + 1,
                    kind: ErrorKind::MissingField,
                })
            };
            let invalid = |column| CsvError {
                line: offset + 2,
                column: column + 1,
                kind: ErrorKind::InvalidNumber,
            };
            Ok(RoomSeed {
                index: field(index)?.parse().map_err(|_| invalid(index))?,
                room: u16::from_str_radix(field(room)?, 16).map_err(|_| invalid(room))?,
                movie_frame: field(frame)?.parse().map_err(|_| invalid(frame))?,
            })
        })
        .collect()
}

pub(super) fn lag_log(text: &str) -> Result<Vec<[i64; 7]>, CsvError> {
    text.lines()
        .skip(1)
        .enumerate()
        .map(|(offset, line)| {
            let fields: Vec<_> = line.split(',').collect();
            let mut row = [0; 7];
            for (value, column) in row.iter_mut().zip([3, 5, 6, 7, 8, 9, 10]) {
                let field = fields.get(column).ok_or(CsvError {
                    line: offset + 2,
                    column: column + 1,
                    kind: ErrorKind::MissingField,
                })?;
                // Non-numeric fields retain the lag log's unknown-value sentinel.
                *value = field.parse().unwrap_or(-1);
            }
            Ok(row)
        })
        .collect()
}
