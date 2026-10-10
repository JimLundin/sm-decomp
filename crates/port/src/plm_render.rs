//! PLM block coordinates and tilemap rendering, translated from pinned bank84.
use crate::Wram;
fn slot(k: u16) -> Result<usize, &'static str> {
    if k > 78 || k & 1 != 0 { Err("invalid PLM slot") } else { Ok(k as usize) }
}
pub fn calculate_block_coords(r: &mut Wram<'_>, k: u16) -> Result<(), &'static str> {
    let k = slot(k)?;
    let block = r.word(0x1c87 + k) >> 1;
    let width = r.byte(0x7a5) as u16;
    r.set_word(0x1c2b, if width == 0 { 0xffff } else { block / width });
    r.set_word(0x1c29, if width == 0 { block } else { block % width });
    Ok(())
}
fn negative(n: u16) -> bool {
    n & 0x8000 != 0
}
fn data(r: &Wram<'_>, rom: &[u8], pc: u16) -> Result<u16, &'static str> {
    if pc < 0x8000 {
        if pc >= 0x1fff {
            return Err("PLM draw RAM extent");
        }
        return Ok(r.word(pc as usize));
    }
    if pc == 0xffff {
        return Err("PLM draw bank extent");
    }
    let off = 0x20000 + (pc as usize & 0x7fff);
    let b = rom.get(off..off + 2).ok_or("PLM draw ROM extent")?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}
fn destination(r: &Wram<'_>, x: u16, y: u16) -> u16 {
    let column = x & 31;
    let screen = if column >= 16 { 0x53e0u16 } else { 0x5000u16 };
    let mut dst = screen.wrapping_add((y & 15) * 64).wrapping_add(column * 2);
    if r.word(0x91d) & 0x100 != 0 {
        dst = if column >= 16 { dst.wrapping_sub(1024) } else { dst.wrapping_add(1024) };
    }
    dst
}
fn entry(r: &mut Wram<'_>, tail: usize, size: u16, src: u16, dst: u16) {
    let p = 0xd0 + tail;
    r.set_word(p, size);
    r.set_word(p + 2, src);
    r.set_byte(p + 4, 0x7e);
    r.set_word(p + 5, dst);
}
fn tiles(r: &Wram<'_>, block: u16) -> [u16; 4] {
    let base = 0xa000 + (block as usize & 0x3ff) * 8;
    let original = [r.word(base), r.word(base + 2), r.word(base + 4), r.word(base + 6)];
    let flip = block & 0xc00;
    let xor = flip << 4;
    let order = match flip {
        0x400 => [1, 0, 3, 2],
        0x800 => [2, 3, 0, 1],
        0xc00 => [3, 2, 1, 0],
        _ => [0, 1, 2, 3],
    };
    order.map(|i| original[i] ^ xor)
}
/// Expand all visible draw-data segments into WRAM tiles and the VRAM queue.
/// Invalid pointers and malformed zero-length runs are rejected, never read out
/// of bounds. Valid data preserves the C renderer's clipping and capacity stops.
pub fn draw(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<(), &'static str> {
    let k = slot(k)?;
    let mut pc = r.word(0xde6c + k);
    let origin_x = r.word(0x1c29);
    let origin_y = r.word(0x1c2b);
    let mut x = origin_x;
    let mut y = origin_y;
    for _ in 0..4096 {
        let camera_y = r.word(0x915) >> 4;
        if negative(camera_y.wrapping_add(15).wrapping_sub(y)) {
            return Ok(());
        }
        let header = data(r, rom, pc)?;
        let vertical = negative(header);
        let mut count = header & 0x7fff;
        if count == 0 {
            return Err("empty PLM draw run");
        }
        let mut skip = 0u16;
        let draw_x;
        let draw_y;
        let excess;
        if vertical {
            let camera_x = r.word(0x911) >> 4;
            if negative(x.wrapping_sub(camera_x)) {
                return Ok(());
            }
            let right = camera_x.wrapping_add(17);
            if right == x || negative(right.wrapping_sub(x)) {
                return Ok(());
            }
            draw_x = x;
            draw_y = if negative(camera_y.wrapping_sub(y)) { y } else { camera_y };
            if !negative(camera_y.wrapping_sub(y)) {
                skip = camera_y.wrapping_sub(y);
                if negative(y.wrapping_add(count).wrapping_sub(camera_y)) || count == skip {
                    return Ok(());
                }
                if negative(count.wrapping_sub(skip)) {
                    return Err("PLM vertical clipping extent");
                }
                count = count.wrapping_sub(skip);
            }
            excess = count.wrapping_add(draw_y).wrapping_sub(camera_y.wrapping_add(16));
            if !negative(excess) {
                let old = count;
                count = count.wrapping_sub(excess);
                if count == 0 || negative(old.wrapping_sub(excess)) {
                    return Ok(());
                }
            }
        } else {
            if negative(y.wrapping_sub(camera_y)) {
                return Ok(());
            }
            draw_y = y;
            let left = (r.word(0x911).wrapping_add(15) >> 4).wrapping_sub(1);
            let mut start = x;
            if !negative(left.wrapping_sub(x)) && left != x {
                skip = left.wrapping_sub(x);
                let end = x.wrapping_add(count);
                if end == left || negative(end.wrapping_sub(left)) {
                    return Ok(());
                }
                count = count.wrapping_sub(skip);
                start = left;
            }
            draw_x = start;
            let right = left.wrapping_add(17);
            if negative(right.wrapping_sub(x)) {
                return Ok(());
            }
            excess = count.wrapping_add(start).wrapping_sub(1).wrapping_sub(right);
            if !negative(excess) {
                count = count.wrapping_sub(excess);
                if count == 0 {
                    return Ok(());
                }
            }
        }
        let index = r.word(0x1c25);
        let tail = r.word(0x330);
        if index > 512 {
            return Err("PLM draw buffer extent");
        }
        if tail > 0x260 {
            return Err("VRAM queue extent");
        }
        if !negative(tail.wrapping_sub(if vertical { 240 } else { 480 })) || count > (512 - index) / 8 {
            return Ok(());
        }
        let dst = destination(r, draw_x, draw_y) | if vertical { 0x8000 } else { 0 };
        let base = 0xc6c8 + index;
        let row_size = count * 4;
        let split = !vertical && (count * 2 + (dst & 31) - 1) & 0xffe0 != 0;
        if split && (!negative(tail.wrapping_sub(228)) || negative(32u16.wrapping_sub(dst & 31))) {
            return Ok(());
        }
        if split {
            let first = (32 - (dst & 31)) * 2;
            let second = row_size - first;
            let other = (dst & 0xffe0) ^ 0x400;
            entry(r, tail as usize, first, base, dst);
            entry(r, tail as usize + 7, second, base + first, other);
            entry(r, tail as usize + 14, first, base + row_size, dst + 32);
            entry(r, tail as usize + 21, second, base + row_size + first, other + 32);
            r.set_word(0x330, tail + 28);
        } else {
            entry(r, tail as usize, row_size, base, dst);
            entry(r, tail as usize + 7, row_size, base + row_size, dst.wrapping_add(if vertical { 1 } else { 32 }));
            r.set_word(0x330, tail + 14);
        }
        let mut cursor =
            pc.checked_add(2).and_then(|p| p.checked_add(skip.checked_mul(2)?)).ok_or("PLM draw pointer extent")?;
        for i in 0..count {
            let tile = tiles(r, data(r, rom, cursor)?);
            let offset = base as usize + i as usize * 4;
            let (first, second) = if vertical {
                ([tile[0], tile[2]], [tile[1], tile[3]])
            } else {
                ([tile[0], tile[1]], [tile[2], tile[3]])
            };
            r.set_word(offset, first[0]);
            r.set_word(offset + 2, first[1]);
            r.set_word(offset + row_size as usize, second[0]);
            r.set_word(offset + row_size as usize + 2, second[1]);
            cursor = cursor.checked_add(2).ok_or("PLM draw pointer extent")?;
            r.set_word(0x1c25, r.word(0x1c25) + 8);
            if r.word(0x1c25) >= 512 {
                return Ok(());
            }
        }
        if !negative(excess) {
            cursor = cursor
                .checked_add(excess.checked_mul(2).ok_or("PLM draw pointer extent")?)
                .ok_or("PLM draw pointer extent")?;
        }
        let offset = data(r, rom, cursor)?;
        if offset == 0 {
            return Ok(());
        }
        x = origin_x.wrapping_add(offset as u8 as i8 as i16 as u16);
        y = origin_y.wrapping_add((offset >> 8) as u8 as i8 as i16 as u16);
        pc = cursor.checked_add(2).ok_or("PLM draw pointer extent")?;
    }
    Err("PLM draw segment budget")
}
