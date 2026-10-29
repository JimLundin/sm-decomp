//! Area select map foreground palette table at $81:A4E6.
//!
//! Each area has a run of (source, destination) copies for its active and
//! inactive look, ended by a $FFFF word. A copy moves five colours from the
//! foreground palette data at $81:A40E into the palette buffer; both are byte
//! offsets. The twelve runs follow each other in area order, active first.

/// LoROM address of the first run.
pub const ADDRESS: u32 = 0x81_a4e6;
/// Bytes up to the last run's end marker.
pub const BYTES: usize = 0x60;
pub const AREAS: usize = 6;
/// Byte offset of each area's active run; its inactive run is 0x30 further.
pub const RUN_OFFSETS: [usize; AREAS] = [0, 0xa, 0x10, 0x16, 0x24, 0x2a];
pub const INACTIVE_OFFSET: usize = 0x30;
const END: u16 = 0xffff;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    Length(usize),
    /// A run does not start at its fixed offset or runs past the table.
    Layout {
        offset: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaletteCopy {
    /// Byte offset into the foreground palette data at $81:A40E.
    pub source: u16,
    /// Byte offset into the palette buffer.
    pub destination: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AreaSelectPalettes {
    /// Active runs for areas 0-5, then inactive runs.
    runs: Vec<Vec<PaletteCopy>>,
}

impl AreaSelectPalettes {
    /// Rejects a table whose runs do not start at `RUN_OFFSETS` and end at `BYTES`.
    pub fn import(bytes: &[u8]) -> Result<Self, ImportError> {
        if bytes.len() != BYTES {
            return Err(ImportError::Length(bytes.len()));
        }
        let word = |at: usize| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        let starts = RUN_OFFSETS
            .iter()
            .copied()
            .chain(RUN_OFFSETS.iter().map(|offset| offset + INACTIVE_OFFSET));
        let mut runs = Vec::new();
        let mut offset = 0;
        for start in starts {
            if offset != start {
                return Err(ImportError::Layout { offset });
            }
            let mut run = Vec::new();
            loop {
                if offset + 2 > BYTES {
                    return Err(ImportError::Layout { offset });
                }
                if word(offset) == END {
                    offset += 2;
                    break;
                }
                if offset + 4 > BYTES {
                    return Err(ImportError::Layout { offset });
                }
                run.push(PaletteCopy {
                    source: word(offset),
                    destination: word(offset + 2),
                });
                offset += 4;
            }
            runs.push(run);
        }
        if offset != BYTES {
            return Err(ImportError::Layout { offset });
        }
        Ok(Self { runs })
    }

    pub fn copies(&self, area: usize, active: bool) -> Option<&[PaletteCopy]> {
        let run = if active { area } else { AREAS + area };
        (area < AREAS).then(|| self.runs[run].as_slice())
    }
}
