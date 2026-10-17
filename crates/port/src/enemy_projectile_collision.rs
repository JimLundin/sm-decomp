//! Bank86 terrain collision, including extension blocks and bank94 slope heights.
use crate::Wram;
struct Collision {
    negative: bool,
    next: u16,
    remaining: u16,
    rx: u16,
    ry: u16,
    total: u16,
    edge: u16,
}
fn slot(k: u16) -> Result<usize, &'static str> {
    if k <= 34 && k & 1 == 0 { Ok(k as usize) } else { Err("invalid enemy projectile slot") }
}
fn square(r: &mut Wram<'_>, c: &Collision, bts: u8, horizontal: bool) -> Result<u8, &'static str> {
    const FLAGS: [u8; 20] =
        [0, 1, 0x82, 0x83, 0, 0x81, 2, 0x83, 0, 1, 2, 0x83, 0, 0x81, 0x82, 0x83, 0x80, 0x81, 0x82, 0x83];
    let k = slot(r.word(0x1991))?;
    let (center, radius, xor, shift, pos, sub) = if horizontal {
        (r.word(0x1a93 + k), c.ry, 2, 3, 0x1a4b, 0x1a27)
    } else {
        (r.word(0x1a4b + k), c.rx, 1, 2, 0x1a93, 0x1a6f)
    };
    let index = usize::from(4 * u16::from(bts & 31) + (u16::from(bts >> 6) ^ ((c.edge & 8) >> shift)));
    let solid = |i: usize| FLAGS[i] & 0x80 != 0;
    let low_half = center.wrapping_sub(radius) & 8 == 0;
    let high_half = center.wrapping_add(radius).wrapping_sub(1) & 8 != 0;
    let hit = if c.total == 0 {
        (low_half && solid(index)) || (high_half && solid(index ^ xor))
    } else if c.remaining == 0 && !high_half {
        solid(index)
    } else {
        ((c.remaining != c.total || low_half) && solid(index)) || solid(index ^ xor)
    };
    if hit {
        let radius = if horizontal { c.rx } else { c.ry };
        let aligned = if c.negative {
            radius.wrapping_add(c.edge | 7).wrapping_add(1)
        } else {
            (c.edge & 0xfff8).wrapping_sub(radius)
        };
        r.set_word(sub + k, 0);
        r.set_word(pos + k, aligned);
    }
    Ok(u8::from(hit))
}
fn slope(r: &mut Wram<'_>, rom: &[u8], c: &Collision, bts: u8, block: u16) -> Result<u8, &'static str> {
    let k = slot(r.word(0x1991))?;
    if r.word(0x1a4b + k) >> 4 != if r.word(0x7a5) as u8 == 0 { block } else { block % u16::from(r.word(0x7a5) as u8) }
        || (bts & 0x80 != 0) != c.negative
    {
        return Ok(0);
    }
    let local_y =
        if c.negative { (c.next.wrapping_sub(c.ry) & 15) ^ 15 } else { c.next.wrapping_add(c.ry).wrapping_sub(1) & 15 };
    let x = r.word(0x1a4b + k) ^ if bts & 0x40 != 0 { 15 } else { 0 };
    let offset = 0xa0b2b + usize::from(bts & 31) * 16 + usize::from(x & 15);
    let height = i16::from(*rom.get(offset).ok_or("missing enemy projectile slope ROM")? & 31);
    let delta = height - local_y as i16 - 1;
    if delta <= 0 {
        r.set_word(
            0x1a93 + k,
            if c.negative { c.next.wrapping_sub(delta as u16) } else { c.next.wrapping_add(delta as u16) },
        );
        r.set_word(0x1a6f + k, if c.negative { 0 } else { 0xffff });
        Ok(1)
    } else {
        Ok(0)
    }
}
fn check(r: &mut Wram<'_>, rom: &[u8], c: &Collision, start: u16, horizontal: bool) -> Result<u8, &'static str> {
    let mut block = start >> 1;
    for _ in 0..0x3200 {
        if block >= 0x3200 {
            return Err("enemy projectile terrain extent");
        }
        r.set_word(0xdc4, block);
        let bts = r.byte(0x16402 + usize::from(block));
        match r.word(0x10002 + usize::from(block) * 2) >> 12 {
            5 | 13 if bts != 0 => {
                let scale = if r.word(0x10002 + usize::from(block) * 2) >> 12 == 13 { r.word(0x7a5) } else { 1 };
                block = block.wrapping_add((bts as i8 as u16).wrapping_mul(scale));
            }
            1 if bts & 31 < 5 => return square(r, c, bts, horizontal),
            1 if horizontal => {
                r.set_word(0x1e77, u16::from(bts));
                return Ok(0);
            }
            1 => return slope(r, rom, c, bts, block),
            8..=12 | 15 => return Ok(1),
            _ => return Ok(0),
        }
    }
    Err("enemy projectile extension cycle")
}
fn move_colliding(r: &mut Wram<'_>, rom: &[u8], k: u16, horizontal: bool) -> Result<u8, &'static str> {
    let k = slot(k)?;
    let width = r.word(0x7a5);
    if width == 0 {
        return Err("zero enemy projectile room width");
    }
    let (pos, sub, vel) = if horizontal { (0x1a4b, 0x1a27, 0x1ab7) } else { (0x1a93, 0x1a6f, 0x1adb) };
    let amt = (r.word(vel + k) as i16 as i32) << 8;
    let result = ((u32::from(r.word(pos + k)) << 16) | u32::from(r.word(sub + k))).wrapping_add(amt as u32);
    let next = (result >> 16) as u16;
    let radius = r.word(0x1bb3 + k);
    let (rx, ry) = (radius & 255, radius >> 8);
    let (perpendicular, extent) = if horizontal { (r.word(0x1a93 + k), ry) } else { (r.word(0x1a4b + k), rx) };
    let low = perpendicular.wrapping_sub(extent);
    let total = perpendicular.wrapping_add(extent).wrapping_sub(1).wrapping_sub(low & 0xfff0) >> 4;
    let along = if horizontal { rx } else { ry };
    let edge = if amt < 0 { next.wrapping_sub(along) } else { next.wrapping_add(along).wrapping_sub(1) };
    let mut c = Collision { negative: amt < 0, next, remaining: total, rx, ry, total, edge };
    let (row, col) = if horizontal { (low >> 4, edge >> 4) } else { (edge >> 4, low >> 4) };
    let product = u16::from(row as u8) * u16::from(width as u8);
    let mut index = product.wrapping_add(col).wrapping_mul(2);
    loop {
        if check(r, rom, &c, index, horizontal)? & 1 != 0 {
            break;
        }
        index = index.wrapping_add(if horizontal { width.wrapping_mul(2) } else { 2 });
        c.remaining = c.remaining.wrapping_sub(1);
        if c.remaining & 0x8000 != 0 {
            r.set_word(pos + k, next);
            r.set_word(sub + k, result as u16);
            return Ok(0);
        }
    }
    r.set_word(sub + k, 0);
    let aligned =
        if c.negative { along.wrapping_add(edge | 15).wrapping_add(1) } else { (edge & 0xfff0).wrapping_sub(along) };
    if (c.negative && aligned <= r.word(pos + k)) || (!c.negative && aligned >= r.word(pos + k)) {
        r.set_word(pos + k, aligned);
    }
    Ok(1)
}
pub fn horizontal(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<u8, &'static str> {
    move_colliding(r, rom, k, true)
}
pub fn vertical(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<u8, &'static str> {
    move_colliding(r, rom, k, false)
}

pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], name: &str, a: i32, _b: i32) -> Result<u32, &'static str> {
    match name {
        "EprojBlockCollisition_Horiz" => horizontal(r, rom, a as u16).map(u32::from),
        "EprojBlockCollisition_Vertical" => vertical(r, rom, a as u16).map(u32::from),
        _ => Err("unsupported enemy projectile collision routine"),
    }
}
