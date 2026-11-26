//! Publish the warning's imported tiles and cells in Ceres's native room layout.

use crate::exact::art::planar_4bpp;
use crate::state::video::{TilemapEntry, VRAM_WORDS};
use data::assets::art::Cell;
use data::assets::ridley_warning::{Letter, Line, Place, SheetPatch};
use std::ops::Range;

const TILE_WORDS: usize = 16;
const TILE_BASE: usize = 0x6000;
const MAP_BASE: usize = 0x5000;
const MAP_SIDE: usize = 32;

/// Write a patch to the shared sprite and foreground tile memory.
///
/// Returns the written word span. Returns `None` without writing if the
/// complete span does not fit.
#[must_use]
pub fn sheet(vram: &mut [u16], patch: &SheetPatch) -> Option<Range<usize>> {
    let first = usize::from(patch.first.0);
    if first >= 1024 || patch.tiles.0.is_empty() {
        return None;
    }
    // Both sheets start at the same word, and foreground tile addresses wrap.
    let start = TILE_BASE.checked_add(first.checked_mul(TILE_WORDS)?)? % VRAM_WORDS;
    let end = start.checked_add(patch.tiles.0.len().checked_mul(TILE_WORDS)?)?;
    let span = start..end;
    let words = vram.get_mut(span.clone())?;
    let bytes = planar_4bpp(&patch.tiles);
    for (word, &source) in words.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *word = u16::from_le_bytes(source);
    }
    Some(span)
}

/// Write a complete heading or subtitle strip to the foreground map.
///
/// Returns the written word span. Returns `None` without writing if the
/// strip or target does not fit one row.
#[must_use]
pub fn line(vram: &mut [u16], line: &Line) -> Option<Range<usize>> {
    cells(vram, line.first, &line.cells)
}

/// Write an English glyph in its foreground cell. A space leaves the cell in place.
///
/// Returns the addressed cell span for either slot. Returns `None` without
/// writing when the position does not fit the map.
#[must_use]
pub fn letter(vram: &mut [u16], letter: &Letter) -> Option<Range<usize>> {
    if let Some(glyph) = letter.glyph {
        return cells(vram, letter.place, std::slice::from_ref(&glyph));
    }
    let span = map_span(letter.place, 1)?;
    vram.get(span.clone())?;
    Some(span)
}

fn cells(vram: &mut [u16], place: Place, updates: &[Cell]) -> Option<Range<usize>> {
    let span = map_span(place, updates.len())?;
    for (word, &cell) in vram.get_mut(span.clone())?.iter_mut().zip(updates) {
        *word = TilemapEntry::from(cell).word();
    }
    Some(span)
}

fn map_span(place: Place, width: usize) -> Option<Range<usize>> {
    let column = usize::from(place.column);
    let row = usize::from(place.row);
    if width == 0 || column.checked_add(width)? > MAP_SIDE || row >= MAP_SIDE {
        return None;
    }
    let start = MAP_BASE
        .checked_add(row.checked_mul(MAP_SIDE)?)?
        .checked_add(column)?;
    Some(start..start.checked_add(width)?)
}
