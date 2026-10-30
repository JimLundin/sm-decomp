//! Fixed-size tables of little-endian word rows.

/// The importer was given a byte count other than the table's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LengthError(pub usize);

/// `ROWS` records, each built from one row of little-endian words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rows<T, const ROWS: usize>(Box<[T; ROWS]>);

impl<T: Copy, const ROWS: usize> Rows<T, ROWS> {
    pub(crate) fn import<const WORDS: usize>(
        bytes: &[u8],
        row: impl Fn([u16; WORDS]) -> T,
    ) -> Result<Self, LengthError> {
        Self::import_with(bytes, WORDS, |word, index| {
            row(core::array::from_fn(|column| word(WORDS * index + column)))
        })
    }

    /// Rows whose fields are stored as consecutive lists of `ROWS` words each.
    pub(crate) fn import_columns<const COLUMNS: usize>(
        bytes: &[u8],
        row: impl Fn([u16; COLUMNS]) -> T,
    ) -> Result<Self, LengthError> {
        Self::import_with(bytes, COLUMNS, |word, index| {
            row(core::array::from_fn(|column| word(ROWS * column + index)))
        })
    }

    fn import_with(
        bytes: &[u8],
        words: usize,
        row: impl Fn(&dyn Fn(usize) -> u16, usize) -> T,
    ) -> Result<Self, LengthError> {
        if bytes.len() != ROWS * words * 2 {
            return Err(LengthError(bytes.len()));
        }
        let word = |at: usize| u16::from_le_bytes([bytes[2 * at], bytes[2 * at + 1]]);
        Ok(Self(Box::new(core::array::from_fn(|index| {
            row(&word, index)
        }))))
    }

    /// Row `index`, or `None` past the table.
    pub fn row(&self, index: usize) -> Option<T> {
        self.0.get(index).copied()
    }
}

impl<const WORDS: usize, const ROWS: usize> Rows<[u16; WORDS], ROWS> {
    /// The word at `index` counted across rows, or `None` past the table.
    pub fn word(&self, index: usize) -> Option<u16> {
        self.row(index / WORDS).map(|row| row[index % WORDS])
    }
}

/// `ROWS` words, such as a list of bank-local pointers.
pub fn words<const ROWS: usize>(bytes: &[u8]) -> Result<Rows<u16, ROWS>, LengthError> {
    Rows::import(bytes, |[word]| word)
}

/// `ROWS` rows of `WORDS` words, such as colour runs.
pub fn grid<const WORDS: usize, const ROWS: usize>(
    bytes: &[u8],
) -> Result<Rows<[u16; WORDS], ROWS>, LengthError> {
    Rows::import(bytes, |row| row)
}

/// Panics when `index` selects no entry of a `len`-entry table, with the same
/// message as an out-of-range slice index. Jump tables use it to reject an
/// invalid selector before dispatching, as the original's checked tables did.
#[track_caller]
pub fn check_index(index: usize, len: usize) {
    if index >= len {
        panic!("index out of bounds: the len is {len} but the index is {index}");
    }
}
