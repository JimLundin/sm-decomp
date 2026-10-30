//! `sm_data` records imported from a cartridge image.
use crate::cartridge::LoadedImage;
use sm_data::cartridge::{RomImage, lorom_offset};
use sm_data::table::{Rows, grid, words};

/// A record and the allocation it was imported from.
pub struct Imported<T> {
    image: RomImage,
    /// `image.writes()` at import: any later write, such as a bug-fix patch,
    /// retires the record until the image is imported again.
    writes: u64,
    record: T,
}

impl<T> Imported<T> {
    /// Import `bytes` bytes at LoROM `address`; `None` for images too short to
    /// hold them or holding content the importer rejects.
    pub fn import<E>(
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
            writes: image.writes(),
            record: import(&bytes).ok()?,
        })
    }

    /// The record while `rom` is still the allocation it came from, unwritten
    /// since the import; callers keep the original ROM read otherwise.
    pub fn from<'a>(imported: &'a Option<Self>, rom: &RomImage) -> Option<&'a T> {
        imported
            .as_ref()
            .filter(|imported| {
                imported.image.same_allocation(rom) && imported.writes == rom.writes()
            })
            .map(|imported| &imported.record)
    }

    /// `get`'s value from the record while the loaded image is its allocation;
    /// otherwise, or when `get` finds nothing, `load` runs the original reads.
    pub fn read<R>(
        imported: &Option<Self>,
        game: &(impl LoadedImage + ?Sized),
        get: impl FnOnce(&T) -> Option<R>,
        load: impl FnOnce(&RomImage) -> R,
    ) -> R {
        let rom = game.loaded_image();
        Self::from(imported, rom)
            .and_then(get)
            .unwrap_or_else(|| load(rom))
    }
}

/// The original indexed load: the word at signed word `index` from `address`.
pub fn load(rom: &RomImage, address: u32, index: isize) -> u16 {
    rom.table_word(lorom_offset(address) as usize, index)
}

/// Word `index` of a one-word-per-row table while the loaded image is its
/// allocation; other images and indices past the table run the original
/// indexed load at `address`.
pub fn word<const N: usize>(
    table: &Option<Imported<sm_data::table::Rows<u16, N>>>,
    game: &(impl LoadedImage + ?Sized),
    address: u32,
    index: isize,
) -> u16 {
    Imported::read(
        table,
        game,
        |rows| rows.row(usize::try_from(index).ok()?),
        |rom| load(rom, address, index),
    )
}

/// Word `index` of a table of `W`-word rows read as one word list: row
/// `index / W`, column `index % W`. Other images and indices past the table
/// run the original indexed load at `address`.
pub fn grid_word<const W: usize, const N: usize>(
    table: &Option<Imported<sm_data::table::Rows<[u16; W], N>>>,
    game: &(impl LoadedImage + ?Sized),
    address: u32,
    index: isize,
) -> u16 {
    Imported::read(
        table,
        game,
        |rows| {
            let index = usize::try_from(index).ok()?;
            Some(rows.row(index / W)?[index % W])
        },
        |rom| load(rom, address, index),
    )
}

/// A word index as a row of `words`-word rows, if it starts one.
pub fn row_of(index: isize, words: usize) -> Option<usize> {
    let index = usize::try_from(index).ok()?;
    (index % words == 0).then_some(index / words)
}

/// A word list at LoROM `ADDRESS`, read by word index.
#[derive(Default)]
pub struct ImportedWords<const ADDRESS: u32, const ROWS: usize>(Option<Imported<Rows<u16, ROWS>>>);

impl<const ADDRESS: u32, const ROWS: usize> ImportedWords<ADDRESS, ROWS> {
    pub fn import(image: &RomImage) -> Self {
        Self(Imported::import(image, ADDRESS, 2 * ROWS, words))
    }

    /// The word at `index` while `rom` is the imported allocation; any other
    /// image or index keeps the original indexed load with its failures.
    pub fn word(&self, rom: &RomImage, index: isize) -> u16 {
        usize::try_from(index)
            .ok()
            .and_then(|index| Imported::from(&self.0, rom)?.row(index))
            .unwrap_or_else(|| rom.table_word(lorom_offset(ADDRESS) as usize, index))
    }
}

/// Rows of `WORDS` words at LoROM `ADDRESS`, read by word index across rows.
#[derive(Default)]
pub struct ImportedGrid<const ADDRESS: u32, const WORDS: usize, const ROWS: usize>(
    Option<Imported<Rows<[u16; WORDS], ROWS>>>,
);

impl<const ADDRESS: u32, const WORDS: usize, const ROWS: usize> ImportedGrid<ADDRESS, WORDS, ROWS> {
    pub fn import(image: &RomImage) -> Self {
        Self(Imported::import(image, ADDRESS, 2 * WORDS * ROWS, grid))
    }

    /// As `ImportedWords::word`, counting `index` across rows.
    pub fn word(&self, rom: &RomImage, index: isize) -> u16 {
        usize::try_from(index)
            .ok()
            .and_then(|index| Imported::from(&self.0, rom)?.word(index))
            .unwrap_or_else(|| rom.table_word(lorom_offset(ADDRESS) as usize, index))
    }
}

/// The loaded cartridge image.
pub fn rom(game: &(impl LoadedImage + ?Sized)) -> &RomImage {
    game.loaded_image()
}

/// Row `index` of an imported table while `rom` is its allocation; `None` for
/// other images and for indices outside the table, which keep the original load.
pub fn row<T: Copy, const ROWS: usize>(
    table: &Option<Imported<sm_data::table::Rows<T, ROWS>>>,
    rom: &RomImage,
    index: isize,
) -> Option<T> {
    let index = usize::try_from(index).ok()?;
    Imported::from(table, rom)?.row(index)
}
