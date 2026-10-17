//! Normal Samus projectile flight, shot terrain reactions and firing.
//! Pinned snesrev/sm banks $90/$94/$9B/$8B; upstream MIT notice applies.
use crate::core::{INDEX, RADIUS_X, RADIUS_Y, SPEED_X, SPEED_Y, SUB_X, SUB_Y, X, Y, slot};
use crate::{Host, Result, Wram, core as projectile_core, rw};
const AREA: usize = 0x79f;
const PLM_HEADERS: usize = 0x1c37;
const SAMUS_COLLISION_DIRECTION: usize = 0xb02;
const WIDTH: usize = 0x7a5;
const HEIGHT: usize = 0x7a7;
const WIDTH_SCROLLS: usize = 0x7a9;
const HEIGHT_SCROLLS: usize = 0x7ab;
const ROOM_BYTES: usize = 0x7b9;
const BLOCK: usize = 0xdc4;
const LEVEL: usize = 0x10002;
const BTS: usize = 0x16402;
const SQUARE: [u8; 20] = [0, 0, 128, 128, 0, 128, 0, 128, 0, 0, 0, 128, 0, 128, 128, 128, 128, 128, 128, 128];
const SLOPE: [u8; 512] = [
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 16, 16, 16, 16, 16, 16, 16, 16, 0, 0, 0, 0, 0, 0, 0, 0, 16, 16, 16,
    16, 16, 16, 16, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 16, 15, 14, 13, 12, 11, 10, 9, 9, 10, 11, 12, 13, 14, 15, 16, 16, 14, 12, 10, 8, 6, 4, 2,
    2, 4, 6, 8, 10, 12, 14, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 12, 12, 12, 12, 8, 8, 8, 8, 4, 4, 4, 4, 0, 0, 0, 0, 14, 14, 12, 12, 10, 10, 8, 8, 6, 6, 4, 4,
    2, 2, 0, 0, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20,
    20, 20, 20, 16, 16, 16, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 16, 16, 16, 16, 16, 16, 16, 16, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 16, 16, 15, 15, 14, 14, 13, 13, 12, 12, 11, 11, 10, 10, 9, 9, 8, 8, 7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 2, 1,
    1, 16, 16, 16, 15, 15, 15, 14, 14, 14, 13, 13, 13, 12, 12, 12, 11, 11, 11, 10, 10, 10, 9, 9, 9, 8, 8, 8, 7, 7, 7,
    6, 6, 6, 5, 5, 5, 4, 4, 4, 3, 3, 3, 2, 2, 2, 1, 1, 1, 20, 20, 20, 20, 20, 20, 20, 20, 16, 14, 12, 10, 8, 6, 4, 2,
    16, 14, 12, 10, 8, 6, 4, 2, 0, 0, 0, 0, 0, 0, 0, 0, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 15, 12, 9, 6, 3,
    20, 20, 20, 20, 20, 20, 14, 11, 8, 5, 2, 0, 0, 0, 0, 0, 16, 13, 10, 7, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
fn ptr(rom: &[u8], bank: u32, p: u16, delta: u16) -> Result<u16> {
    rw(rom, bank | u32::from(p.checked_add(delta).ok_or("ROM pointer wrap")?))
}
fn room(r: &Wram<'_>) -> Result<()> {
    let w = r.word(WIDTH);
    let h = r.word(HEIGHT);
    let size = r.word(ROOM_BYTES);
    if w == 0 || h == 0 || w > 255 || h > 255 || size & 1 != 0 || size > 0x6400 {
        return Err("room geometry/backing extent".into());
    }
    Ok(())
}
fn block(r: &Wram<'_>, k: u16) -> Result<(u16, u8)> {
    if k >= 0x3200 {
        return Err("tile backing extent".into());
    }
    Ok((r.word(LEVEL + 2 * k as usize), r.byte(BTS + k as usize)))
}
const PLM_SETUP: [u16; 115] = [
    0xab27, 0xb3cf, 0xb8ef, 0xbafa, 0xd6f6, 0xd6f7, 0xd18e, 0xab28, 0xb04a, 0xb0dc, 0xb113, 0xb18b, 0xb1c8, 0xb1f0,
    0xb21d, 0xb245, 0xb26d, 0xb29d, 0xb2d0, 0xb300, 0xb33a, 0xb345, 0xb350, 0xb35b, 0xb371, 0xb382, 0xb393, 0xb3c1,
    0xb3d0, 0xb3d2, 0xb3d4, 0xb3e3, 0xb3eb, 0xb408, 0xb497, 0xb4a8, 0xb4b6, 0xb4c4, 0xb541, 0xb54f, 0xb551, 0xb56f,
    0xb590, 0xb5ee, 0xb5f8, 0xb7c3, 0xb89c, 0xb8dc, 0xb96c, 0xb978, 0xb9c5, 0xb9f1, 0xbb09, 0xc54d, 0xc56c, 0xc58b,
    0xc5af, 0xc5d3, 0xc5f1, 0xc610, 0xc627, 0xc6be, 0xc6cb, 0xc6d8, 0xc6dc, 0xc6e0, 0xc73a, 0xc794, 0xc7b1, 0xc7bb,
    0xc7e2, 0xcdc2, 0xcdea, 0xce37, 0xce6b, 0xce83, 0xceda, 0xcf2e, 0xcf67, 0xcfa0, 0xcfb5, 0xcfcd, 0xcfd1, 0xcfd5,
    0xd108, 0xd117, 0xd12b, 0xd18f, 0xd5f6, 0xd606, 0xd616, 0xd620, 0xd67f, 0xd689, 0xd693, 0xd6cc, 0xda8c, 0xdab9,
    0xdb1e, 0xde94, 0xdeb9, 0xdef0, 0xdf15, 0xdf4c, 0xee4d, 0xee52, 0xee57, 0xee5c, 0xee64, 0xee77, 0xee7c, 0xee81,
    0xee86, 0xee8e, 0xeeab,
];
fn spawn(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host, header: u16) -> Result<()> {
    // Shot reactions do not point cur_coll_amt32 at the flight displacement.
    if (0..80).step_by(2).any(|j| r.word(PLM_HEADERS + j) == 0) {
        let setup = rw(rom, 0x840000 | u32::from(header))?;
        ptr(rom, 0x840000, header, 2)?;
        if setup == 0xb4c4 && r.word(SAMUS_COLLISION_DIRECTION) & 2 != 0 {
            return Err("quicksand setup requires actual movement context".into());
        }
        if !PLM_SETUP.contains(&setup) {
            return Err("unknown actual C PLM setup dispatch".into());
        }
    }
    host.spawn_plm(r.0, header)?;
    Ok(())
}
#[derive(Default)]
struct Shot {
    span: u16,
    edge: u16,
    remaining: u16,
    hits: u16,
}
fn square(r: &Wram<'_>, ci: &Shot, b: u8, vertical: bool) -> Result<bool> {
    let k = slot(r.word(INDEX))?;
    let shape = (b & 31) as usize;
    let flips = (b >> 6) as usize;
    let i = 4 * shape + (flips ^ ((ci.edge & 8) as usize >> if vertical { 2 } else { 3 }));
    let (position, radius, bit) =
        if vertical { (r.word(X + k), r.word(RADIUS_X + k), 1) } else { (r.word(Y + k), r.word(RADIUS_Y + k), 2) };
    if i >= 20 {
        return Err("square slope table extent".into());
    }
    let min = position.wrapping_sub(radius);
    let max = (position as u8 as u16).wrapping_add(radius as u8 as u16).wrapping_sub(1);
    if ci.span == 0 {
        return Ok(!((min & 8 != 0 || SQUARE[i] == 0) && (max & 8 == 0 || SQUARE[i ^ bit] == 0)));
    }
    if ci.remaining != 0 {
        if ci.remaining == ci.span && min & 8 != 0 {
            return Ok(SQUARE[i ^ bit] != 0);
        }
    } else if max & 8 == 0 {
        return Ok(SQUARE[i] != 0);
    }
    Ok(SQUARE[i] != 0 || SQUARE[i ^ bit] != 0)
}
fn slope(r: &Wram<'_>, ci: &mut Shot, k: u16, b: u8, vertical: bool) -> Result<bool> {
    if b & 31 < 5 {
        return square(r, ci, b, vertical);
    }
    let p = slot(r.word(INDEX))?;
    let width = r.word(WIDTH);
    let aligned = if vertical { r.word(X + p) >> 4 == k % width } else { r.word(Y + p) >> 4 == k / width };
    if !aligned {
        return Ok(false);
    }
    let x = (r.word(X + p) & 15) ^ if b & 64 != 0 { 15 } else { 0 };
    let y = (r.word(Y + p) & 15) ^ if b & 128 != 0 { 15 } else { 0 };
    if i16::from(SLOPE[16 * (b & 31) as usize + x as usize] & 31) - y as i16 <= 0 {
        ci.remaining = 0;
        ci.hits = 0;
        Ok(true)
    } else {
        Ok(false)
    }
}
fn reaction(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    ci: &mut Shot,
    byte_index: u16,
    vertical: bool,
) -> Result<u8> {
    if byte_index >= r.word(ROOM_BYTES) {
        return Ok(1);
    }
    let mut index = byte_index >> 1;
    r.set_word(BLOCK, index);
    for _ in 0..256 {
        let (tile, b) = block(r, index)?;
        let kind = tile >> 12;
        let hit = match kind {
            0 | 2 | 3 | 6 => false,
            1 => slope(r, ci, index, b, vertical)?,
            5 | 13 => {
                if b == 0 {
                    return Ok(0);
                }
                let delta =
                    if kind == 5 { i16::from(b as i8) } else { i16::from(b as i8).wrapping_mul(r.word(WIDTH) as i16) };
                index = index.wrapping_add(delta as u16);
                block(r, index)?;
                r.set_word(BLOCK, index);
                continue;
            }
            4 | 12 => {
                if b & 128 != 0 {
                    if kind == 12 {
                        let area = r.word(AREA);
                        if area >= 8 {
                            return Err("regional shot PLM area".into());
                        }
                        let table = rw(rom, 0x949fc6 + 2 * u32::from(area))?;
                        let header = ptr(rom, 0x940000, table, 2 * u16::from(b & 127))?;
                        spawn(r, rom, host, header)?;
                    }
                } else {
                    let header = rw(rom, 0x949ea6 + 2 * u32::from(b))?;
                    if header == 0xb974 {
                        ci.remaining = 0;
                        ci.hits = 65535;
                    }
                    spawn(r, rom, host, header)?;
                }
                kind == 12
            }
            7 | 15 => {
                if b & 128 == 0 {
                    let header = rw(rom, 0x94a012 + 2 * u32::from(b))?;
                    spawn(r, rom, host, header)?;
                }
                kind == 15
            }
            8 | 9 | 10 | 11 | 14 => true,
            _ => unreachable!(),
        };
        if hit {
            ci.hits = ci.hits.wrapping_sub(1);
        }
        return Ok(u8::from(hit));
    }
    Err("cyclic shot tile extension".into())
}
fn move_axis(r: &mut Wram<'_>, position: usize, sub: usize, amount: i32) {
    let full = (u32::from(r.word(position)) << 16) | u32::from(r.word(sub));
    let next = full.wrapping_add(amount as u32);
    r.set_word(position, (next >> 16) as u16);
    r.set_word(sub, next as u16);
}
/// Move an ordinary beam and apply every traversed tile.
pub(crate) fn collide(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host, k: u16, vertical: bool) -> Result<u8> {
    room(r)?;
    let p = slot(k)?;
    slot(r.word(INDEX))?;
    let (position, sub, speed, radius, cross, cross_radius, limit) = if vertical {
        (Y, SUB_Y, SPEED_Y, RADIUS_Y, X, RADIUS_X, HEIGHT_SCROLLS)
    } else {
        (X, SUB_X, SPEED_X, RADIUS_X, Y, RADIUS_Y, WIDTH_SCROLLS)
    };
    let amount = i32::from(r.word(speed + p) as i16) * 256;
    let start = r.word(cross + p).wrapping_sub(r.word(cross_radius + p)) & 0xfff0;
    let span = r.word(cross_radius + p).wrapping_add(r.word(cross + p)).wrapping_sub(1).wrapping_sub(start) >> 4;
    let initial_cross = r.word(cross + p).wrapping_sub(r.word(cross_radius + p)) >> 4;
    let prod = (initial_cross as u8 as u16) * (r.word(WIDTH) as u8 as u16);
    move_axis(r, position + p, sub + p, amount);
    let center = r.word(position + p);
    let edge = if amount < 0 {
        center.wrapping_sub(r.word(radius + p))
    } else {
        center.wrapping_add(r.word(radius + p)).wrapping_sub(1)
    };
    let index = if vertical {
        2u16.wrapping_mul((((edge >> 4) as u8 as u16) * (r.word(WIDTH) as u8 as u16)).wrapping_add(initial_cross))
    } else {
        2u16.wrapping_mul(prod.wrapping_add(edge >> 4))
    };
    if span.wrapping_sub(16) & 0x8000 == 0 {
        return Ok(0);
    }
    let outside = (edge >> 8) as i16 - r.word(limit) as i16 >= 0;
    if outside {
        return Ok(0);
    }
    let mut ci = Shot { span, edge, remaining: span, hits: span };
    let mut index = index;
    loop {
        reaction(r, rom, host, &mut ci, index, vertical)?;
        index = index.wrapping_add(if vertical { 2 } else { 2 * r.word(WIDTH) });
        ci.remaining = ci.remaining.wrapping_sub(1);
        if ci.remaining & 0x8000 != 0 {
            break;
        }
    }
    if ci.hits & 0x8000 != 0 {
        projectile_core::kill(r, rom, k)?;
        return Ok(1);
    }
    Ok(0)
}
