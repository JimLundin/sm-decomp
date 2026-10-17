//! Bomb terrain reactions and special beam attacks from pinned snesrev/sm.
//! Upstream MIT notice applies; HDMA power-bomb lifetime remains external.
use crate::{Wram, audio, projectile_flight, projectiles};
pub trait Host {
    fn spawn_plm(&mut self, ram: &mut Wram<'_>, header: u16) -> Result<u8>;
}
const HEIGHT: usize = 0x7a7;
const WIDTH_SCROLLS: usize = 0x7a9;
const HEIGHT_SCROLLS: usize = 0x7ab;
const AREA: usize = 0x79f;
const PLM_HEADERS: usize = 0x1c37;
const COLLISION_DIRECTION: usize = 0xb02;
const EQUIPPED_BEAMS: usize = 0x9a6;
const HUD_ITEM: usize = 0x9d2;
const POWER_BOMBS: usize = 0x9ce;
const AUTO_CANCEL_HUD: usize = 0xa04;
const PROJECTILE_COUNT: usize = 0xcce;
const BOMB_COUNT: usize = 0xcd2;
const SPREAD_CHARGE: usize = 0xcd4;
const GRAVITY_INTEGER: usize = 0xb34;
const GRAVITY_FRACTION: usize = 0xb32;
const SAMUS_FACING: usize = 0xa1e;
const ECHO_INDEX: usize = 0xaae;
const ECHO_Y_ALIAS: usize = 0xab6;
const ECHO_CLEAR_ALIAS: usize = 0xabe;
const MOVEMENT_HANDLER: usize = 0xa58;
const SHINE_TIMER: usize = 0xa68;
const SHINE_TIMER_PHASE: usize = 0xacc;
const PALETTE_FRAME: usize = 0xace;
type Result<T> = std::result::Result<T, &'static str>;
const X: usize = 0xb64;
const Y: usize = 0xb78;
const SUB_X: usize = 0xb8c;
const SUB_Y: usize = 0xba0;
const YR: usize = 0xbc8;
const VX: usize = 0xbdc;
const VY: usize = 0xbf0;
const DIR: usize = 0xc04;
const TYPE: usize = 0xc18;
const VARIABLE: usize = 0xc7c;
const TIMER: usize = 0xc90;
const PRE: usize = 0xc68;
const AUX: usize = 0xca4;
const INDEX: usize = 0xdde;
const SAMUS_X: usize = 0xaf6;
const SAMUS_Y: usize = 0xafa;
const CAMERA_X: usize = 0x911;
const CAMERA_Y: usize = 0x915;
const COOLDOWN: usize = 0xccc;
const FLARE: usize = 0xcd0;
const ROTATION: usize = 0xb60;
const WIDTH: usize = 0x7a5;
const BLOCK: usize = 0xdc4;
const LEVEL: usize = 0x10002;
const BTS: usize = 0x16402;
fn slot(k: u16) -> Result<usize> {
    if k > 18 || k & 1 != 0 {
        return Err("projectile slot");
    }
    Ok(k as usize)
}
fn sba_slot(k: u16) -> Result<usize> {
    let p = slot(k)?;
    if p > 6 {
        return Err("four projectile SBA slot");
    }
    Ok(p)
}
fn rb(rom: &[u8], a: u32) -> Result<u8> {
    if a & 65535 < 0x8000 {
        return Err("ROM window");
    }
    let o = (((a >> 16) & 127) * 32768 + (a & 32767)) as usize;
    rom.get(o).copied().ok_or("ROM extent")
}
fn rw(rom: &[u8], a: u32) -> Result<u16> {
    if a & 65535 == 65535 {
        return Err("ROM bank extent");
    }
    Ok(u16::from_le_bytes([rb(rom, a)?, rb(rom, a + 1)?]))
}
const SIN_COS: [i16; 320] = [
    // 0x94A957
    -256, -255, -255, -255, -254, -254, -253, -252, -251, -249, -248, -246, -244, -243, -241, -238, -236, -234, -231,
    -228, -225, -222, -219, -216, -212, -209, -205, -201, -197, -193, -189, -185, -181, -176, -171, -167, -162, -157,
    -152, -147, -142, -136, -131, -126, -120, -115, -109, -103, -97, -92, -86, -80, -74, -68, -62, -56, -49, -43, -37,
    -31, -25, -18, -12, -6, 0, 6, 12, 18, 25, 31, 37, 43, 49, 56, 62, 68, 74, 80, 86, 92, 97, 103, 109, 115, 120, 126,
    131, 136, 142, 147, 152, 157, 162, 167, 171, 176, 181, 185, 189, 193, 197, 201, 205, 209, 212, 216, 219, 222, 225,
    228, 231, 234, 236, 238, 241, 243, 244, 246, 248, 249, 251, 252, 253, 254, 254, 255, 255, 255, 256, 255, 255, 255,
    254, 254, 253, 252, 251, 249, 248, 246, 244, 243, 241, 238, 236, 234, 231, 228, 225, 222, 219, 216, 212, 209, 205,
    201, 197, 193, 189, 185, 181, 176, 171, 167, 162, 157, 152, 147, 142, 136, 131, 126, 120, 115, 109, 103, 97, 92,
    86, 80, 74, 68, 62, 56, 49, 43, 37, 31, 25, 18, 12, 6, 0, -6, -12, -18, -25, -31, -37, -43, -49, -56, -62, -68,
    -74, -80, -86, -92, -97, -103, -109, -115, -120, -126, -131, -136, -142, -147, -152, -157, -162, -167, -171, -176,
    -181, -185, -189, -193, -197, -201, -205, -209, -212, -216, -219, -222, -225, -228, -231, -234, -236, -238, -241,
    -243, -244, -246, -248, -249, -251, -252, -253, -254, -254, -255, -255, -255, -256, -255, -255, -255, -254, -254,
    -253, -252, -251, -249, -248, -246, -244, -243, -241, -238, -236, -234, -231, -228, -225, -222, -219, -216, -212,
    -209, -205, -201, -197, -193, -189, -185, -181, -176, -171, -167, -162, -157, -152, -147, -142, -136, -131, -126,
    -120, -115, -109, -103, -97, -92, -86, -80, -74, -68, -62, -56, -49, -43, -37, -31, -25, -18, -12, -6,
];
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
fn sine_inner(angle: u16, radius: u16) -> Result<u16> {
    let index = 64usize + angle as usize;
    if index >= 320 {
        return Err("sine table index");
    }
    let value = SIN_COS[index] as u16;
    Ok((((value as u8 as u16) * (radius as u8 as u16)) >> 8) + ((value >> 8) * (radius as u8 as u16)))
}
/// Exact byte products from the original fixed-point table; no floating point.
pub fn sine(angle: u16, radius: u16) -> Result<(u16, u16)> {
    let x = if angle.wrapping_sub(128) & 0x8000 != 0 {
        sine_inner(angle, radius)?
    } else {
        0u16.wrapping_sub(sine_inner((angle as u8).wrapping_add(128) as u16, radius)?)
    };
    let quarter = (angle as u8).wrapping_sub(64);
    let y = if quarter < 128 {
        sine_inner(u16::from(quarter), radius)?
    } else {
        0u16.wrapping_sub(sine_inner(u16::from(quarter.wrapping_add(128)), radius)?)
    };
    Ok((x, y))
}
fn plm(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, header: u16) -> Result<()> {
    if (0..80).step_by(2).any(|j| r.word(PLM_HEADERS + j) == 0) {
        let setup = rw(rom, 0x840000 | u32::from(header))?;
        if !PLM_SETUP.contains(&setup) {
            return Err("unknown PLM setup");
        }
        if setup == 0xb4c4 && r.word(COLLISION_DIRECTION) & 2 != 0 {
            return Err("quicksand needs movement context");
        }
        rw(rom, 0x840000 | u32::from(header.checked_add(2).ok_or("PLM pointer wrap")?))?;
    }
    h.spawn_plm(r, header)?;
    Ok(())
}
fn room(r: &Wram<'_>) -> Result<()> {
    if r.word(WIDTH) == 0 || r.word(WIDTH) > 255 || r.word(HEIGHT) == 0 || r.word(HEIGHT) > 255 {
        return Err("room geometry");
    }
    Ok(())
}
fn tile(r: &Wram<'_>, i: u16) -> Result<(u16, u8)> {
    if i >= 0x3200 {
        return Err("bomb tile backing extent");
    }
    Ok((r.word(LEVEL + 2 * i as usize), r.byte(BTS + i as usize)))
}
fn calculate(r: &mut Wram<'_>, x: u16, y: u16) {
    r.set_word(
        BLOCK,
        if (x as i16) >= 0 && x < 4096 && (y as i16) >= 0 && y < 4096 {
            (x >> 4) + ((y >> 4) as u8 as u16) * (r.word(WIDTH) as u8 as u16)
        } else {
            65535
        },
    );
}
pub(crate) fn bomb_reaction(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, byte_index: u16) -> Result<()> {
    let mut i = byte_index >> 1;
    r.set_word(BLOCK, i);
    for _ in 0..256 {
        let (t, b) = tile(r, i)?;
        match t >> 12 {
            5 | 13 => {
                if b == 0 {
                    return Ok(());
                }
                let delta = if t >> 12 == 5 { i16::from(b as i8) } else { i16::from(b as i8) * r.word(WIDTH) as i16 };
                i = i.wrapping_add(delta as u16);
                tile(r, i)?;
                r.set_word(BLOCK, i);
                continue;
            }
            4 | 12 => {
                if b & 128 == 0 {
                    let header = rw(rom, 0x949ea6 + 2 * u32::from(b))?;
                    plm(r, rom, h, header)?;
                } else if t >> 12 == 12 {
                    let area = r.word(AREA);
                    if area >= 8 {
                        return Err("regional bomb area");
                    }
                    let p = rw(rom, 0x949fc6 + 2 * u32::from(area))?;
                    let header = rw(
                        rom,
                        0x940000 | u32::from(p.checked_add(2 * u16::from(b & 127)).ok_or("regional bomb pointer")?),
                    )?;
                    plm(r, rom, h, header)?;
                }
            }
            7 | 15 => {
                if b & 128 == 0 {
                    let header = rw(rom, 0x94a012 + 2 * u32::from(b))?;
                    plm(r, rom, h, header)?;
                }
            }
            11 => {
                let header = if b & 128 != 0 {
                    let area = r.word(AREA);
                    if area >= 8 {
                        return Err("special bomb area");
                    }
                    let p = rw(rom, 0x949e44 + 2 * u32::from(area))?;
                    rw(rom, 0x940000 | u32::from(p.checked_add(2 * u16::from(b & 127)).ok_or("special bomb pointer")?))?
                } else {
                    rw(rom, 0x949da4 + 2 * u32::from(b))?
                };
                plm(r, rom, h, header)?;
            }
            _ => {}
        }
        return Ok(());
    }
    Err("cyclic bomb extension")
}
fn explode(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, k: u16) -> Result<()> {
    let p = slot(k)?;
    if r.word(VARIABLE + p) != 0 || r.word(TYPE + p) & 1 != 0 {
        return Ok(());
    }
    r.set_word(TYPE + p, r.word(TYPE + p) | 1);
    let i = r.word(BLOCK);
    if i == 65535 {
        return Ok(());
    }
    let center = 2 * i;
    let above = center.wrapping_sub(2 * r.word(WIDTH));
    let right = above.wrapping_add(2 * r.word(WIDTH) + 2);
    let left = right.wrapping_sub(4);
    let below = left.wrapping_add(2 * r.word(WIDTH) + 2);
    for i in [center, above, right, left, below] {
        bomb_reaction(r, rom, h, i)?;
    }
    Ok(())
}
pub fn bomb_contact(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, k: u16) -> Result<()> {
    room(r)?;
    let p = slot(k)?;
    let t = r.word(TYPE + p);
    let category = (t >> 8) & 15;
    let beam = t & 15;
    let code = if t & 0xf00 != 0 {
        if category > 8 {
            return Err("bomb type index");
        }
        [0, 0, 0, 3, 255, 2, 255, 255, 255][category as usize]
    } else {
        if beam > 11 {
            return Err("beam bomb reaction index");
        }
        [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1][beam as usize]
    };
    let x = r.word(X + p);
    let y = r.word(Y + p);
    if (x as i16) < 0 || x >> 8 >= r.word(WIDTH_SCROLLS) || (y as i16) < 0 || y >> 8 >= r.word(HEIGHT_SCROLLS) {
        return Ok(());
    }
    calculate(r, x, y);
    if code == 2 { explode(r, rom, h, k) } else { Err("power-bomb radius walk requires HDMA closure") }
}
pub fn spread_collision(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, k: u16) -> Result<u8> {
    room(r)?;
    let p = slot(k)?;
    calculate(r, r.word(X + p), r.word(Y + p));
    if r.word(VARIABLE + p) == 0 {
        explode(r, rom, h, k)?;
        return Ok(0);
    }
    let mut i = r.word(BLOCK);
    if i == 65535 {
        return Ok(1);
    }
    for _ in 0..256 {
        let (t, b) = tile(r, i)?;
        let result = match t >> 12 {
            0 | 2 | 3 | 6 | 7 => 0,
            1 => {
                if b & 31 < 5 {
                    1
                } else {
                    let x = (r.word(X + p) & 15) ^ if b & 64 != 0 { 15 } else { 0 };
                    let y = (r.word(Y + p) & 15) ^ if b & 128 != 0 { 15 } else { 0 };
                    u8::from((SLOPE[16 * (b & 31) as usize + x as usize] & 31) as i16 - y as i16 <= 0)
                }
            }
            5 | 13 => {
                if b == 0 {
                    return Ok(0);
                }
                let delta = if t >> 12 == 5 { i16::from(b as i8) } else { i16::from(b as i8) * r.word(WIDTH) as i16 };
                i = i.wrapping_add(delta as u16);
                tile(r, i)?;
                r.set_word(BLOCK, i);
                continue;
            }
            _ => 1,
        };
        return Ok(result);
    }
    Err("cyclic spread bomb extension")
}
fn trail_tick(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<u16> {
    let p = slot(k)?;
    let t = r.word(TIMER + p);
    r.set_word(TIMER + p, t.wrapping_sub(1));
    if t == 1 {
        r.set_word(TIMER + p, 4);
        projectile_flight::spawn_trail(r, rom, k)?;
        Ok(r.word(INDEX))
    } else {
        Ok(k)
    }
}
fn radial(r: &mut Wram<'_>, k: usize, radius: u16) -> Result<()> {
    let (x, y) = sine(r.word(VARIABLE + k), radius)?;
    r.set_word(X + k, x.wrapping_add(r.word(SAMUS_X)));
    r.set_word(Y + k, y.wrapping_add(r.word(SAMUS_Y)));
    Ok(())
}
fn advance_angle(r: &mut Wram<'_>, k: usize) {
    r.set_word(VARIABLE + k, (r.word(ROTATION).wrapping_add(r.word(VARIABLE + k)) as u8) as u16);
}
fn outside_sba(r: &Wram<'_>, k: usize) -> bool {
    let x = r.word(X + k).wrapping_sub(r.word(CAMERA_X));
    let y = r.word(Y + k).wrapping_sub(r.word(CAMERA_Y));
    x.wrapping_add(32) & 0x8000 != 0
        || x.wrapping_sub(288) & 0x8000 == 0
        || y.wrapping_sub(16) & 0x8000 != 0
        || y.wrapping_sub(256) & 0x8000 == 0
}
fn maintain(r: &mut Wram<'_>) {
    r.set_word(COOLDOWN, 2);
    r.set_word(FLARE, 0);
}
pub fn ice(r: &mut Wram<'_>, rom: &[u8], k: u16, expanding: bool) -> Result<()> {
    let p = sba_slot(k)?;
    if r.word(DIR + p) & 0xf0 != 0 {
        if !expanding {
            audio::dispatch(r, "QueueSfx1_Max6", 36, 0)?;
        }
        return projectiles::clear(r, k);
    }
    let k = trail_tick(r, rom, k)?;
    let p = sba_slot(k)?;
    if expanding {
        let (x, y) = sine(r.word(VARIABLE + p), r.word(VX + p))?;
        r.set_word(X + p, x.wrapping_add(r.word(SAMUS_X)));
        let dx = r.word(X + p).wrapping_sub(r.word(CAMERA_X));
        if dx.wrapping_add(32) & 0x8000 != 0 || dx.wrapping_sub(288) & 0x8000 == 0 {
            return projectiles::clear(r, k);
        }
        r.set_word(Y + p, y.wrapping_add(r.word(SAMUS_Y)));
        if outside_sba(r, p) {
            return projectiles::clear(r, k);
        }
        advance_angle(r, p);
        r.set_word(VX + p, (r.word(VX + p).wrapping_add(8) as u8) as u16);
        maintain(r);
    } else {
        radial(r, p, 32)?;
        advance_angle(r, p);
        let t = r.word(VY + p);
        r.set_word(VY + p, t.wrapping_sub(1));
        if t == 1 {
            r.set_word(PRE + p, 0xcf7a);
            r.set_word(VX + p, 40);
            audio::dispatch(r, "QueueSfx1_Max6", 36, 0)?;
        }
        maintain(r);
    }
    Ok(())
}
fn plasma_phase(r: &mut Wram<'_>, k: u16, phase: u16) -> Result<()> {
    let p = sba_slot(k)?;
    match phase {
        0 => {
            let v = (r.word(VX + p).wrapping_add(4) as u8) as u16;
            r.set_word(VX + p, v);
            if v.wrapping_sub(192) & 0x8000 == 0 {
                r.set_word(VY + p, 1);
            }
        }
        1 => {
            let v = (r.word(VX + p).wrapping_sub(4) as u8) as u16;
            r.set_word(VX + p, v);
            if v.wrapping_sub(45) & 0x8000 != 0 {
                r.set_word(VY + p, 2);
            }
        }
        2 => {
            if outside_sba(r, p) {
                projectiles::clear(r, k)?;
            } else {
                r.set_word(VX + p, (r.word(VX + p).wrapping_add(4) as u8) as u16);
            }
        }
        _ => return Err("plasma SBA phase"),
    };
    Ok(())
}
pub fn plasma(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<()> {
    let _ = rom;
    let p = sba_slot(k)?;
    if r.word(DIR + p) & 0xf0 != 0 {
        return projectiles::clear(r, k);
    }
    let phase = r.word(VY + p);
    if phase > 2 {
        return Err("plasma SBA phase");
    }
    maintain(r);
    radial(r, p, r.word(VX + p))?;
    advance_angle(r, p);
    plasma_phase(r, k, phase)
}
fn add_pair(r: &mut Wram<'_>, hi: usize, lo: usize, a: u32) {
    let v = ((r.word(hi) as u32) << 16) | r.word(lo) as u32;
    let v = v.wrapping_add(a);
    r.set_word(hi, (v >> 16) as u16);
    r.set_word(lo, v as u16);
}
pub fn wave(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<()> {
    let p = sba_slot(k)?;
    let t = r.word(VY + p);
    if r.word(DIR + p) & 0xf0 != 0 {
        audio::dispatch(r, "QueueSfx1_Max6", 41, 0)?;
        return projectiles::clear(r, k);
    }
    r.set_word(VY + p, t.wrapping_sub(1));
    if t == 1 || t.wrapping_sub(1) & 0x8000 != 0 {
        audio::dispatch(r, "QueueSfx1_Max6", 41, 0)?;
        return projectiles::clear(r, k);
    }
    let old = r.word(VX + p);
    let k = trail_tick(r, rom, k)?;
    for (position, speed, sub, target) in [(X, VX, SUB_X, SAMUS_X), (Y, VARIABLE, SUB_Y, SAMUS_Y)] {
        let v = r.word(speed + p);
        let delta = r.word(target).wrapping_sub(r.word(position + p));
        let next = if delta & 0x8000 != 0 {
            if v.wrapping_add(2047) & 0x8000 == 0 { v.wrapping_sub(64) } else { v }
        } else if v.wrapping_sub(2048) & 0x8000 != 0 {
            v.wrapping_add(64)
        } else {
            v
        };
        r.set_word(speed + p, next);
        add_pair(r, position + p, sub + p, (i32::from(next as i16) * 256) as u32);
    }
    if k == 6 && (r.word(VX + 6) ^ old) & 0x8000 != 0 {
        audio::dispatch(r, "QueueSfx1_Max6", 40, 0)?;
    }
    maintain(r);
    Ok(())
}
fn spazer_end(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<()> {
    let p = sba_slot(k)?;
    r.set_word(X + p, r.word(X + p).wrapping_add(if p < 4 { 16 } else { 65520 }));
    r.set_word(DIR + p, 5);
    r.set_word(TIMER + p, 4);
    r.set_word(PRE + p, 0xdc9c);
    if p < 4 {
        r.set_word(TYPE + p, 0x8024);
        projectiles::dispatch_rom(r, rom, "InitializeShinesparkEchoOrSpazerSba", i32::from(k), 0)?;
    }
    Ok(())
}
fn spazer_phase(r: &mut Wram<'_>, rom: &[u8], k: u16, y: u16, phase: u16) -> Result<()> {
    let p = sba_slot(k)?;
    let direction = r.word(VY + p) as u8;
    if phase == 0 {
        r.set_word(Y + p, y.wrapping_add(r.word(SAMUS_Y)));
        let angle = direction.wrapping_add(r.word(VARIABLE + p) as u8);
        r.set_word(VARIABLE + p, u16::from(angle));
        if angle == 128 {
            r.set_word(VX + p, 160);
            r.set_word(VY + p, if p & 2 == 0 { 2 } else { 65534 });
            r.set_word(DIR + p, 0);
            r.set_word(AUX + p, 2);
        }
        return Ok(());
    }
    if phase != 1 && phase != 2 {
        return Err("spazer phase");
    }
    let yy = y.wrapping_add(r.word(SAMUS_Y)).wrapping_sub(114);
    r.set_word(Y + p, yy);
    if yy.wrapping_sub(r.word(CAMERA_Y)).wrapping_sub(16) & 0x8000 != 0 {
        return spazer_end(r, rom, k);
    }
    r.set_word(VARIABLE + p, u16::from(direction.wrapping_add(r.word(VARIABLE + p) as u8)));
    let speed = if phase == 1 { r.word(VX + p).wrapping_sub(5) } else { r.word(VX + p).wrapping_add(5) };
    r.set_word(VX + p, speed);
    if phase == 1 {
        if speed == 0 {
            r.set_word(VY + p, if p & 2 == 0 { 65534 } else { 2 });
            r.set_word(VARIABLE + p, u16::from((r.word(VARIABLE + p) as u8).wrapping_add(128)));
            r.set_word(AUX + p, 4);
            if p == 0 {
                audio::dispatch(r, "QueueSfx1_Max6", 38, 0)?;
            }
        }
    } else if speed.wrapping_sub(96) & 0x8000 == 0 {
        spazer_end(r, rom, k)?;
    }
    Ok(())
}
pub fn spazer(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<()> {
    let p = sba_slot(k)?;
    if r.word(DIR + p) & 0xf0 != 0 {
        projectiles::clear(r, k)?;
        return projectiles::clear(r, [4, 6, 0, 2][p / 2]);
    }
    let k = trail_tick(r, rom, k)?;
    let p = sba_slot(k)?;
    let phase = r.word(AUX + p) >> 1;
    if phase > 2 {
        return Err("spazer phase index");
    }
    let (x, y) = sine(r.word(VARIABLE + p), r.word(VX + p))?;
    r.set_word(X + p, x.wrapping_add(r.word(SAMUS_X)));
    spazer_phase(r, rom, k, y, phase)?;
    maintain(r);
    Ok(())
}
fn speed_echo(r: &mut Wram<'_>, k: u16) -> Result<()> {
    let p = slot(k)?;
    r.set_word(VX + p, r.word(VX + p).wrapping_add(8));
    let (x, y) = sine(r.word(VARIABLE + p), r.word(VX + p) as u8 as u16)?;
    let x = x.wrapping_add(r.word(SAMUS_X));
    r.set_word(ECHO_INDEX + p, x);
    r.set_word(X + p, x);
    let dx = x.wrapping_sub(r.word(CAMERA_X));
    let mut outside = dx & 0x8000 != 0 || dx.wrapping_sub(256) & 0x8000 == 0;
    if !outside {
        let yy = y.wrapping_add(r.word(SAMUS_Y));
        r.set_word(ECHO_Y_ALIAS + p, yy);
        r.set_word(Y + p, yy);
        let dy = yy.wrapping_sub(r.word(CAMERA_Y));
        outside = dy & 0x8000 != 0 || dy.wrapping_sub(256) & 0x8000 == 0;
    }
    if outside {
        r.set_word(ECHO_CLEAR_ALIAS + p, 0);
        r.set_word(ECHO_INDEX + p, 0);
        r.set_word(ECHO_Y_ALIAS + p, 0);
        projectiles::clear(r, k)?;
    }
    Ok(())
}
pub fn spread(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, k: u16) -> Result<()> {
    let p = slot(k)?;
    if p < 10 {
        return Err("spread bomb slot");
    }
    if r.word(DIR + p) & 0xf0 != 0 {
        return projectiles::clear(r, k);
    }
    slot(r.word(INDEX))?;
    projectiles::dispatch_rom(r, rom, "Bomb_Func2", 0, 0)?;
    if r.word(VARIABLE + p) != 0 {
        let accel = (u32::from(r.word(GRAVITY_INTEGER)) << 16) | u32::from(r.word(GRAVITY_FRACTION));
        add_pair(r, VY + p, TIMER + p, accel);
        let velocity = (u32::from(r.word(VY + p)) << 16) | u32::from(r.word(TIMER + p));
        add_pair(r, Y + p, SUB_Y + p, velocity);
        if spread_collision(r, rom, h, k)? != 0 {
            add_pair(r, Y + p, SUB_Y + p, velocity.wrapping_neg());
            if r.word(VY + p) & 0x8000 != 0 {
                r.set_word(VY + p, 0);
                r.set_word(YR + p, 0);
            } else {
                r.set_word(TIMER + p, if p == 14 { 0x8000 } else { 0 });
                r.set_word(VY + p, r.word(AUX + p));
            }
            return Ok(());
        }
        let t = r.word(VX + p);
        let amount = if t & 0x8000 != 0 { -(i32::from((t & 0x7fff) as i16) * 256) } else { i32::from(t as i16) * 256 };
        add_pair(r, X + p, SUB_X + p, amount as u32);
    }
    if spread_collision(r, rom, h, k)? != 0 {
        let t = r.word(VX + p);
        r.set_word(VX + p, t ^ 0x8000);
        let reverse = if t & 0x8000 == 0 { -(i32::from(t as i16) * 256) } else { i32::from((t & 0x7fff) as i16) * 256 };
        add_pair(r, X + p, SUB_X + p, reverse as u32);
    }
    Ok(())
}
fn spawn_spread(r: &mut Wram<'_>, rom: &[u8]) -> Result<()> {
    if r.word(PRE + 10) == 0xd8f7 {
        return Ok(());
    }
    for i in 0..5 {
        let p = 10 + 2 * i;
        r.set_word(TYPE + p, 0x8500);
        r.set_word(DIR + p, 0);
        r.set_word(PRE + p, 0xd8f7);
        projectiles::dispatch_rom(r, rom, "InitializeInstrForMissile", p as i32, 0)?;
        r.set_word(X + p, r.word(SAMUS_X));
        r.set_word(SUB_X + p, 0);
        r.set_word(Y + p, r.word(SAMUS_Y));
        r.set_word(SUB_Y + p, 0);
        r.set_word(VX + p, [0x8100, 0x8080, 0, 0x80, 0x100][i]);
        r.set_word(TIMER + p, if i == 2 { 0x8000 } else { 0 });
        let speed = 0u16.wrapping_sub([0, 1, 2, 1, 0][i] + ((r.word(SPREAD_CHARGE) >> 6) & 3));
        r.set_word(VY + p, speed);
        r.set_word(AUX + p, speed);
        r.set_word(VARIABLE + p, [120, 110, 100, 110, 120][i]);
    }
    r.set_word(BOMB_COUNT, 5);
    r.set_word(COOLDOWN, 16);
    r.set_word(SPREAD_CHARGE, 0);
    r.set_word(FLARE, 0);
    Ok(())
}
fn fire_sba(r: &mut Wram<'_>, rom: &[u8], kind: u16) -> Result<u8> {
    if kind == 0 {
        return Ok(0);
    }
    if kind == 2 && matches!(r.word(PRE), 0xcf09 | 0xcf7a) || kind == 8 && r.word(PRE) == 0xd793 {
        return Ok(0);
    }
    let beams = r.word(EQUIPPED_BEAMS) & 0x100f | 0x8010;
    if beams & 15 > 11 {
        return Err("SBA beam type");
    }
    for p in (0..8).step_by(2).rev() {
        let i = p / 2;
        r.set_word(TYPE + p, beams);
        r.set_word(DIR + p, 0);
        match kind {
            1 => {
                r.set_word(TIMER + p, 4);
                r.set_word(PRE + p, 0xda08);
                r.set_word(VY + p, 600);
                r.set_word(VX + p, 0);
                r.set_word(VARIABLE + p, 0);
                r.set_word(SUB_X + p, 0);
                r.set_word(SUB_Y + p, 0);
                r.set_word(X + p, r.word(SAMUS_X).wrapping_add([128, 128, 65408, 65408][i]));
                r.set_word(Y + p, r.word(SAMUS_Y).wrapping_add([128, 65408, 65408, 128][i]));
                projectiles::dispatch_rom(r, rom, "InitializeSbaProjectile", p as i32, 0)?;
            }
            2 => {
                r.set_word(TIMER + p, 4);
                r.set_word(PRE + p, 0xcf09);
                r.set_word(VARIABLE + p, [0, 64, 128, 192][i]);
                r.set_word(VY + p, 600);
                projectiles::initialize(r, rom, p as u16)?;
            }
            4 => {
                r.set_word(TIMER + p, if p >= 4 { 4 } else { 0 });
                r.set_word(DIR + p, 5);
                r.set_word(PRE + p, 0xdb06);
                r.set_word(VX + p, 40);
                r.set_word(VY + p, if i & 1 == 0 { 4 } else { 65532 });
                r.set_word(VARIABLE + p, 0);
                r.set_word(AUX + p, 0);
                r.set_word(SUB_X + p, 0);
                r.set_word(SUB_Y + p, 0);
                if p >= 4 {
                    r.set_word(TYPE + p, 0x8024);
                    projectiles::dispatch_rom(r, rom, "InitializeShinesparkEchoOrSpazerSba", p as i32, 0)?;
                } else {
                    projectiles::dispatch_rom(r, rom, "InitializeSbaProjectile", p as i32, 0)?;
                }
            }
            8 => {
                r.set_word(PRE + p, 0xd793);
                r.set_word(VARIABLE + p, [0, 64, 128, 192][i]);
                r.set_word(VX + p, 40);
                r.set_word(VY + p, 0);
                projectiles::dispatch_rom(r, rom, "InitializeSbaProjectile", p as i32, 0)?;
            }
            _ => return Err("SBA kind"),
        }
    }
    r.set_word(PROJECTILE_COUNT, 4);
    let t = (r.word(TYPE) & 63) as usize;
    let cooldown = if t < 12 {
        if t == 10 { 12 } else { 15 }
    } else if (16..28).contains(&t) {
        30
    } else if t < 38 {
        0
    } else {
        return Err("SBA cooldown index");
    };
    r.set_word(COOLDOWN, cooldown);
    r.set_word(
        ROTATION,
        if kind == 4 {
            0
        } else if kind == 1 || r.byte(SAMUS_FACING) != 4 {
            4
        } else {
            65532
        },
    );
    audio::dispatch(
        r,
        "QueueSfx1_Max6",
        match kind {
            1 => 40,
            2 => 35,
            4 => 37,
            8 => 39,
            _ => 0,
        },
        0,
    )?;
    Ok(1)
}
pub fn dispatch_host(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, symbol: &str, a: i32, b: i32) -> Result<u32> {
    let k = a as u16;
    match symbol {
        "ProjPreInstr_Bomb" => {
            let p = slot(k)?;
            if r.word(DIR + p) & 0xf0 != 0 {
                projectiles::clear(r, k)?;
            } else {
                slot(r.word(INDEX))?;
                projectiles::dispatch_rom(r, rom, "Bomb_Func2", 0, 0)?;
                bomb_contact(r, rom, h, k)?;
            }
        }
        "BombOrPowerBomb_Func1" => bomb_contact(r, rom, h, k)?,
        "BlockCollSpreadBomb" => return Ok(u32::from(spread_collision(r, rom, h, k)?)),
        "ProjPreInstr_SpreadBomb" => spread(r, rom, h, k)?,
        "BombSpread" => spawn_spread(r, rom)?,
        "ProjPreInstr_IceSba" => ice(r, rom, k, false)?,
        "ProjPreInstr_IceSba2" => ice(r, rom, k, true)?,
        "ProjPreInstr_PlasmaSba" => plasma(r, rom, k)?,
        "ProjPreInstr_WaveSba" => wave(r, rom, k)?,
        "ProjPreInstr_PlasmaSbaFunc_0" => plasma_phase(r, k, 0)?,
        "ProjPreInstr_PlasmaSbaFunc_1" => plasma_phase(r, k, 1)?,
        "ProjPreInstr_PlasmaSbaFunc_2" => plasma_phase(r, k, 2)?,
        "ProjPreInstr_SpazerSba" => spazer(r, rom, k)?,
        "FireEndOfSpazerSba" => spazer_end(r, rom, k)?,
        "ProjPreInstr_EndOfSpazerSba" => {
            let p = sba_slot(k)?;
            if r.word(DIR + p) & 0xf0 != 0 {
                projectiles::clear(r, k)?;
            } else {
                let k = trail_tick(r, rom, k)?;
                let p = sba_slot(k)?;
                r.set_word(Y + p, r.word(Y + p).wrapping_add(8));
                if r.word(Y + p).wrapping_sub(r.word(CAMERA_Y)).wrapping_sub(248) & 0x8000 == 0 {
                    projectiles::clear(r, k)?;
                } else {
                    maintain(r);
                }
            }
        }
        "ProjPreInstr_SpeedEcho" => speed_echo(r, k)?,
        "FireSba_ClearCarry" => return Ok(0),
        "FireSba_FireWave" => return Ok(u32::from(fire_sba(r, rom, 1)?)),
        "FireSba_FireIce" => return Ok(u32::from(fire_sba(r, rom, 2)?)),
        "FireSba_FireSpazer" => return Ok(u32::from(fire_sba(r, rom, 4)?)),
        "FireSba_FirePlasma" => return Ok(u32::from(fire_sba(r, rom, 8)?)),
        "FireSba" => {
            if r.word(HUD_ITEM) != 3 {
                return Ok(0);
            }
            let kind = r.word(EQUIPPED_BEAMS) & 15;
            if kind > 11 {
                return Err("SBA dispatch type");
            }
            let cost = [0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0][kind as usize];
            let ammo = r.word(POWER_BOMBS).wrapping_sub(cost);
            r.set_word(POWER_BOMBS, if ammo & 0x8000 != 0 { 0 } else { ammo });
            let result = fire_sba(r, rom, if matches!(kind, 1 | 2 | 4 | 8) { kind } else { 0 })?;
            if r.word(POWER_BOMBS) == 0 {
                r.set_word(HUD_ITEM, 0);
                r.set_word(AUTO_CANCEL_HUD, 0);
            }
            return Ok(u32::from(result));
        }
        name if name.starts_with("ProjPreInstr_SpazerSba_FuncA_") => {
            let p = sba_slot(k)?;
            let index = name.chars().last().unwrap().to_digit(10).ok_or("spazer A handler")? as usize;
            if index > 3 {
                return Err("spazer A handler");
            }
            let _ = p;
            projectiles::clear(r, k)?;
            projectiles::clear(r, [4, 6, 0, 2][index])?;
        }
        name if name.starts_with("ProjPreInstr_SpazerSba_FuncB_") => {
            let phase = name.chars().last().unwrap().to_digit(10).ok_or("spazer B handler")? as u16;
            spazer_phase(r, rom, k, b as u16, phase)?;
        }
        "ProjPreInstr_UnknownProj8027" => {
            let p = sba_slot(k)?;
            let i = p / 2;
            r.set_word(X + p, r.word(X + p).wrapping_add([-4i16, -4, 4, 4][i] as u16));
            let y = r.word(Y + p).wrapping_add([4i16, -4, -4, 4][i] as u16);
            r.set_word(Y + p, y);
            if y == r.word(SAMUS_Y) {
                if r.word(VARIABLE + p) == 1 {
                    if k == 6 {
                        r.set_word(MOVEMENT_HANDLER, 0xf04b);
                    }
                    projectiles::clear(r, k)?;
                } else {
                    r.set_word(VARIABLE + p, r.word(VARIABLE + p).wrapping_add(1));
                    r.set_word(SHINE_TIMER, 180);
                    r.set_word(SHINE_TIMER_PHASE, 1);
                    r.set_word(PALETTE_FRAME, 0);
                    r.set_word(X + p, r.word(SAMUS_X).wrapping_add([128i16, 128, -128, -128][i] as u16));
                    r.set_word(Y + p, r.word(SAMUS_Y).wrapping_add([-128i16, 128, 128, -128][i] as u16));
                }
            }
        }
        _ => return Err("unknown special projectile routine"),
    }
    Ok(0)
}
