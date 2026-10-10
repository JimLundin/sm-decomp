//! Normal Samus projectile flight, shot terrain reactions and firing.
//! Pinned snesrev/sm banks $90/$94/$9B/$8B; upstream MIT notice applies.
use crate::{Wram, audio};
#[path = "projectiles.rs"]
mod projectile_core;
pub trait Host {
    fn spawn_plm(&mut self, ram: &mut Wram<'_>, header: u16) -> Result<u8>;
}
const AREA: usize = 0x79f;
const PLM_HEADERS: usize = 0x1c37;
const CERES_STATUS: usize = 0x93f;
const MODE7_A: usize = 0x78;
const MODE7_B: usize = 0x7a;
const MODE7_C: usize = 0x7c;
const MODE7_X: usize = 0x80;
const MODE7_Y: usize = 0x82;
const CAMERA_X: usize = 0x911;
const CAMERA_Y: usize = 0x915;
const TRAIL_LEFT_TIMER: usize = 0xd658;
const TRAIL_RIGHT_TIMER: usize = 0xd67c;
const TRAIL_LEFT_LIST: usize = 0xd6a0;
const TRAIL_RIGHT_LIST: usize = 0xd6c4;
const TRAIL_LEFT_X: usize = 0xd730;
const TRAIL_RIGHT_X: usize = 0xd754;
const TRAIL_LEFT_Y: usize = 0xd778;
const TRAIL_RIGHT_Y: usize = 0xd79c;
const EQUIPPED_BEAMS: usize = 0x9a6;
const PREVIOUS_CHARGE: usize = 0xdc2;
const RESUME_CHARGE_SOUND: usize = 0xdc0;
const SHOOT_BUTTON: usize = 0x9b2;
const NEW_KEYS: usize = 0x8f;
const FILTERED_NEW_KEYS: usize = 0xe00;
const COOLDOWN: usize = 0xccc;
const PROJECTILE_COUNT: usize = 0xcce;
const PROJECTILE_INVINCIBILITY: usize = 0x18ac;
const HUD_ITEM: usize = 0x9d2;
const SUPER_MISSILES: usize = 0x9ca;
const MISSILES: usize = 0x9c6;
const AUTO_CANCEL_HUD: usize = 0xa04;
const CINEMATIC_FUNCTION: usize = 0x1f51;
const HYPER_BEAM: usize = 0xa76;
const CHARGED_GLOW: usize = 0xb18;
const FLARE_COUNTER: usize = 0xcd0;
const FLARE_FRAME: usize = 0xcd6;
const FLARE_SLOW_FRAME: usize = 0xcd8;
const FLARE_FAST_FRAME: usize = 0xcda;
const FLARE_TIMER: usize = 0xcdc;
const FLARE_SLOW_TIMER: usize = 0xcde;
const FLARE_FAST_TIMER: usize = 0xce0;
const SAMUS_COLLISION_DIRECTION: usize = 0xb02;
type Result<T> = std::result::Result<T, &'static str>;
const X: usize = 0xb64;
const Y: usize = 0xb78;
const SUB_X: usize = 0xb8c;
const SUB_Y: usize = 0xba0;
const RADIUS_X: usize = 0xbb4;
const RADIUS_Y: usize = 0xbc8;
const SPEED_X: usize = 0xbdc;
const SPEED_Y: usize = 0xbf0;
const DIR: usize = 0xc04;
const TYPE: usize = 0xc18;
const DAMAGE: usize = 0xc2c;
const VARIABLE: usize = 0xc7c;
const INDEX: usize = 0xdde;
const TIMERS: usize = 0xc90;
const PRE: usize = 0xc68;
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
fn slot(k: u16) -> Result<usize> {
    if k > 18 || k & 1 != 0 {
        return Err("canonical projectile slot");
    }
    Ok(k as usize)
}
fn direction(r: &Wram<'_>, k: usize) -> Result<usize> {
    let d = (r.word(DIR + k) & 15) as usize;
    if d >= 10 {
        return Err("projectile direction");
    }
    Ok(d)
}
fn rb(rom: &[u8], a: u32) -> Result<u8> {
    if a & 65535 < 0x8000 {
        return Err("LoROM window");
    }
    let i = (((a >> 16) & 127) * 32768 + (a & 32767)) as usize;
    rom.get(i).copied().ok_or("ROM extent")
}
fn rw(rom: &[u8], a: u32) -> Result<u16> {
    if a & 65535 == 65535 {
        return Err("ROM bank word extent");
    }
    Ok(u16::from_le_bytes([rb(rom, a)?, rb(rom, a + 1)?]))
}
fn ptr(rom: &[u8], bank: u32, p: u16, delta: u16) -> Result<u16> {
    rw(rom, bank | u32::from(p.checked_add(delta).ok_or("ROM pointer wrap")?))
}
fn room(r: &Wram<'_>) -> Result<()> {
    let w = r.word(WIDTH);
    let h = r.word(HEIGHT);
    let size = r.word(ROOM_BYTES);
    if w == 0 || h == 0 || w > 255 || h > 255 || size == 0 || size & 1 != 0 || size > 0x6400 {
        return Err("room geometry/backing extent");
    }
    Ok(())
}
fn block(r: &Wram<'_>, k: u16) -> Result<(u16, u8)> {
    if k >= 0x3200 {
        return Err("tile backing extent");
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
            return Err("quicksand setup requires actual movement context");
        }
        if !PLM_SETUP.contains(&setup) {
            return Err("unknown actual C PLM setup dispatch");
        }
    }
    host.spawn_plm(r, header)?;
    Ok(())
}
#[derive(Default)]
struct Shot {
    span: u16,
    edge: u16,
    missile: bool,
    remaining: u16,
    hits: u16,
}
fn square(r: &Wram<'_>, ci: &Shot, b: u8, vertical: bool) -> Result<bool> {
    let k = slot(r.word(INDEX))?;
    let shape = (b & 31) as usize;
    let flips = (b >> 6) as usize;
    let mut i = 4 * shape + (flips ^ ((ci.edge & 8) as usize >> if vertical { 2 } else { 3 }));
    let (position, radius, bit) =
        if vertical { (r.word(X + k), r.word(RADIUS_X + k), 1) } else { (r.word(Y + k), r.word(RADIUS_Y + k), 2) };
    if i >= 20 {
        return Err("square slope table extent");
    }
    if ci.missile {
        if position & 8 != 0 {
            i ^= bit;
        }
        return Ok(SQUARE[i] != 0);
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
                            return Err("regional shot PLM area");
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
    Err("cyclic shot tile extension")
}
fn move_axis(r: &mut Wram<'_>, position: usize, sub: usize, amount: i32) {
    let full = (u32::from(r.word(position)) << 16) | u32::from(r.word(sub));
    let next = full.wrapping_add(amount as u32);
    r.set_word(position, (next >> 16) as u16);
    r.set_word(sub, next as u16);
}
/// Move a normal beam, wave beam or missile and apply every traversed tile.
pub fn collide(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    k: u16,
    vertical: bool,
    wave: bool,
    missile: bool,
) -> Result<u8> {
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
    let edge = if missile {
        center
    } else if amount < 0 {
        center.wrapping_sub(r.word(radius + p))
    } else {
        center.wrapping_add(r.word(radius + p)).wrapping_sub(1)
    };
    let index = if vertical {
        2u16.wrapping_mul((((edge >> 4) as u8 as u16) * (r.word(WIDTH) as u8 as u16)).wrapping_add(if missile {
            r.word(X + p) >> 4
        } else {
            initial_cross
        }))
    } else {
        2u16.wrapping_mul(if missile {
            (((r.word(Y + p) >> 4) as u8 as u16) * (r.word(WIDTH) as u8 as u16)).wrapping_add(center >> 4)
        } else {
            prod.wrapping_add(edge >> 4)
        })
    };
    if !missile && span.wrapping_sub(16) & 0x8000 == 0 {
        return Ok(0);
    }
    let outside = (edge >> 8) as i16 - r.word(limit) as i16 >= 0;
    if !wave && outside {
        return Ok(0);
    }
    if wave {
        let cross_hi = (r.word(cross + p) >> 8) as u8;
        if (cross_hi as i8) < 0
            || i16::from(cross_hi) - r.word(if vertical { WIDTH_SCROLLS } else { HEIGHT_SCROLLS }) as i16 >= 0
            || outside
        {
            return Ok(0);
        }
    }
    let mut ci = Shot {
        span: if missile { 0 } else { span },
        edge,
        missile,
        remaining: if missile { 0 } else { span },
        hits: if missile { 0 } else { span },
    };
    let mut index = index;
    loop {
        let hit = reaction(r, rom, host, &mut ci, index, vertical)?;
        if missile {
            if hit != 0 {
                projectile_core::kill(r, rom, k)?;
                return Ok(1);
            }
            return Ok(0);
        }
        index = index.wrapping_add(if vertical { 2 } else { 2 * r.word(WIDTH) });
        ci.remaining = ci.remaining.wrapping_sub(1);
        if ci.remaining & 0x8000 != 0 {
            break;
        }
    }
    if !wave && ci.hits & 0x8000 != 0 {
        projectile_core::kill(r, rom, k)?;
        return Ok(1);
    }
    Ok(0)
}
fn mode7(r: &Wram<'_>, k: usize) -> (u16, u16) {
    fn product(a: u16, b: u16) -> u16 {
        ((i32::from(a as i16) * i32::from(b as i16)) >> 8) as u16
    }
    let dx = r.word(X + k).wrapping_sub(r.word(MODE7_X));
    let dy = r.word(MODE7_Y).wrapping_sub(r.word(Y + k));
    let x = product(dx, r.word(MODE7_A))
        .wrapping_add(product(r.word(MODE7_B), dy))
        .wrapping_add(r.word(MODE7_X))
        .wrapping_sub(r.word(CAMERA_X));
    let y = r
        .word(0x82)
        .wrapping_sub(product(r.word(MODE7_C), dx).wrapping_add(product(r.word(MODE7_A), dy)))
        .wrapping_sub(r.word(CAMERA_Y));
    (x, y)
}
/// Trail coordinates include native Ceres Mode7 arithmetic and ROM trail frames.
pub fn place_trail(r: &mut Wram<'_>, rom: &[u8], k: u16, j: u16) -> Result<()> {
    let p = slot(k)?;
    if j > 34 || j & 1 != 0 {
        return Err("trail slot");
    }
    let d = direction(r, p)?;
    let value = projectile_core::dispatch_rom(r, rom, "ProjectileInsts_GetValue", i32::from(k), 0)? as u16;
    let (x, y) = if r.word(CERES_STATUS) & 0x8000 == 0 {
        (r.word(X + p), r.word(Y + p))
    } else {
        let (x, y) = mode7(r, p);
        (r.word(CAMERA_X).wrapping_add(x), r.word(CAMERA_Y).wrapping_add(y))
    };
    let t = r.word(TYPE + p);
    let ty = t & 15;
    if ty > 11 {
        return Err("trail type table index");
    }
    let table = if t & 32 != 0 {
        0x9ba4e3
    } else if t & 16 != 0 {
        0x9ba4cb
    } else {
        0x9ba4b3
    };
    let list = rw(rom, table + 2 * u32::from(ty))?;
    let list = list.checked_add(2 * d as u16).ok_or("trail direction pointer")?;
    let base = ptr(rom, 0x9b0000, list, 0)?;
    let frame = base.checked_add(value.checked_mul(4).ok_or("trail frame multiply")?).ok_or("trail frame extent")?;
    let mut delta = [0i8; 4];
    for (i, v) in delta.iter_mut().enumerate() {
        *v = rb(rom, 0x9b0000 | u32::from(frame.checked_add(i as u16).ok_or("trail bytes")?))? as i8;
    }
    let j = j as usize;
    r.set_word(TRAIL_LEFT_Y + j, y.wrapping_add(delta[1] as u16).wrapping_sub(4));
    r.set_word(TRAIL_LEFT_X + j, x.wrapping_add(delta[0] as u16).wrapping_sub(4));
    r.set_word(TRAIL_RIGHT_Y + j, y.wrapping_add(delta[3] as u16).wrapping_sub(4));
    r.set_word(TRAIL_RIGHT_X + j, x.wrapping_add(delta[2] as u16).wrapping_sub(4));
    Ok(())
}
pub fn spawn_trail(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<()> {
    let p = slot(k)?;
    let t = r.word(TYPE + p);
    let index = if t & 0xf00 != 0 {
        let n = (t >> 8) & 15;
        if n >= 3 {
            return Ok(());
        }
        n + 31
    } else {
        t & 63
    };
    if index > 38 {
        return Err("trail list table index");
    }
    let Some(j) = (0..36).step_by(2).rev().find(|j| r.word(TRAIL_LEFT_TIMER + j) == 0) else {
        return Ok(());
    };
    let left = rw(rom, 0x90b5bb + 2 * u32::from(index))?;
    let right = rw(rom, 0x90b609 + 2 * u32::from(index))?;
    // Placement consumes projectile_index, deliberately not the caller's k.
    slot(r.word(INDEX))?;
    r.set_word(TRAIL_LEFT_TIMER + j, 1);
    r.set_word(TRAIL_RIGHT_TIMER + j, 1);
    r.set_word(TRAIL_LEFT_LIST + j, left);
    r.set_word(TRAIL_RIGHT_LIST + j, right);
    place_trail(r, rom, r.word(INDEX), j as u16)
}
fn dir_collision(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    wave: bool,
    missile: bool,
    d: usize,
) -> Result<()> {
    let k = r.word(INDEX);
    if matches!(d, 1 | 3 | 6 | 8) {
        if collide(r, rom, host, k, false, wave, missile)? == 0 {
            collide(r, rom, host, k, true, wave, missile)?;
        }
    } else {
        collide(r, rom, host, k, !matches!(d, 2 | 7), wave, missile)?;
    }
    Ok(())
}
fn super_link_axis(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host, vertical: bool) -> Result<()> {
    let raw = r.word(INDEX);
    let p = slot(raw)?;
    let t = r.word(TYPE + p) & 0xf00;
    if t != 0x200 && t != 0x800 {
        return Ok(());
    }
    let v = r.word(VARIABLE + p);
    if v & 0xff00 == 0 {
        return Ok(());
    }
    let linked = v as u8 as u16;
    let l = slot(linked)?;
    let axis = if vertical { Y } else { X };
    let speed = if vertical { SPEED_Y } else { SPEED_X };
    let absolute =
        if r.word(speed + p) & 0x8000 != 0 { 0u16.wrapping_sub(r.word(speed + p)) } else { r.word(speed + p) } & 0xff00;
    if absolute.wrapping_sub(2816) & 0x8000 != 0 {
        if t != 0x800 {
            r.set_word(axis + l, r.word(axis + p));
            return Ok(());
        }
    } else {
        let offset = (absolute >> 8).wrapping_sub(10);
        r.set_word(
            axis + l,
            if r.word(speed + p) & 0x8000 != 0 {
                r.word(axis + p).wrapping_add(offset)
            } else {
                r.word(axis + p).wrapping_sub(offset)
            },
        );
        r.set_word(INDEX, linked);
        collide(r, rom, host, linked, vertical, false, true)?;
        r.set_word(INDEX, raw);
        if r.word(TYPE + p) & 0xf00 != 0x800 {
            return Ok(());
        }
    }
    projectile_core::clear(r, linked)
}
pub fn pre_instruction(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host, k: u16, kind: u16) -> Result<()> {
    let p = slot(k)?;
    let super_missile = kind == 0xafe5;
    if r.word(DIR + p) & 0xf0 != 0 {
        projectile_core::clear(r, k)?;
        if super_missile {
            for j in (0..10).step_by(2).rev() {
                if r.word(TYPE + j) & 0xfff == 512 {
                    projectile_core::clear(r, j as u16)?;
                }
            }
        }
        return Ok(());
    }
    let mut k = k;
    if kind != 0xb159 {
        let timer = r.word(TIMERS + p);
        r.set_word(TIMERS + p, timer.wrapping_sub(1));
        if timer == 1 {
            r.set_word(
                TIMERS + p,
                if super_missile {
                    2
                } else if kind == 0xb0e4 {
                    3
                } else {
                    4
                },
            );
            spawn_trail(r, rom, k)?;
            k = r.word(INDEX);
        }
    }
    let p = slot(k)?;
    let d = direction(r, p)?;
    let missile = kind == 0xaf68 || super_missile;
    let wave = matches!(kind, 0xb0c3 | 0xb0e4 | 0xb159);
    if !super_missile {
        let dx = [0, 16, 16, 16, 0, 0, -16, -16, -16, 0][d];
        let dy = [-16, -16, 0, 16, 16, 16, 16, 0, -16, -16][d];
        r.set_word(SPEED_X + p, r.word(SPEED_X + p).wrapping_add(dx as u16));
        r.set_word(SPEED_Y + p, r.word(SPEED_Y + p).wrapping_add(dy as u16));
    }
    if missile {
        projectile_core::missile_accelerate(r, rom, k)?;
    }
    if super_missile {
        let raw = r.word(INDEX);
        if matches!(d, 1 | 3 | 6 | 8) {
            let hit = collide(r, rom, host, raw, false, false, true)?;
            super_link_axis(r, rom, host, false)?;
            if hit == 0 {
                collide(r, rom, host, raw, true, false, true)?;
                super_link_axis(r, rom, host, true)?;
            }
        } else {
            let vertical = !matches!(d, 2 | 7);
            collide(r, rom, host, raw, vertical, false, true)?;
            super_link_axis(r, rom, host, vertical)?;
        }
    } else {
        dir_collision(r, rom, host, wave, missile, d)?;
    }
    let deleted = projectile_core::dispatch_rom(r, rom, "DeleteProjectileIfFarOffScreen", 0, 0)?;
    if super_missile && deleted != 0 {
        for j in (0..10).step_by(2).rev() {
            if r.word(TYPE + j) & 0xfff == 512 {
                projectile_core::clear(r, j as u16)?;
            }
        }
    }
    Ok(())
}
const UNCHARGED_SFX: [u16; 12] = [11, 13, 12, 14, 15, 18, 16, 17, 19, 22, 20, 21];
const CHARGED_SFX: [u16; 12] = [23, 25, 24, 26, 27, 30, 28, 29, 31, 34, 32, 33];
const COOLDOWNS: [u16; 38] = [
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 12, 15, 0, 0, 0, 0, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
];
fn initial_collision(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, k: u16, wave: bool) -> Result<()> {
    let p = slot(k)?;
    let d = direction(r, p)?;
    r.set_word(INDEX, k);
    if d == 7 {
        r.set_word(SPEED_X + p, 65535);
    }
    dir_collision(r, rom, h, wave, false, d)
}
pub fn fire_beam(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, charged: bool, hyper: bool) -> Result<()> {
    if projectile_core::dispatch_rom(r, rom, "Samus_CanFireBeam", 0, 0)? == 0 {
        if !hyper && r.word(PREVIOUS_CHARGE).wrapping_sub(16) & 0x8000 == 0 {
            r.set_word(RESUME_CHARGE_SOUND, 0);
            audio::dispatch(r, "QueueSfx1_Max15", 2, 0)?;
        }
        return Ok(());
    }
    let k = (0..10).step_by(2).find(|k| r.word(DAMAGE + k) == 0).unwrap_or(8) as u16;
    if projectile_core::init_position(r, rom, k)? != 0 {
        if !hyper && r.word(PREVIOUS_CHARGE).wrapping_sub(16) & 0x8000 == 0 {
            r.set_word(RESUME_CHARGE_SOUND, 0);
            audio::dispatch(r, "QueueSfx1_Max15", 2, 0)?;
        }
        return Ok(());
    }
    let p = slot(k)?;
    let equipped = r.word(EQUIPPED_BEAMS);
    let t = if hyper {
        0x9018
    } else if charged {
        equipped & 0x100f | 0x8010
    } else {
        equipped | 0x8000
    };
    let ty = (t & 15) as usize;
    if ty >= 12 {
        return Err("fired beam type");
    }
    r.set_word(PROJECTILE_INVINCIBILITY, 10);
    if !hyper {
        r.set_word(TIMERS + p, 4);
    }
    r.set_word(TYPE + p, t);
    audio::dispatch(r, "QueueSfx1_Max15", if charged || hyper { CHARGED_SFX[ty] } else { UNCHARGED_SFX[ty] }, 0)?;
    r.set_word(RESUME_CHARGE_SOUND, 0);
    projectile_core::initialize(r, rom, k)?;
    if !hyper {
        let automatic = !charged
            && equipped & 0x1000 == 0
            && r.word(SHOOT_BUTTON) & r.word(NEW_KEYS) == 0
            && r.word(SHOOT_BUTTON) & r.word(FILTERED_NEW_KEYS) == 0;
        let index = (t & 63) as usize;
        if index >= 38 || automatic && index >= 12 {
            return Err("beam cooldown table index");
        }
        r.set_word(COOLDOWN, if automatic { 25 } else { COOLDOWNS[index] });
    }
    r.set_word(SPEED_X + p, 0);
    r.set_word(SPEED_Y + p, 0);
    initial_collision(r, rom, h, k, t & 1 != 0 || hyper)?;
    let k = r.word(INDEX);
    let p = slot(k)?;
    if hyper {
        r.set_word(DAMAGE + p, 1000);
        r.set_word(PRE + p, 0xb159);
        projectile_core::dispatch_rom(r, rom, "SetInitialProjectileSpeed", i32::from(k), 0)?;
        r.set_word(COOLDOWN, 21);
        r.set_word(CHARGED_GLOW, 0x8014);
        r.set_word(FLARE_FRAME, 29);
        r.set_word(FLARE_SLOW_FRAME, 5);
        r.set_word(FLARE_FAST_FRAME, 5);
        r.set_word(FLARE_TIMER, 3);
        r.set_word(FLARE_SLOW_TIMER, 3);
        r.set_word(FLARE_FAST_TIMER, 3);
        r.set_word(FLARE_COUNTER, 0x8000);
        return Ok(());
    }
    if r.word(TYPE + p) & 0xf00 == 0 {
        let ty = (r.word(TYPE + p) & 15) as usize;
        if ty >= 12 {
            return Err("beam pre-instruction table");
        }
        r.set_word(
            PRE + p,
            if r.word(TYPE + p) & 1 == 0 {
                0xaef3
            } else if charged || ty >= 5 {
                0xb0c3
            } else {
                0xb0e4
            },
        );
        projectile_core::dispatch_rom(r, rom, "SetInitialProjectileSpeed", i32::from(k), 0)?;
    }
    if charged {
        r.set_word(CHARGED_GLOW, 4);
    }
    Ok(())
}
pub fn fire_missile(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host) -> Result<()> {
    let _ = h;
    if r.word(SHOOT_BUTTON) & r.word(NEW_KEYS) == 0 && r.word(SHOOT_BUTTON) & r.word(FILTERED_NEW_KEYS) == 0 {
        return Ok(());
    }
    if projectile_core::dispatch_rom(r, rom, "Samus_CanFireSuperMissile", 0, 0)? == 0 {
        return Ok(());
    }
    let hud = r.word(HUD_ITEM);
    let ammo = if hud == 2 { SUPER_MISSILES } else { MISSILES };
    let free = (0..10).step_by(2).find(|k| r.word(DAMAGE + k) == 0);
    if r.word(ammo) == 0 || free.is_none() {
        r.set_word(PROJECTILE_COUNT, r.word(PROJECTILE_COUNT).wrapping_sub(1));
        return Ok(());
    }
    if hud != 1 && hud != 2 {
        return Err("missile HUD type");
    }
    let k = free.unwrap() as u16;
    if projectile_core::init_position(r, rom, k)? != 0 {
        return Ok(());
    }
    let p = slot(k)?;
    r.set_word(PROJECTILE_INVINCIBILITY, 20);
    r.set_word(ammo, r.word(ammo).wrapping_sub(1));
    r.set_word(TIMERS + p, 4);
    r.set_word(TYPE + p, r.word(TYPE + p) | (hud << 8) | 0x8000);
    if r.word(CINEMATIC_FUNCTION) == 0 {
        audio::dispatch(r, "QueueSfx1_Max6", if hud == 2 { 4 } else { 3 }, 0)?;
    }
    projectile_core::initialize_speed(r, k, 0)?;
    projectile_core::initialize(r, rom, k)?;
    let ty = (r.word(TYPE + p) >> 8) & 15;
    if ty > 8 {
        return Err("missile cooldown table");
    }
    r.set_word(PRE + p, if r.word(TYPE + p) & 0x200 != 0 { 0xafe5 } else { 0xaf68 });
    r.set_word(COOLDOWN, [0, 10, 20, 40, 0, 16, 0, 0, 0][ty as usize]);
    if r.word(AUTO_CANCEL_HUD) != 0 {
        r.set_word(HUD_ITEM, 0);
        r.set_word(AUTO_CANCEL_HUD, 0);
    } else if r.word(ammo) == 0 {
        r.set_word(HUD_ITEM, 0);
    }
    Ok(())
}
pub fn dispatch_host(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, symbol: &str, a: i32, b: i32) -> Result<u32> {
    let _ = b;
    let k = a as u16;
    match symbol {
        "BlockCollNoWaveBeamHoriz" => return Ok(u32::from(collide(r, rom, h, k, false, false, false)?)),
        "BlockCollNoWaveBeamVert" => return Ok(u32::from(collide(r, rom, h, k, true, false, false)?)),
        "BlockCollWaveBeamHoriz" => return Ok(u32::from(collide(r, rom, h, k, false, true, false)?)),
        "BlockCollWaveBeamVert" => return Ok(u32::from(collide(r, rom, h, k, true, true, false)?)),
        "BlockCollMissileHoriz" => return Ok(u32::from(collide(r, rom, h, k, false, false, true)?)),
        "BlockCollMissileVert" => return Ok(u32::from(collide(r, rom, h, k, true, false, true)?)),
        "SpawnProjectileTrail" => spawn_trail(r, rom, k)?,
        "ProjectileTrail_Func5" => place_trail(r, rom, k, b as u16)?,
        "SuperMissileBlockCollDetect_X" => super_link_axis(r, rom, h, false)?,
        "SuperMissileBlockCollDetect_Y" => super_link_axis(r, rom, h, true)?,
        "ProjPreInstr_Beam_NoWaveBeam" => pre_instruction(r, rom, h, k, 0xaef3)?,
        "ProjPreInstr_Missile" => pre_instruction(r, rom, h, k, 0xaf68)?,
        "ProjPreInstr_SuperMissile" => pre_instruction(r, rom, h, k, 0xafe5)?,
        "ProjPreInstr_WavePlasmaEtc" => pre_instruction(r, rom, h, k, 0xb0c3)?,
        "ProjPreInstr_BeamOrIceWave" => pre_instruction(r, rom, h, k, 0xb0e4)?,
        "ProjPreInstr_HyperBeam" => pre_instruction(r, rom, h, k, 0xb159)?,
        "FireUnchargedBeam" => fire_beam(r, rom, h, false, r.word(HYPER_BEAM) != 0)?,
        "FireChargedBeam" => fire_beam(r, rom, h, true, false)?,
        "FireHyperBeam" => fire_beam(r, rom, h, true, true)?,
        "HudSelectionHandler_MissilesOrSuperMissiles" => fire_missile(r, rom, h)?,
        "CheckBeamCollByDir" | "WaveBeam_CheckColl" => {
            let d = direction(r, slot(k)?)?;
            let wave = symbol == "WaveBeam_CheckColl";
            if d == 7 {
                let p = slot(r.word(INDEX))?;
                r.set_word(SPEED_X + p, 65535);
            }
            dir_collision(r, rom, h, wave, false, d)?;
        }
        name if name.starts_with("ProjPreInstr_Dir")
            || name.starts_with("ProjPreInstr_Missile_Func")
            || name.starts_with("ProjPreInstr_WavePlasmaEtc_")
            || name.starts_with("ProjPreInstr_SuperMissile_Func") =>
        {
            let suffix = name.rsplit('_').next().unwrap();
            let d = if suffix.ends_with("0459") {
                0
            } else if suffix.ends_with("1368") {
                1
            } else if suffix.ends_with("27") {
                2
            } else {
                return Err("unknown projectile directional wrapper");
            };
            let wave = name.starts_with("ProjPreInstr_WavePlasmaEtc_");
            let missile = name.contains("Missile");
            if name.starts_with("ProjPreInstr_SuperMissile_Func") {
                let raw = r.word(INDEX);
                if d == 1 {
                    let hit = collide(r, rom, h, raw, false, false, true)?;
                    super_link_axis(r, rom, h, false)?;
                    if hit == 0 {
                        collide(r, rom, h, raw, true, false, true)?;
                        super_link_axis(r, rom, h, true)?;
                    }
                } else {
                    let vertical = d == 0;
                    collide(r, rom, h, raw, vertical, false, true)?;
                    super_link_axis(r, rom, h, vertical)?;
                }
            } else {
                dir_collision(r, rom, h, wave, missile, d)?;
            }
        }
        name if name.starts_with("CheckBeamCollByDir_") || name.starts_with("WaveBeam_CheckColl_") => {
            let suffix = name.rsplit('_').next().unwrap();
            let d = match suffix {
                "0459" => 0,
                "1368" => 1,
                "2" => 2,
                "7" => 7,
                _ => return Err("unknown initial collision wrapper"),
            };
            if d == 7 {
                let p = slot(r.word(INDEX))?;
                r.set_word(SPEED_X + p, 65535);
            }
            dir_collision(r, rom, h, name.starts_with("WaveBeam_"), false, d)?;
        }
        _ => return Err("unknown projectile flight routine"),
    }
    Ok(0)
}
