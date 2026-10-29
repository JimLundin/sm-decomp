//! `sm_data` records imported from a cartridge image.
#![forbid(unsafe_code)]
use crate::rom::{RomImage, lorom_offset};

/// A record and the allocation it was imported from.
pub(crate) struct Imported<T> {
    image: RomImage,
    record: T,
}

impl<T> Imported<T> {
    /// Import `bytes` bytes at LoROM `address`; `None` for images too short to
    /// hold them or holding content the importer rejects.
    pub(crate) fn import<E>(
        image: &RomImage,
        address: u32,
        bytes: usize,
        import: impl FnOnce(&[u8]) -> Result<T, E>,
    ) -> Option<Self> {
        let start = lorom_offset(address) as usize;
        let bytes = (start..start + bytes)
            .map(|at| (at < image.len()).then(|| image.read(at)))
            .collect::<Option<Vec<u8>>>()?;
        Some(Self {
            image: image.clone(),
            record: import(&bytes).ok()?,
        })
    }

    /// The record while `rom` is still the allocation it came from; callers
    /// keep the original ROM read otherwise.
    pub(crate) fn from<'a>(imported: &'a Option<Self>, rom: &RomImage) -> Option<&'a T> {
        imported
            .as_ref()
            .filter(|imported| imported.image.same_allocation(rom))
            .map(|imported| &imported.record)
    }
}
