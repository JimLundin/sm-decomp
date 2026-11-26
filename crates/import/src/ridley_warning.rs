//! Import the Ceres warning's pictures without exposing transfer descriptors to the engine.

use crate::tileset::{cell_of, sheet};
use data::assets::art::{Colour, TileRef};
use data::assets::ridley_warning::{
    Art, ForegroundArt, ForegroundPatch, Letter, Line, Place, SheetPatch,
};
use data::rom::{AddressError, Lorom, RomAddress};

const BANK: u8 = 0xa6;
const SPRITE_LIST: u16 = 0xc4cb;
const FOREGROUND_LIST: u16 = 0xc4fe;
const HEADING: u16 = 0xc15d;
const SUBTITLE_LIST: u16 = 0xc3b8;
const GLOW: u16 = 0xc1df;
const ENGLISH: u16 = 0xc450;
const TRANSFER_BYTES: u16 = 7;
const ENGLISH_TIMING: u16 = 1;
const ENGLISH_PLACE: u16 = 13;
const FOREGROUND_MAP: u16 = 0x5000;
const SPRITE_BASE: u16 = 0x6000;
const FOREGROUND_BASE: u16 = 0x6000;
const VRAM_WORDS: u16 = 0x8000;
const TILE_WORDS: u16 = 16;
const MAP_WIDTH: u16 = 32;

const SPRITES: [(u16, u16); 7] = [
    (0x7e00, 0x200),
    (0x7f00, 0x120),
    (0x7820, 0x200),
    (0x7920, 0x200),
    (0x7a20, 0x200),
    (0x7b20, 0x200),
    (0x7c20, 0x100),
];
const FOREGROUND: [(u16, u16); 8] = [
    (0x1820, 0x200),
    (0x1920, 0x200),
    (0x1a20, 0x200),
    (0x1b20, 0x200),
    (0x1c20, 0x100),
    (0x0d00, 0x200),
    (0x0e00, 0x200),
    (0x0f00, 0x200),
];
const SUBTITLE: [(u16, u16); 4] = [(0x528a, 24), (0x52aa, 24), (0x52ca, 22), (0x52ea, 22)];
const ENGLISH_ROWS: [(u16, u16); 3] = [(0x5105, 22), (0x5145, 18), (0x5185, 18)];

/// Why the Ceres warning pictures cannot be admitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// A descriptor or its source data does not fit the image.
    Address(AddressError),
    /// A transfer is outside the warning's expected destination or is not tile-aligned.
    Transfer,
    /// A heading or subtitle strip has invalid placement, colours or priority.
    Caption,
    /// The English slots, positions or timing differ from the supported text format.
    EnglishText,
}

impl From<AddressError> for ImportError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// Read the artwork and validate each transfer against the destination sheet or map.
///
/// # Errors
/// Returns an error when a transfer, tile, caption or text slot is malformed.
pub fn import(rom: Lorom<'_>) -> Result<Art, ImportError> {
    let sprites = transfer_list(rom, SPRITE_LIST, SPRITES, |dest, bytes| {
        patch(dest, bytes, SPRITE_BASE, 512)
    })?;
    let foreground = transfer_list(rom, FOREGROUND_LIST, FOREGROUND, |dest, bytes| {
        let art = if dest < 0x1000 {
            ForegroundArt::Doors
        } else {
            ForegroundArt::Lettering
        };
        Ok(ForegroundPatch {
            art,
            patch: patch(dest, bytes, FOREGROUND_BASE, 1024)?,
        })
    })?;
    if !sprites
        .iter()
        .skip(2)
        .zip(&foreground)
        .all(|(sprite, front)| sprite.tiles == front.patch.tiles)
    {
        return Err(ImportError::Transfer);
    }
    let heading = transfer(rom, HEADING, (0x50cb, 18), |dest, bytes| {
        caption(dest, bytes, 6)
    })?;
    let subtitle = transfer_list(rom, SUBTITLE_LIST, SUBTITLE, |dest, bytes| {
        caption(dest, bytes, 7)
    })?;
    let english = english(rom)?;
    let cells = heading
        .cells
        .iter()
        .copied()
        .chain(subtitle.iter().flat_map(|row| row.cells.iter().copied()))
        .chain(english.iter().filter_map(|slot| slot.glyph));
    if !cells.into_iter().all(|cell| {
        // Sprites and the foreground share the room's initial tile base.
        sprites
            .iter()
            .chain(foreground.iter().map(|batch| &batch.patch))
            .any(|batch| {
                cell.tile
                    .0
                    .checked_sub(batch.first.0)
                    .is_some_and(|offset| usize::from(offset) < batch.tiles.0.len())
            })
    }) {
        return Err(ImportError::Caption);
    }
    let mut glow = [[Colour(0); 3]; 16];
    let source = bank(rom, GLOW, 96)?;
    for (step, bytes) in glow.iter_mut().zip(source.as_chunks::<6>().0) {
        for (colour, &word) in step.iter_mut().zip(bytes.as_chunks::<2>().0) {
            *colour = Colour(u16::from_le_bytes(word));
        }
    }
    Ok(Art {
        sprites,
        foreground,
        heading,
        subtitle,
        english,
        glow,
    })
}

fn bank(rom: Lorom<'_>, offset: u16, bytes: usize) -> Result<&[u8], AddressError> {
    rom.read(RomAddress::new(BANK, offset), bytes)
}

fn word(rom: Lorom<'_>, offset: u16) -> Result<u16, ImportError> {
    let [low, high] = bank(rom, offset, 2)? else {
        return Err(ImportError::Transfer);
    };
    Ok(u16::from_le_bytes([*low, *high]))
}

fn transfer_list<T, const N: usize>(
    rom: Lorom<'_>,
    mut offset: u16,
    shape: [(u16, u16); N],
    convert: impl Fn(u16, &[u8]) -> Result<T, ImportError>,
) -> Result<[T; N], ImportError> {
    let values = shape
        .into_iter()
        .map(|expected| {
            let value = transfer(rom, offset, expected, &convert)?;
            offset = offset.wrapping_add(TRANSFER_BYTES);
            Ok(value)
        })
        .collect::<Result<Vec<_>, ImportError>>()?;
    if word(rom, offset)? != 0 {
        return Err(ImportError::Transfer);
    }
    values.try_into().map_err(|_| ImportError::Transfer)
}

fn transfer<T>(
    rom: Lorom<'_>,
    offset: u16,
    shape: (u16, u16),
    convert: impl FnOnce(u16, &[u8]) -> Result<T, ImportError>,
) -> Result<T, ImportError> {
    let [
        size_lo,
        size_hi,
        source_lo,
        source_hi,
        page,
        dest_lo,
        dest_hi,
    ] = bank(rom, offset, usize::from(TRANSFER_BYTES))?
    else {
        return Err(ImportError::Transfer);
    };
    let size = u16::from_le_bytes([*size_lo, *size_hi]);
    let dest = u16::from_le_bytes([*dest_lo, *dest_hi]);
    if (dest, size) != shape {
        return Err(ImportError::Transfer);
    }
    let source = RomAddress::new(*page, u16::from_le_bytes([*source_lo, *source_hi]));
    convert(dest, rom.read(source, usize::from(size))?)
}

fn patch(dest: u16, bytes: &[u8], base: u16, capacity: u16) -> Result<SheetPatch, ImportError> {
    // Foreground tile addresses wrap from the room's base across the end of VRAM.
    let words = dest.wrapping_sub(base) & VRAM_WORDS.wrapping_sub(1);
    if !words.is_multiple_of(TILE_WORDS) {
        return Err(ImportError::Transfer);
    }
    let first = words / TILE_WORDS;
    let tiles = sheet(bytes).ok_or(ImportError::Transfer)?;
    if tiles.0.len() > usize::from(capacity.saturating_sub(first)) {
        return Err(ImportError::Transfer);
    }
    Ok(SheetPatch {
        first: TileRef(first),
        tiles,
    })
}

fn place(dest: u16) -> Option<Place> {
    let offset = dest.checked_sub(FOREGROUND_MAP)?;
    (offset < MAP_WIDTH * MAP_WIDTH).then(|| {
        let [column, _] = (offset % MAP_WIDTH).to_le_bytes();
        let [row, _] = (offset / MAP_WIDTH).to_le_bytes();
        Place { column, row }
    })
}

fn caption(dest: u16, bytes: &[u8], palette: u8) -> Result<Line, ImportError> {
    let first = place(dest).ok_or(ImportError::Caption)?;
    let (words, tail) = bytes.as_chunks::<2>();
    if !tail.is_empty()
        || words.len() > usize::from(MAP_WIDTH.saturating_sub(u16::from(first.column)))
    {
        return Err(ImportError::Caption);
    }
    let cells: Vec<_> = words
        .iter()
        .map(|&word| cell_of(u16::from_le_bytes(word)))
        .collect();
    if !cells
        .iter()
        .all(|cell| cell.priority && cell.palette.0 == palette)
    {
        return Err(ImportError::Caption);
    }
    Ok(Line { first, cells })
}

fn english(rom: Lorom<'_>) -> Result<Vec<Letter>, ImportError> {
    if word(rom, ENGLISH)? != ENGLISH_TIMING || word(rom, ENGLISH.wrapping_add(2))? != 2 {
        return Err(ImportError::EnglishText);
    }
    let mut cursor = ENGLISH.wrapping_add(4);
    let mut slots = Vec::new();
    for (dest, count) in ENGLISH_ROWS {
        if word(rom, cursor)? != ENGLISH_PLACE || word(rom, cursor.wrapping_add(2))? != dest {
            return Err(ImportError::EnglishText);
        }
        cursor = cursor.wrapping_add(4);
        for (column, &letter) in (0..count).zip(bank(rom, cursor, usize::from(count))?) {
            let glyph = match letter {
                b' ' => None,
                b'A'..=b'Z' => Some(cell_of(
                    0x3582u16.wrapping_add(u16::from(letter.wrapping_sub(b'A'))),
                )),
                _ => return Err(ImportError::EnglishText),
            };
            slots.push(Letter {
                place: place(dest.wrapping_add(column)).ok_or(ImportError::EnglishText)?,
                glyph,
                rest: if slots.is_empty() { 0 } else { 2 },
            });
        }
        cursor = cursor.wrapping_add(count);
    }
    if word(rom, cursor)? != 0 {
        return Err(ImportError::EnglishText);
    }
    Ok(slots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_checks_alignment_capacity_and_complete_tiles() {
        let bytes = [0; 32];
        assert_eq!(
            patch(0x1820, &bytes, FOREGROUND_BASE, 1024).unwrap().first,
            TileRef(898)
        );
        assert_eq!(
            patch(0x1821, &bytes, FOREGROUND_BASE, 1024),
            Err(ImportError::Transfer)
        );
        assert_eq!(
            patch(0x1820, &bytes[..31], FOREGROUND_BASE, 1024),
            Err(ImportError::Transfer)
        );
        assert_eq!(
            patch(0x1820, &bytes, FOREGROUND_BASE, 898),
            Err(ImportError::Transfer)
        );
    }

    #[test]
    fn checked_warning_imports_from_the_cartridge() {
        let Some(path) = std::env::var_os("SM_ROM") else {
            return;
        };
        let rom = std::fs::read(path).unwrap();
        let art = import(Lorom::new(&rom)).unwrap();
        assert_eq!(
            art.sprites
                .iter()
                .map(|batch| batch.tiles.0.len())
                .sum::<usize>(),
            97
        );
        assert_eq!(art.foreground.len(), 8);
        assert_eq!(art.english.len(), 58);
        assert_eq!(
            art.english
                .iter()
                .filter(|slot| slot.glyph.is_some())
                .count(),
            54
        );
        assert_eq!(art.english[0].rest, 0);
        assert!(art.english[1..].iter().all(|slot| slot.rest == 2));

        let mut malformed = rom;
        let dest = Lorom::new(&malformed)
            .file_offset(RomAddress::new(BANK, SPRITE_LIST.wrapping_add(5)))
            .unwrap();
        malformed[dest] ^= 1;
        assert_eq!(import(Lorom::new(&malformed)), Err(ImportError::Transfer));
    }
}
