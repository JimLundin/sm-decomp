//! A copy of the cartridge prepared once for the script interpreters, so their
//! cursors read prepared content rather than the cartridge image.
use crate::cartridge::RomImage;

/// The bytes of `source` when it was prepared.
pub struct ProgramImage {
    bytes: Box<[u8]>,
    source: RomImage,
}

impl ProgramImage {
    pub fn prepare(source: &RomImage) -> Self {
        Self {
            bytes: source.bytes().into(),
            source: source.clone(),
        }
    }

    /// Whether this copy was prepared from `image`.
    pub fn serves(&self, image: &RomImage) -> bool {
        self.source.same_allocation(image)
    }

    /// The image it was prepared from.
    pub fn source(&self) -> &RomImage {
        &self.source
    }

    /// The byte at `offset`, with the image's bounds failure.
    pub fn read(&self, offset: usize) -> u8 {
        self.bytes[offset]
    }
}
