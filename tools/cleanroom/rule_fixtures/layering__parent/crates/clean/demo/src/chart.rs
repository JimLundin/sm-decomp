//! The chart reads its table.
mod table;

/// A key.
pub enum Key { Up }

/// The first row.
pub fn first() -> &'static [Key] { table::ROWS }
