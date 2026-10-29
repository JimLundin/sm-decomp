//! `sm_data` records imported from a cartridge image.
#![forbid(unsafe_code)]
use crate::rom::{RomImage, lorom_offset};
use sm_data::table::{Rows, grid, words};

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

/// A word list at LoROM `ADDRESS`, read by word index.
#[derive(Default)]
pub(crate) struct ImportedWords<const ADDRESS: u32, const ROWS: usize>(
    Option<Imported<Rows<u16, ROWS>>>,
);

impl<const ADDRESS: u32, const ROWS: usize> ImportedWords<ADDRESS, ROWS> {
    pub(crate) fn import(image: &RomImage) -> Self {
        Self(Imported::import(image, ADDRESS, 2 * ROWS, words))
    }

    /// The word at `index` while `rom` is the imported allocation; any other
    /// image or index keeps the original indexed load with its failures.
    pub(crate) fn word(&self, rom: &RomImage, index: isize) -> u16 {
        usize::try_from(index)
            .ok()
            .and_then(|index| Imported::from(&self.0, rom)?.row(index))
            .unwrap_or_else(|| rom.table_word(lorom_offset(ADDRESS) as usize, index))
    }
}

/// Rows of `WORDS` words at LoROM `ADDRESS`, read by word index across rows.
#[derive(Default)]
pub(crate) struct ImportedGrid<const ADDRESS: u32, const WORDS: usize, const ROWS: usize>(
    Option<Imported<Rows<[u16; WORDS], ROWS>>>,
);

impl<const ADDRESS: u32, const WORDS: usize, const ROWS: usize> ImportedGrid<ADDRESS, WORDS, ROWS> {
    pub(crate) fn import(image: &RomImage) -> Self {
        Self(Imported::import(image, ADDRESS, 2 * WORDS * ROWS, grid))
    }

    /// As `ImportedWords::word`, counting `index` across rows.
    pub(crate) fn word(&self, rom: &RomImage, index: isize) -> u16 {
        usize::try_from(index)
            .ok()
            .and_then(|index| Imported::from(&self.0, rom)?.word(index))
            .unwrap_or_else(|| rom.table_word(lorom_offset(ADDRESS) as usize, index))
    }
}
