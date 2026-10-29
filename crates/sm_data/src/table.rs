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
        if bytes.len() != ROWS * WORDS * 2 {
            return Err(LengthError(bytes.len()));
        }
        let word = |at: usize| u16::from_le_bytes([bytes[2 * at], bytes[2 * at + 1]]);
        Ok(Self(Box::new(core::array::from_fn(|index| {
            row(core::array::from_fn(|column| word(WORDS * index + column)))
        }))))
    }

    /// Row `index`, or `None` past the table.
    pub fn row(&self, index: usize) -> Option<T> {
        self.0.get(index).copied()
    }
}
