//! An immutable copy of the cartridge prepared once for the script
//! interpreters, so their cursors read prepared content rather than the live
//! cartridge image.
use crate::cartridge::RomImage;

/// The bytes of `source` when it was prepared.
pub struct ProgramImage {
    bytes: Box<[u8]>,
    source: RomImage,
    /// `source.writes()` when prepared: any later write, such as a bug-fix
    /// patch, retires the copy and reads go to the live image again.
    writes: u64,
}

impl ProgramImage {
    pub fn prepare(source: &RomImage) -> Self {
        Self {
            bytes: (0..source.len()).map(|at| source.read(at)).collect(),
            source: source.clone(),
            writes: source.writes(),
        }
    }

    /// Whether this copy still matches `image`.
    pub fn serves(&self, image: &RomImage) -> bool {
        self.source.same_allocation(image) && self.source.writes() == self.writes
    }

    /// The image it was prepared from.
    pub fn source(&self) -> &RomImage {
        &self.source
    }

    /// The byte at `offset`: the prepared copy while the source is unwritten,
    /// otherwise the live image with its bounds failure.
    pub fn read(&self, offset: usize) -> u8 {
        match self.bytes.get(offset) {
            Some(&byte) if self.source.writes() == self.writes => byte,
            _ => self.source.read(offset),
        }
    }
}
