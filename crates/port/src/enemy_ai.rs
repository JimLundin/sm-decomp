//! Rio, Waver and Ripper flying-enemy AI from pinned MIT snesrev/sm banks A2/A3.
//! Movement retains the A0 terrain contract; PLM spawning is an explicit host call.
use crate::{Wram, audio, enemy};
const ENEMIES: usize = 0xf78;
const CURRENT: usize = 0xe54;
const X: usize = 2;
const XS: usize = 4;
const Y: usize = 6;
const YS: usize = 8;
const WIDTH: usize = 10;
const HEIGHT: usize = 12;
const TIMER: usize = 0x18;
const INSTRUCTION: usize = 0x1a;
const INSTRUCTION_TIMER: usize = 0x1c;
const A: usize = 0x30;
const B: usize = 0x32;
const C: usize = 0x34;
const D: usize = 0x36;
const E: usize = 0x38;
const F: usize = 0x3a;
const PARAM1: usize = 0x3c;
const PARAM2: usize = 0x3e;
const ROOM_WIDTH: usize = 0x7a5;
const BLOCK: usize = 0xdc4;
const LEVEL: usize = 0x10002;
const BTS: usize = 0x16402;
const SAMUS_X: usize = 0xaf6;
const SAMUS_Y: usize = 0xafa;
/// Observable external allocation required by enemy-sensitive spike tiles.
pub trait Host {
    fn spawn_plm(&mut self, ram: &mut Wram<'_>, header: u16, amount: &mut i32) -> Result<u8, &'static str>;
}
fn record(index: u16) -> Result<usize, &'static str> {
    if index >= 0x800 || index & 63 != 0 {
        return Err("invalid enemy record");
    }
    Ok(ENEMIES + index as usize)
}
fn rom_offset(address: u32) -> usize {
    (((address >> 16) & 0x7f) * 0x8000 + (address & 0x7fff)) as usize
}
fn rom_byte(rom: &[u8], address: u32) -> Result<u8, &'static str> {
    rom.get(rom_offset(address)).copied().ok_or("missing enemy ROM table")
}
fn rom_word(rom: &[u8], address: u32) -> Result<u16, &'static str> {
    Ok(u16::from_le_bytes([rom_byte(rom, address)?, rom_byte(rom, address + 1)?]))
}
fn set(r: &mut Wram<'_>, e: usize, field: usize, value: u16) {
    r.set_word(e + field, value);
}
fn animation(r: &mut Wram<'_>, e: usize, address: u16) {
    set(r, e, INSTRUCTION, address);
    set(r, e, INSTRUCTION_TIMER, 1);
    set(r, e, TIMER, 0);
}
fn rio_animation(r: &mut Wram<'_>, e: usize, address: u16) {
    if r.word(e + F) != address {
        set(r, e, F, address);
        animation(r, e, address);
    }
}
fn waver_animation(r: &mut Wram<'_>, rom: &[u8], e: usize) -> Result<(), &'static str> {
    let variant = r.word(e + F);
    if variant >= 4 {
        return Err("Waver animation index outside four lists");
    }
    if variant != r.word(e + C) {
        let address = rom_word(rom, 0xa386db + 2 * u32::from(variant))?;
        set(r, e, C, variant);
        animation(r, e, address);
    }
    Ok(())
}
const SINE_COSINE: [i16; 320] = [
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
const SQUARE: [u8; 20] =
    [0, 1, 0x82, 0x83, 0, 0x81, 2, 0x83, 0, 1, 2, 0x83, 0, 0x81, 0x82, 0x83, 0x80, 0x81, 0x82, 0x83];
struct Collision {
    amount: i32,
    position: u16,
    edge: u16,
    remaining: u16,
    total: u16,
    horizontal: bool,
}
fn terrain(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    coll: &Collision,
    index: u16,
) -> Result<bool, &'static str> {
    r.set_word(BLOCK, index);
    // Extensions are data chains. A finite visited set admits no looping C chain.
    let mut seen = std::collections::BTreeSet::new();
    loop {
        let block = r.word(BLOCK);
        if block >= 0x3200 || !seen.insert(block) {
            return Err("invalid or cyclic enemy terrain extension");
        }
        let bts = r.byte(BTS + block as usize);
        match r.word(LEVEL + 2 * block as usize) >> 12 {
            0 | 2 | 3 | 4 | 6 | 7 => return Ok(false),
            8 | 9 | 11 | 12 | 14 | 15 => return Ok(true),
            5 | 13 => {
                if bts == 0 {
                    return Ok(false);
                }
                let shift = i32::from(bts as i8)
                    * if r.word(LEVEL + 2 * block as usize) >> 12 == 13 { i32::from(r.word(ROOM_WIDTH)) } else { 1 };
                r.set_word(BLOCK, block.wrapping_add(shift as u16));
            }
            10 => {
                if bts & 0x7f >= 15 {
                    return Err("spike BTS outside 15-entry PLM header table");
                }
                let header = rom_word(rom, 0xa0c2da + 2 * u32::from(bts & 0x7f))?;
                if header == 0 {
                    return Ok(true);
                }
                let mut amount = 0;
                host.spawn_plm(r, header, &mut amount)?;
                return Ok(false);
            }
            1 => {
                let e = record(r.word(CURRENT))?;
                let slope = bts & 31;
                if slope < 5 {
                    let flip = u16::from(bts >> 6);
                    let lane = if coll.horizontal { (coll.edge & 8) >> 3 } else { (coll.edge & 8) >> 2 };
                    let i = (4 * u16::from(slope) + (flip ^ lane)) as usize;
                    let radius = r.word(e + if coll.horizontal { HEIGHT } else { WIDTH });
                    let pos = r.word(e + if coll.horizontal { Y } else { X });
                    if coll.remaining == 0 && (radius as u8).wrapping_add(pos as u8).wrapping_sub(1) & 8 == 0 {
                        return Ok(SQUARE[i] & 128 != 0);
                    }
                    if (coll.remaining == 0 || coll.remaining != coll.total || pos.wrapping_sub(radius) & 8 == 0)
                        && SQUARE[i] & 128 != 0
                    {
                        return Ok(true);
                    }
                    return Ok(SQUARE[i ^ if coll.horizontal { 2 } else { 1 }] & 128 != 0);
                }
                if coll.horizontal {
                    r.set_word(0x1e77, u16::from(bts));
                    return Ok(false);
                }
                let width = r.word(ROOM_WIDTH);
                if width == 0 {
                    return Err("slope modulus by zero");
                }
                if r.word(e + X) >> 4 != block % width {
                    return Ok(false);
                }
                let x = if bts & 0x40 != 0 { r.word(e + X) ^ 15 } else { r.word(e + X) };
                let table = rom_byte(rom, 0x948b2b + 16 * u32::from(slope) + u32::from(x & 15))? & 31;
                let radius = r.word(e + HEIGHT);
                if coll.amount < 0 {
                    let edge = (coll.position.wrapping_sub(radius) & 15) ^ 15;
                    let delta = i32::from(table) - i32::from(edge) - 1;
                    if bts & 128 != 0 && delta <= 0 {
                        set(r, e, Y, coll.position.wrapping_sub(delta as u16));
                        set(r, e, YS, 0);
                        return Ok(true);
                    }
                } else {
                    let edge = radius.wrapping_add(coll.position).wrapping_sub(1) & 15;
                    let delta = i32::from(table) - i32::from(edge) - 1;
                    if bts & 128 == 0 && delta <= 0 {
                        set(r, e, Y, coll.position.wrapping_add(delta as u16));
                        set(r, e, YS, 65535);
                        return Ok(true);
                    }
                }
                return Ok(false);
            }
            _ => unreachable!(),
        }
    }
}
/// A0 collision movement, including fractional carry, extensions, slopes and PLMs.
fn movement(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    k: u16,
    amount: i32,
    horizontal: bool,
) -> Result<bool, &'static str> {
    let e = record(k)?;
    if amount == 0 {
        return Ok(false);
    }
    let (axis, sub, radius, perp, perp_radius) =
        if horizontal { (X, XS, WIDTH, Y, HEIGHT) } else { (Y, YS, HEIGHT, X, WIDTH) };
    let lower = r.word(e + perp).wrapping_sub(r.word(e + perp_radius));
    let total =
        r.word(e + perp_radius).wrapping_add(r.word(e + perp)).wrapping_sub(1).wrapping_sub(lower & 0xfff0) >> 4;
    let new = ((u32::from(r.word(e + axis)) << 16) | u32::from(r.word(e + sub))).wrapping_add(amount as u32);
    let position = (new >> 16) as u16;
    let edge = if amount < 0 {
        position.wrapping_sub(r.word(e + radius))
    } else {
        r.word(e + radius).wrapping_add(position).wrapping_sub(1)
    };
    let width = r.word(ROOM_WIDTH);
    let mult = |a: u16| u16::from(a as u8) * u16::from(width as u8);
    let mut byte_index = if horizontal {
        2u16.wrapping_mul(mult(lower >> 4).wrapping_add(edge >> 4))
    } else {
        2u16.wrapping_mul(mult(edge >> 4).wrapping_add(lower >> 4))
    };
    let mut coll = Collision { amount, position, edge, remaining: total, total, horizontal };
    loop {
        if terrain(r, rom, host, &coll, byte_index >> 1)? {
            break;
        }
        coll.remaining = coll.remaining.wrapping_sub(1);
        if coll.remaining & 0x8000 != 0 {
            set(r, e, axis, (new >> 16) as u16);
            set(r, e, sub, new as u16);
            return Ok(false);
        }
        byte_index = byte_index.wrapping_add(if horizontal { width.wrapping_mul(2) } else { 2 });
    }
    if amount < 0 {
        let snapped = r.word(e + radius).wrapping_add(1).wrapping_add(edge | 15);
        if snapped <= r.word(e + axis) {
            set(r, e, axis, snapped);
        }
        set(r, e, sub, 0);
    } else {
        let snapped = (edge & 0xfff0).wrapping_sub(r.word(e + radius));
        if snapped >= r.word(e + axis) {
            set(r, e, axis, snapped);
        }
        set(r, e, sub, 65535);
    }
    Ok(true)
}
fn speed(rom: &[u8], index: u16, offset: u16) -> Result<u16, &'static str> {
    // Linear table occupies 0x8187..0x8387; four words per speed row.
    if index > 252 || index & 3 != 0 {
        return Err("Ripper speed index outside linear table");
    }
    rom_word(rom, 0xa28187 + 2 * u32::from(index + offset))
}
fn ripper_init(r: &mut Wram<'_>, rom: &[u8], e: usize, jet: bool) -> Result<(), &'static str> {
    let parameter = r.word(e + PARAM1);
    if parameter > 63 {
        return Err("Ripper speed parameter outside table");
    }
    let right = r.word(e + PARAM2) != 0;
    let index = parameter * 4;
    let d = speed(rom, index, if right { 0 } else { 2 })?;
    let c = speed(rom, index, if right { 1 } else { 3 })?;
    set(
        r,
        e,
        INSTRUCTION,
        if jet {
            if right { 0xe2f4 } else { 0xe2e0 }
        } else if right {
            0xe477
        } else {
            0xe48b
        },
    );
    set(r, e, E, parameter * 8);
    set(r, e, D, d);
    set(r, e, C, c);
    Ok(())
}
fn ripper_main(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host, k: u16, jet: bool) -> Result<(), &'static str> {
    let e = record(k)?;
    let index = r.word(e + E) >> 1;
    speed(rom, index, 3)?;
    let amount = ((u32::from(r.word(e + D)) << 16) | u32::from(r.word(e + C))) as i32;
    if movement(r, rom, host, k, amount, true)? {
        let right = r.word(e + D) & 0x8000 == 0;
        let d = speed(rom, index, if right { 2 } else { 0 })?;
        let c = speed(rom, index, if right { 3 } else { 1 })?;
        set(r, e, D, d);
        set(r, e, C, c);
        animation(
            r,
            e,
            if jet {
                if right { 0xe2e0 } else { 0xe2f4 }
            } else if right {
                0xe48b
            } else {
                0xe477
            },
        );
    }
    Ok(())
}
fn rio(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host, k: u16, state: u16) -> Result<(), &'static str> {
    let e = record(k)?;
    match state {
        0xbbed => {
            if enemy::dispatch_rom(r, rom, "CompareDistToSamus_X", i32::from(k), 160)? & 1 == 0 {
                let c = rom_word(rom, 0xa2bbbb)?;
                let mut d = rom_word(rom, 0xa2bbbf)?;
                if r.word(SAMUS_X).wrapping_sub(r.word(e + X)) & 0x8000 != 0 {
                    d = d.wrapping_neg();
                }
                set(r, e, C, c);
                set(r, e, D, d);
                rio_animation(r, e, 0xbb7f);
                set(r, e, B, 0xbc48);
                if enemy::dispatch_rom(r, rom, "CheckIfEnemyIsOnScreen", 0, 0)? == 0 {
                    audio::dispatch(r, "QueueSfx2_Max6", 0x65, 0)?;
                }
            }
        }
        0xbc32 => {
            if r.word(e + E) != 0 {
                set(r, e, E, 0);
                rio_animation(r, e, 0xbb53);
                set(r, e, B, 0xbbed);
            }
        }
        0xbc48 => {
            let hit_x = movement(r, rom, host, k, i32::from(r.word(e + D) as i16) * 256, true)?;
            if hit_x {
                set(r, e, D, r.word(e + D).wrapping_neg());
            }
            let hit_y =
                if hit_x { false } else { movement(r, rom, host, k, i32::from(r.word(e + C) as i16) * 256, false)? };
            if hit_x || hit_y {
                set(r, e, C, r.word(e + C).wrapping_neg());
                set(r, e, B, 0xbcb7);
            } else {
                let c = r.word(e + C).wrapping_sub(24);
                set(r, e, C, c);
                if c & 0x8000 != 0 {
                    set(r, e, A, r.word(e + D));
                    set(r, e, D, 0);
                    set(r, e, C, 0);
                    set(r, e, B, 0xbcff);
                } else if r.word(e + E) != 0 {
                    set(r, e, E, 0);
                    rio_animation(r, e, 0xbb97);
                }
            }
        }
        0xbcb7 => {
            if movement(r, rom, host, k, i32::from(r.word(e + D) as i16) * 256, true)? {
                set(r, e, D, r.word(e + D).wrapping_neg());
            }
            if movement(r, rom, host, k, i32::from(r.word(e + C) as i16) * 256, false)? {
                rio_animation(r, e, 0xbba3);
                set(r, e, B, 0xbc32);
            } else {
                set(r, e, C, r.word(e + C).wrapping_sub(24));
            }
        }
        0xbcff => {
            if r.word(e + Y).wrapping_sub(r.word(SAMUS_Y)) & 0x8000 == 0 {
                set(r, e, D, r.word(e + A));
                set(r, e, C, 65535);
                set(r, e, B, 0xbcb7);
            } else {
                let angle = enemy::dispatch_rom(r, rom, "CalculateAngleOfSamusFromEnemy", i32::from(k), 0)?;
                let d = SINE_COSINE[(angle + 64) as usize] as u16;
                let c = SINE_COSINE[angle as usize] as u16;
                set(r, e, D, d);
                set(r, e, C, c);
                movement(r, rom, host, k, i32::from(d as i16) * 256, true)?;
                movement(r, rom, host, k, i32::from(c as i16) * 256, false)?;
            }
        }
        _ => return Err("unsupported Rio state"),
    }
    Ok(())
}
fn waver_main(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host, k: u16) -> Result<(), &'static str> {
    let e = record(k)?;
    if r.word(e + F) >= 4 {
        return Err("invalid Waver animation state");
    }
    let amount = ((u32::from(r.word(e + B)) << 16) | u32::from(r.word(e + A))) as i32;
    if movement(r, rom, host, k, amount, true)? {
        let packed = r.word(e + A) >> 8 | r.word(e + B) << 8;
        set(r, e, B, (((packed.wrapping_neg() & 0xff00) >> 8) as u8 as i8 as i16) as u16);
        set(r, e, A, ((-(packed as u8 as i8 as i16)) as u16) << 8);
        set(r, e, F, (r.word(e + F) ^ 1) & 1);
        waver_animation(r, rom, e)?;
    } else {
        let sine = enemy::dispatch_rom(r, rom, "SineMult8bit", i32::from(r.word(e + D) as u8), 4)? as u16;
        if movement(r, rom, host, k, i32::from(sine as i16) * 65536, false)? {
            set(r, e, D, u16::from(r.word(e + D).wrapping_add(128) as u8));
        } else {
            set(r, e, D, r.word(e + D).wrapping_add(2));
        }
    }
    if r.word(e + D) & 127 == 56 {
        set(r, e, F, r.word(e + F) | 2);
        waver_animation(r, rom, e)?;
    }
    if r.word(e + E) != 0 {
        set(r, e, E, 0);
        set(r, e, F, r.word(e + F) & 1);
        waver_animation(r, rom, e)?;
    }
    Ok(())
}
/// Dispatch only admitted enemy and table domains; unknown AI terminates explicitly.
pub fn dispatch_host(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    symbol: &str,
    a: i32,
    b: i32,
) -> Result<u32, &'static str> {
    let k = r.word(CURRENT);
    let e = record(k)?;
    match symbol {
        "Enemy_MoveRight_IgnoreSlopes" | "Enemy_MoveDown" => {
            return Ok(u32::from(movement(
                r,
                rom,
                host,
                u16::try_from(a).map_err(|_| "invalid enemy argument")?,
                b,
                symbol == "Enemy_MoveRight_IgnoreSlopes",
            )?));
        }
        "Ripper_Init" | "JetPowerRipper_Init" => ripper_init(r, rom, e, symbol.starts_with("Jet"))?,
        "Ripper_Main" | "JetPowerRipper_Main" => ripper_main(r, rom, host, k, symbol.starts_with("Jet"))?,
        "Rio_Init" => {
            set(r, e, E, 0);
            set(r, e, F, 0);
            set(r, e, INSTRUCTION, 0xbb4b);
            set(r, e, B, 0xbbed);
        }
        "Rio_Main" => {
            let state = r.word(e + B);
            if ![0xbbed, 0xbc32, 0xbc48, 0xbcb7, 0xbcff].contains(&state) {
                return Err("unsupported Rio state");
            }
            let n = r.word(0x5e5);
            let low = u16::from(n as u8) * 5;
            let carry = (low >> 8) + u16::from((n >> 8) as u8).wrapping_mul(5) as u8 as u16 + 1;
            r.set_word(0x5e5, (low & 255 | (carry as u8 as u16) << 8).wrapping_add(carry >> 8).wrapping_add(17));
            rio(r, rom, host, k, state)?;
        }
        "Rio_1" | "Rio_2" | "Rio_3" | "Rio_4" | "Rio_5" => {
            let index = u16::try_from(a).map_err(|_| "invalid Rio record")?;
            rio(
                r,
                rom,
                host,
                index,
                match symbol {
                    "Rio_1" => 0xbbed,
                    "Rio_2" => 0xbc32,
                    "Rio_3" => 0xbc48,
                    "Rio_4" => 0xbcb7,
                    _ => 0xbcff,
                },
            )?;
        }
        "Rio_6" => rio_animation(r, e, u16::try_from(a).map_err(|_| "invalid Rio instruction")?),
        "Waver_Init" => {
            let right = r.word(e + PARAM1) & 1 != 0;
            set(r, e, B, if right { 1 } else { 0xfffe });
            set(r, e, A, 0x8000);
            for field in [F, C, E] {
                set(r, e, field, 0);
            }
            set(r, e, INSTRUCTION, 0x86a7);
            set(r, e, F, r.word(e + PARAM1) & 1);
            waver_animation(r, rom, e)?;
        }
        "Waver_Main" => waver_main(r, rom, host, k)?,
        "Waver_Func_1" => waver_animation(r, rom, e)?,
        _ => return Err("unknown flying enemy routine"),
    }
    Ok(0)
}
