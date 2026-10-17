//! Bank80 room scrolling and tilemap/DMA queue preparation.
//! Upstream license: crates/decomp/LICENSE.upstream.txt.
use crate::{Host, Result, WRAM_SIZE, Wram};
const X: usize = 0x0911;
const Y: usize = 0x0915;
const L2_X: usize = 0x0917;
const L2_Y: usize = 0x0919;
const BLOCK_X: usize = 0x08f7;
const BLOCK_Y: usize = 0x08f9;
const L2_BLOCK_X: usize = 0x08fb;
const L2_BLOCK_Y: usize = 0x08fd;
const PREVIOUS_BLOCK_X: usize = 0x08ff;
const PREVIOUS_BLOCK_Y: usize = 0x0901;
const PREVIOUS_L2_BLOCK_X: usize = 0x0903;
const PREVIOUS_L2_BLOCK_Y: usize = 0x0905;
const BG1_BLOCK_X: usize = 0x0907;
const BG1_BLOCK_Y: usize = 0x0909;
const BG2_BLOCK_X: usize = 0x090b;
const BG2_BLOCK_Y: usize = 0x090d;
const WIDTH_BLOCKS: usize = 0x07a5;
const MODE7: usize = 0x0783;
const FROZEN: usize = 0x0a78;
const COLUMN: usize = 0x0956;
const ROW: usize = 0x0964;
const BG2_OFFSET: usize = 0x001c;
const BG2_SIZE: usize = 0x098e;
const UPDATE_X: usize = 0x0990;
const UPDATE_Y: usize = 0x0992;
const VRAM_X: usize = 0x0994;
const VRAM_Y: usize = 0x0996;
const TILE_TABLE: usize = 0xa000;
const COLUMN_LEFT: usize = 0xc8c8;
const COLUMN_RIGHT: usize = 0xc908;
const ROW_TOP: usize = 0xc948;
const ROW_BOTTOM: usize = 0xc98c;
const BG2_BUFFER_OFFSET: usize = 264;

fn multiply_bytes(a: u16, b: u16) -> u16 {
    u16::from(a as u8) * u16::from(b as u8)
}

fn tile_source(ram: &Wram<'_>, background: bool) -> u16 {
    let product = multiply_bytes(ram.word(UPDATE_Y), ram.word(WIDTH_BLOCKS));
    let mut offset = product.wrapping_add(ram.word(UPDATE_X)).wrapping_mul(2).wrapping_add(2);
    if background {
        offset = offset.wrapping_add(0x9600);
    }
    offset
}

fn tile_address(start: u16, index: usize) -> Result<usize> {
    let address = 0x10000 + usize::from(start) + index * 2;
    if address + 2 > WRAM_SIZE {
        return Err("room scroll tile source outside full WRAM".into());
    }
    Ok(address)
}

fn tile(ram: &Wram<'_>, address: usize) -> [u16; 4] {
    let block = ram.word(address);
    let table = TILE_TABLE + usize::from(block & 0x03ff) * 8;
    let flip = block & 0x0c00;
    let (order, mask) = match flip {
        0x0400 => ([1, 0, 3, 2], 0x4000),
        0x0800 => ([2, 3, 0, 1], 0x8000),
        0x0c00 => ([3, 2, 1, 0], 0xc000),
        _ => ([0, 1, 2, 3], 0),
    };
    order.map(|index| ram.word(table + index * 2) ^ mask)
}

fn column(ram: &mut Wram<'_>, background: bool) -> Result<()> {
    if ram.word(MODE7) != 0 {
        return Ok(());
    }
    let start = tile_source(ram, background);
    let descriptor = COLUMN + if background { BG2_OFFSET } else { 0 };
    let buffer = if background { BG2_BUFFER_OFFSET } else { 0 };
    let vram_x = ram.word(VRAM_X) & 0x1f;
    let vram_y = ram.word(VRAM_Y);
    let wrapped_size = vram_y.wrapping_mul(4) & 0x3c;
    let unwrapped_size = (wrapped_size ^ 0x3f).wrapping_add(1);
    let row_offset = multiply_bytes(vram_y & 0x0f, 0x40);
    let mut base = if vram_x >= 0x10 { 0x53e0_u16 } else { 0x5000 };
    if background {
        base = base.wrapping_sub(ram.word(BG2_SIZE));
    }
    let unwrapped_destination = row_offset.wrapping_add(vram_x * 2).wrapping_add(base);
    let wrapped_destination = (vram_x * 2).wrapping_add(base);
    let source = (COLUMN_LEFT + buffer) as u16;
    let wrapped_source = source.wrapping_add(unwrapped_size);
    for (offset, value) in [
        (0, unwrapped_size),
        (2, wrapped_size),
        (4, unwrapped_destination),
        (6, wrapped_destination),
        (8, wrapped_source),
        (10, wrapped_source.wrapping_add(64)),
    ] {
        ram.set_word(descriptor + offset, value);
    }
    let mut native_offset = 0_u16;
    for row in 0..16 {
        let words = tile(ram, tile_address(start, usize::from(native_offset >> 1))?);
        for (offset, value) in [
            (COLUMN_LEFT + buffer + row * 4, words[0]),
            (COLUMN_RIGHT + buffer + row * 4, words[1]),
            (COLUMN_LEFT + buffer + row * 4 + 2, words[2]),
            (COLUMN_RIGHT + buffer + row * 4 + 2, words[3]),
        ] {
            ram.set_word(offset, value);
        }
        native_offset = native_offset.wrapping_add(ram.word(WIDTH_BLOCKS).wrapping_mul(2));
    }
    ram.set_word(descriptor + 12, ram.word(descriptor + 12).wrapping_add(1));
    Ok(())
}

fn row(ram: &mut Wram<'_>, background: bool) -> Result<()> {
    if ram.word(MODE7) != 0 {
        return Ok(());
    }
    let start = tile_source(ram, background);
    let descriptor = ROW + if background { BG2_OFFSET } else { 0 };
    let buffer = if background { BG2_BUFFER_OFFSET } else { 0 };
    let vram_x = ram.word(VRAM_X) & 0x1f;
    let column = vram_x & 0x0f;
    let unwrapped_size = 4 * (16 - column);
    let wrapped_size = 4 * (column + 1);
    let row_offset = multiply_bytes(ram.word(VRAM_Y) & 0x0f, 0x40);
    let (mut base, mut wrapped_base) = if vram_x >= 0x10 { (0x53e0_u16, 0x5000_u16) } else { (0x5000, 0x5400) };
    if background {
        base = base.wrapping_sub(ram.word(BG2_SIZE));
        wrapped_base = wrapped_base.wrapping_sub(ram.word(BG2_SIZE));
    }
    let source = (ROW_TOP + buffer) as u16;
    let wrapped_source = source.wrapping_add(unwrapped_size);
    for (offset, value) in [
        (0, unwrapped_size),
        (2, wrapped_size),
        (4, row_offset.wrapping_add(vram_x * 2).wrapping_add(base)),
        (6, row_offset.wrapping_add(wrapped_base)),
        (8, wrapped_source),
        (10, wrapped_source.wrapping_add(68)),
    ] {
        ram.set_word(descriptor + offset, value);
    }
    for column in 0..17 {
        let words = tile(ram, tile_address(start, column)?);
        for (offset, value) in [
            (ROW_TOP + buffer + column * 4, words[0]),
            (ROW_TOP + buffer + column * 4 + 2, words[1]),
            (ROW_BOTTOM + buffer + column * 4, words[2]),
            (ROW_BOTTOM + buffer + column * 4 + 2, words[3]),
        ] {
            ram.set_word(offset, value);
        }
    }
    ram.set_word(descriptor + 12, ram.word(descriptor + 12).wrapping_add(1));
    Ok(())
}

fn select_blocks(ram: &mut Wram<'_>, background: bool, x_offset: u16, y_offset: u16) {
    let (x, y, vram_x, vram_y) = if background {
        (L2_BLOCK_X, L2_BLOCK_Y, BG2_BLOCK_X, BG2_BLOCK_Y)
    } else {
        (BLOCK_X, BLOCK_Y, BG1_BLOCK_X, BG1_BLOCK_Y)
    };
    ram.set_word(UPDATE_X, ram.word(x).wrapping_add(x_offset));
    ram.set_word(UPDATE_Y, ram.word(y).wrapping_add(y_offset));
    ram.set_word(VRAM_X, ram.word(vram_x).wrapping_add(x_offset));
    ram.set_word(VRAM_Y, ram.word(vram_y).wrapping_add(y_offset));
}

fn update_graphics(ram: &mut Wram<'_>) -> Result<()> {
    calculate_blocks(ram);
    // Native order is foreground X, background X, foreground Y, background Y.
    for (background, current, previous) in [(false, BLOCK_X, PREVIOUS_BLOCK_X), (true, L2_BLOCK_X, PREVIOUS_L2_BLOCK_X)]
    {
        if background && ram.byte(0x091b) & 1 != 0 {
            continue;
        }
        if ram.word(current) != ram.word(previous) {
            let left = ram.word(current).wrapping_sub(ram.word(previous)) & 0x8000 != 0;
            ram.set_word(previous, ram.word(current));
            select_blocks(ram, background, if left { 0 } else { 16 }, 0);
            column(ram, background)?;
        }
    }
    for (background, current, previous) in [(false, BLOCK_Y, PREVIOUS_BLOCK_Y), (true, L2_BLOCK_Y, PREVIOUS_L2_BLOCK_Y)]
    {
        if background && ram.byte(0x091c) & 1 != 0 {
            continue;
        }
        if ram.word(current) != ram.word(previous) {
            let up = ram.word(current).wrapping_sub(ram.word(previous)) & 0x8000 != 0;
            ram.set_word(previous, ram.word(current));
            select_blocks(ram, background, 0, if up { 1 } else { 15 });
            row(ram, background)?;
        }
    }
    Ok(())
}

pub(crate) fn layer2_and_scrolls(ram: &mut Wram<'_>) -> Result<()> {
    if ram.word(FROZEN) != 0 {
        return Ok(());
    }
    ram.set_word(0x00b1, ram.word(0x091d).wrapping_add(ram.word(X)));
    ram.set_word(0x00b3, ram.word(0x091f).wrapping_add(ram.word(Y)));
    if calculate_layer2(ram, true) {
        ram.set_word(0x00b5, ram.word(0x0921).wrapping_add(ram.word(L2_X)));
    }
    if calculate_layer2(ram, false) {
        ram.set_word(0x00b7, ram.word(0x0923).wrapping_add(ram.word(L2_Y)));
    }
    update_graphics(ram)
}
pub(crate) fn display(ram: &mut Wram<'_>, host: &mut impl Host) -> Result<()> {
    let size = (i32::from(ram.byte(0x0058)) - i32::from(ram.byte(0x0059))) * 256;
    ram.set_word(BG2_SIZE, size as u16 & 0xf800);
    calculate_blocks(ram);
    for _ in 0..17 {
        select_blocks(ram, false, 0, 0);
        column(ram, false)?;
        if ram.byte(0x091b) & 1 == 0 {
            select_blocks(ram, true, 0, 0);
            column(ram, true)?;
        }
        host.flush_vram(ram.0)?;
        for offset in [BLOCK_X, BG1_BLOCK_X, L2_BLOCK_X, BG2_BLOCK_X] {
            ram.set_word(offset, ram.word(offset).wrapping_add(1));
        }
    }
    Ok(())
}

fn calculate_layer2(ram: &mut Wram<'_>, x: bool) -> bool {
    let (position, target, ratio) = if x { (X, L2_X, 0x091b) } else { (Y, L2_Y, 0x091c) };
    let factor = ram.byte(ratio);
    if factor == 1 {
        return false;
    }
    let position = ram.word(position);
    let value = if factor == 0 {
        position
    } else {
        let factor = u16::from(factor & 0xfe);
        factor * (position >> 8) + ((factor * u16::from(position as u8)) >> 8)
    };
    ram.set_word(target, value);
    true
}

fn calculate_blocks(ram: &mut Wram<'_>) {
    for (target, source) in [(BG1_BLOCK_X, 0x00b1), (BG2_BLOCK_X, 0x00b5), (BG1_BLOCK_Y, 0x00b3), (BG2_BLOCK_Y, 0x00b7)]
    {
        ram.set_word(target, ram.word(source) >> 4);
    }
    for (target, source) in [(BLOCK_X, X), (L2_BLOCK_X, L2_X), (BLOCK_Y, Y), (L2_BLOCK_Y, L2_Y)] {
        ram.set_word(target, ((ram.word(source) as i16) >> 4) as u16);
    }
}

pub(crate) fn update_scroll_vars(ram: &mut Wram<'_>) -> Result<()> {
    for (target, offset, position) in
        [(0x00b1, 0x091d, X), (0x00b3, 0x091f, Y), (0x00b5, 0x0921, L2_X), (0x00b7, 0x0923, L2_Y)]
    {
        ram.set_word(target, ram.word(offset).wrapping_add(ram.word(position)));
    }
    update_graphics(ram)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
