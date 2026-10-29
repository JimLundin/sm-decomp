//! Common enemy speed tables at $A2:8187 (linear) and $A2:838F (quadratic).
//!
//! Each 8-byte entry holds a 16.16 speed followed by its negation. Linear
//! entries store (speed, subspeed, -speed, -subspeed); quadratic entries store
//! (subspeed, speed, -subspeed, -speed). The two tables are contiguous, and
//! enemies index them with signed word or byte offsets that may cross from one
//! table into the other, so both are kept as one 1,280-byte block.

/// LoROM address of the first byte of the block.
pub const ADDRESS: u32 = 0xa2_8187;
/// Bytes from $A2:8187 up to $A2:8687, the end of the quadratic table.
pub const BYTES: usize = 0x500;
const QUADRATIC_START: usize = 0x208;
const ENTRY: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeedTable {
    Linear,
    Quadratic,
}

impl SpeedTable {
    pub const fn address(self) -> u32 {
        ADDRESS + self.start() as u32
    }
    const fn start(self) -> usize {
        match self {
            Self::Linear => 0,
            Self::Quadratic => QUADRATIC_START,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    Length(usize),
    /// An entry's second half is not the two's-complement negation of its first.
    Negation {
        table: SpeedTable,
        entry: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnemySpeeds {
    bytes: Box<[u8; BYTES]>,
}

impl EnemySpeeds {
    /// Build the block from the bytes at `ADDRESS`, rejecting anything that is
    /// not a pair of speed tables.
    pub fn import(bytes: &[u8]) -> Result<Self, ImportError> {
        let bytes: Box<[u8; BYTES]> = Box::new(
            bytes
                .try_into()
                .map_err(|_| ImportError::Length(bytes.len()))?,
        );
        for (index, entry) in bytes.chunks_exact(ENTRY).enumerate() {
            let w = |i: usize| u32::from(u16::from_le_bytes([entry[2 * i], entry[2 * i + 1]]));
            let (table, entry_index, speed, negated) = if index * ENTRY < QUADRATIC_START {
                (
                    SpeedTable::Linear,
                    index,
                    w(0) << 16 | w(1),
                    w(2) << 16 | w(3),
                )
            } else {
                let entry_index = index - QUADRATIC_START / ENTRY;
                (
                    SpeedTable::Quadratic,
                    entry_index,
                    w(1) << 16 | w(0),
                    w(3) << 16 | w(2),
                )
            };
            if speed.wrapping_add(negated) != 0 {
                return Err(ImportError::Negation {
                    table,
                    entry: entry_index,
                });
            }
        }
        Ok(Self { bytes })
    }

    /// The little-endian word at a signed word index from the table's start,
    /// or `None` when it lies outside the block.
    pub fn word(&self, table: SpeedTable, index: isize) -> Option<u16> {
        self.word_at(table, index.checked_mul(2)?)
    }

    /// The little-endian word at a signed byte offset from the table's start.
    /// Odd offsets straddle two words, as the original byte-indexed reads do.
    pub fn word_at(&self, table: SpeedTable, offset: isize) -> Option<u16> {
        let at = table.start().checked_add_signed(offset)?;
        let pair = self.bytes.get(at..at.checked_add(2)?)?;
        Some(u16::from_le_bytes([pair[0], pair[1]]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(table: SpeedTable, speed: u32) -> [u8; ENTRY] {
        let negated = speed.wrapping_neg();
        let words = match table {
            SpeedTable::Linear => [speed >> 16, speed, negated >> 16, negated],
            SpeedTable::Quadratic => [speed, speed >> 16, negated, negated >> 16],
        };
        let mut out = [0; ENTRY];
        for (i, word) in words.into_iter().enumerate() {
            out[2 * i..2 * i + 2].copy_from_slice(&(word as u16).to_le_bytes());
        }
        out
    }

    // Synthetic tables only: the repository never holds ROM-derived bytes.
    fn block() -> Vec<u8> {
        let linear = (0..65).flat_map(|i| entry(SpeedTable::Linear, i * 0x1000));
        let quadratic = (0..95).flat_map(|i| entry(SpeedTable::Quadratic, i * i * 0x109));
        linear.chain(quadratic).collect()
    }

    #[test]
    fn reads_words_from_both_tables() {
        let speeds = EnemySpeeds::import(&block()).unwrap();
        assert_eq!(SpeedTable::Quadratic.address(), 0xa2_838f);
        assert_eq!(speeds.word(SpeedTable::Linear, 4), Some(0x0000));
        assert_eq!(speeds.word(SpeedTable::Linear, 5), Some(0x1000));
        assert_eq!(speeds.word(SpeedTable::Linear, 6), Some(0xffff));
        assert_eq!(speeds.word(SpeedTable::Quadratic, 4), Some(0x0109));
        assert_eq!(speeds.word(SpeedTable::Quadratic, 6), Some(0xfef7));
    }

    #[test]
    fn offsets_cross_tables_and_straddle_words() {
        let speeds = EnemySpeeds::import(&block()).unwrap();
        let bytes = block();
        // Past the linear table's end is the start of the quadratic table.
        assert_eq!(
            speeds.word(SpeedTable::Linear, 0x104 + 5),
            speeds.word(SpeedTable::Quadratic, 5)
        );
        assert_eq!(
            speeds.word(SpeedTable::Quadratic, -1),
            speeds.word(SpeedTable::Linear, 0x103)
        );
        let odd = QUADRATIC_START + 8 * 3 + 1;
        assert_eq!(
            speeds.word_at(SpeedTable::Quadratic, 8 * 3 + 1),
            Some(u16::from_le_bytes([bytes[odd], bytes[odd + 1]]))
        );
    }

    #[test]
    fn rejects_reads_outside_the_block() {
        let speeds = EnemySpeeds::import(&block()).unwrap();
        assert_eq!(speeds.word(SpeedTable::Linear, -1), None);
        assert_eq!(speeds.word_at(SpeedTable::Linear, BYTES as isize - 1), None);
        let last = u16::from_le_bytes([block()[BYTES - 2], block()[BYTES - 1]]);
        assert_eq!(
            speeds.word_at(SpeedTable::Linear, BYTES as isize - 2),
            Some(last)
        );
        assert_eq!(speeds.word(SpeedTable::Quadratic, isize::MAX), None);
        assert_eq!(speeds.word_at(SpeedTable::Linear, isize::MIN), None);
    }

    #[test]
    fn rejects_other_content() {
        assert_eq!(
            EnemySpeeds::import(&block()[1..]),
            Err(ImportError::Length(BYTES - 1))
        );
        let mut bad = block();
        bad[QUADRATIC_START + 8 * 2 + 4] ^= 1;
        assert_eq!(
            EnemySpeeds::import(&bad),
            Err(ImportError::Negation {
                table: SpeedTable::Quadratic,
                entry: 2
            })
        );
    }
}
