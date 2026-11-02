//! Menu composition on the canonical presentation workspace.
#![forbid(unsafe_code)]
use crate::display::bg_tile;
use crate::presentation_workspace::{Canvas, Point, Role};
use crate::scene::state::DisplayError;

pub fn paint_file(
    workspace: &mut (impl Canvas + ?Sized),
    offset: usize,
    tile: u16,
    palette: u16,
) -> Result<(), DisplayError> {
    workspace.write_native_slice(Point::new(Role::Menu, offset), palette | tile)
}

pub fn clear_file(workspace: &mut (impl Canvas + ?Sized)) -> Result<(), DisplayError> {
    for word in (0..1024).rev() {
        workspace.write_native_word(Point::new(Role::Menu, 2 * word), 15)?;
    }
    Ok(())
}

pub fn fill_native(
    workspace: &mut (impl Canvas + ?Sized),
    first: Point,
    count: usize,
    descending: bool,
    value: u16,
) -> Result<(), DisplayError> {
    for index in 0..count {
        let offset = if descending {
            first.offset - 2 * index
        } else {
            first.offset + 2 * index
        };
        workspace.write_native_word(Point::new(first.role, offset), value)?;
    }
    Ok(())
}

pub fn clear_file_row(
    workspace: &mut (impl Canvas + ?Sized),
    offset: u16,
) -> Result<(), DisplayError> {
    let count = 32 - usize::from((offset & 0x3f) >> 1);
    for word in 0..count {
        workspace.write_native_word(Point::new(Role::Menu, usize::from(offset) + 2 * word), 15)?;
    }
    Ok(())
}

pub fn recolor(
    workspace: &mut (impl Canvas + ?Sized),
    point: Point,
    palette: u16,
) -> Result<(), DisplayError> {
    let tile = workspace.native_word(point)?;
    workspace.write_native_word(point, bg_tile::recolour(tile, palette))
}

pub fn reserve_digits(
    workspace: &mut (impl Canvas + ?Sized),
    hundreds: u16,
    remainder: u16,
) -> Result<(), DisplayError> {
    for (index, digit) in [
        (394, remainder % 10),
        (393, remainder / 10),
        (392, hundreds),
    ] {
        workspace.write_native_word(
            Point::new(Role::Equipment, 2 * index),
            digit + bg_tile::tile(4, 2),
        )?;
    }
    Ok(())
}

pub fn pause_labels(workspace: &mut (impl Canvas + ?Sized), mode: u16) -> Result<(), DisplayError> {
    let lit = bg_tile::palette(2);
    let dim = bg_tile::palette(5);
    let labels = [
        [((822, 5), lit), ((812, 4), lit), ((805, 5), dim)],
        [((812, 4), lit), ((805, 5), dim), ((822, 5), dim)],
        [((805, 5), lit), ((812, 4), lit), ((822, 5), dim)],
    ][usize::from(mode)];
    for ((first, width), palette) in labels {
        for row in [first, first + 32] {
            for tile in row..row + width {
                let point = Point::new(Role::Pause, 2 * tile);
                let previous = workspace.word(point)?;
                workspace
                    .write_slice(point, &bg_tile::recolour(previous, palette).to_le_bytes())?;
            }
        }
    }
    Ok(())
}

/// Lazy map sources and the canonical progress/display destinations.
pub trait MapComposition {
    fn tile(&mut self, index: usize) -> u16;
    fn mapped_word(&mut self, offset: usize) -> u16;
    fn explored_word(&mut self, offset: usize) -> u16;
    fn explored_byte(&mut self, offset: usize) -> u8;
    fn store_explored_byte(&mut self, offset: usize, value: u8);
    fn store_tile(&mut self, index: usize, value: u16);
    fn observed_store(&mut self, byte: usize, index: usize, stage: u8);
}

pub fn compose_map(mapped_area: bool, empty: u16, map: &mut impl MapComposition) {
    if mapped_area {
        let mut mapped = map.mapped_word(0).swap_bytes();
        let mut explored = map.explored_word(0).swap_bytes();
        for index in 0..2048 {
            let mut value = map.tile(index);
            if explored & 0x8000 != 0 {
                value &= 0xfbff;
            } else if mapped & 0x8000 == 0 {
                value = 31;
            }
            explored = explored.wrapping_mul(2);
            mapped = mapped.wrapping_mul(2);
            map.store_tile(index, value);
            if index & 15 == 15 {
                // The final tile also prefetches the following native word.
                let offset = 2 * ((index + 1) / 16);
                mapped = map.mapped_word(offset).swap_bytes();
                explored = map.explored_word(offset).swap_bytes();
            }
        }
    } else {
        for index in 0..2048 {
            let byte = index / 8;
            let previous = map.explored_byte(byte);
            let shifted = map.explored_byte(byte).wrapping_mul(2);
            map.store_explored_byte(byte, shifted);
            map.observed_store(byte, index, 0);
            let value = if previous & 0x80 == 0 {
                empty
            } else {
                let rotated = map.explored_byte(byte).wrapping_add(1);
                map.store_explored_byte(byte, rotated);
                map.observed_store(byte, index, 1);
                map.tile(index) & 0xfbff
            };
            map.store_tile(index, value);
            map.observed_store(byte, index, 2);
        }
    }
}

pub trait TextComposition {
    fn word(&mut self, cursor: u16) -> u16;
    fn paint(&mut self, offset: usize, tile: u16);
}

pub fn load_text(mut destination: u16, mut source: u16, text: &mut impl TextComposition) {
    let mut row = i32::from(destination);
    loop {
        match text.word(source) {
            0xfffe => {
                row += 64;
                let bytes = row.to_le_bytes();
                destination = u16::from_le_bytes([bytes[0], bytes[1]]);
            }
            0xffff => return,
            tile => {
                text.paint(usize::from(destination), tile);
                destination = destination.wrapping_add(2);
            }
        }
        source = source.wrapping_add(2);
    }
}
