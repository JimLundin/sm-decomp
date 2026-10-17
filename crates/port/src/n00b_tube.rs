//! Maridia tube fragments and air bubbles, pinned bank86 D5E1–D8DF.
use crate::Wram;

fn slot(k: u16) -> Result<usize, &'static str> {
    if k > 34 || k & 1 != 0 { Err("invalid enemy projectile slot") } else { Ok(usize::from(k)) }
}

fn word(rom: &[u8], p: u16) -> Result<u16, &'static str> {
    if p < 0x8000 || p == 0xffff {
        return Err("tube ROM extent");
    }
    let offset = 0x30000 + usize::from(p & 0x7fff);
    let value = rom.get(offset..offset + 2).ok_or("missing tube ROM")?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

/// Check initializer dependencies before allocation changes canonical WRAM.
pub fn validate(r: &Wram<'_>, rom: &[u8], init: u16, param: u16) -> Result<(), &'static str> {
    let plm = r.word(0x1c27);
    if plm > 78 || plm & 1 != 0 {
        return Err("invalid PLM slot");
    }
    let index = param / 2;
    match init {
        0xd6a5 => Ok(()),
        0xd6c9 if index < 10 => {
            word(rom, 0xd760 + index * 2)?;
            Ok(())
        }
        0xd774 if index < 6 => Ok(()),
        _ => Err("invalid tube initializer or parameter"),
    }
}

pub fn initialize(r: &mut Wram<'_>, rom: &[u8], init: u16, param: u16, k: u16) -> Result<(), &'static str> {
    let k = slot(k)?;
    validate(r, rom, init, param)?;
    crate::plm_render::calculate_block_coords(r, r.word(0x1c27))?;
    let x = r.word(0x1c29).wrapping_mul(16);
    let y = r.word(0x1c2b).wrapping_mul(16);
    let index = usize::from(param / 2);
    match init {
        0xd6a5 => {
            r.set_word(0x1a4b + k, x.wrapping_add(96));
            r.set_word(0x1a93 + k, y.wrapping_add(48));
        }
        0xd6c9 => {
            const X: [i16; 10] = [-56, -64, -20, -40, -64, -48, -24, -40, 0, -8];
            const Y: [i16; 10] = [8, -12, -26, -24, -32, 28, 16, -8, -24, 16];
            const X_VELOCITY: [i16; 10] = [-384, -384, -160, -288, -288, -320, -96, -352, 0, -64];
            const Y_VELOCITY: [i16; 10] = [320, -256, -416, -288, -288, 448, 576, -96, -288, 384];
            r.set_word(0x1b23 + k, x.wrapping_add(96).wrapping_add(X[index] as u16));
            r.set_word(0x1aff + k, 0);
            r.set_word(0x1a93 + k, y.wrapping_add(48).wrapping_add(Y[index] as u16));
            r.set_word(0x1b47 + k, word(rom, 0xd760 + index as u16 * 2)?);
            r.set_word(0x1ab7 + k, X_VELOCITY[index] as u16);
            r.set_word(0x1adb + k, Y_VELOCITY[index] as u16);
        }
        0xd774 => {
            const X: [u16; 6] = [40, 80, 104, 120, 152, 184];
            const Y: [u16; 6] = [80, 72, 84, 32, 64, 84];
            r.set_word(0x1b23 + k, x.wrapping_add(X[index]));
            r.set_word(0x1aff + k, 0);
            r.set_word(0x1a93 + k, y.wrapping_add(Y[index]));
            r.set_word(0x1adb + k, (-1280_i16) as u16);
        }
        _ => unreachable!(),
    }
    Ok(())
}

/// Zero stops this frame; other returns are bank86 continuations.
pub fn instruction(r: &mut Wram<'_>, rom: &[u8], opcode: u16, pc: u16, k: u16) -> Result<u16, &'static str> {
    let k = slot(k)?;
    match opcode {
        0xd5e1 | 0xd69a => {
            crate::enemy_projectiles::random(r);
            // C reads an unaligned word starting at the RNG's high byte.
            r.set_word(0x1ab7 + k, r.word(0x5e6));
            if opcode == 0xd5e1 {
                r.set_word(0x1adb + k, 192);
            }
            Ok(pc)
        }
        0xd5f2 | 0xd62a => {
            let odd = r.word(0x5b6) & 1 != 0;
            let f = r.word(0x1b23 + k);
            let (x, map, next) = if opcode == 0xd5f2 {
                (
                    if odd { f } else { 256_u16.wrapping_sub(f) },
                    word(rom, pc.wrapping_add(if odd { 0 } else { 2 }))?,
                    pc.wrapping_add(4),
                )
            } else {
                (if odd { f } else { 0xee00 }, word(rom, pc)?, pc.wrapping_add(2))
            };
            r.set_word(0x1a4b + k, x);
            r.set_word(0x1b6b + k, map);
            r.set_word(0x1b47 + k, next);
            r.set_word(0x1b8f + k, 1);
            Ok(0)
        }
        _ => Err("unsupported tube instruction"),
    }
}

fn add_velocity(r: &mut Wram<'_>, k: usize, high: usize, low: usize, velocity: u16) {
    let value = (u32::from(r.word(high + k)) << 16) | u32::from(r.word(low + k));
    let delta = i32::from(velocity as i16) * 256;
    let result = value.wrapping_add(delta as u32);
    r.set_word(high + k, (result >> 16) as u16);
    r.set_word(low + k, result as u16);
}

fn delete_below_camera(r: &mut Wram<'_>, k: usize) {
    if r.word(0x1a93 + k).wrapping_sub(r.word(0x915)) as i16 >= 288 {
        r.set_word(0x1997 + k, 0);
    }
}

pub fn preinstruction(r: &mut Wram<'_>, _rom: &[u8], address: u16, k: u16) -> Result<(), &'static str> {
    let k = slot(k)?;
    match address {
        0xd7bf => {
            if r.word(0x1a4b + k) != 0xee00 {
                r.set_word(0x1aff + k, r.word(0x1a4b + k));
            }
            r.set_word(0x1a4b + k, if r.word(0x5b6) & 1 != 0 { 0xee00 } else { r.word(0x1aff + k) });
        }
        0xd7de => add_velocity(r, k, 0x1a93, 0x1a6f, 192),
        0xd7fd => {
            add_velocity(r, k, 0x1b23, 0x1aff, r.word(0x1ab7 + k));
            add_velocity(r, k, 0x1a93, 0x1a6f, r.word(0x1adb + k));
            delete_below_camera(r, k);
        }
        0xd83d | 0xd89f => {
            // Native callbacks select phase from k but update the global slot.
            let active = slot(r.word(0x1991))?;
            let angle = usize::from((r.word(0x1ab7 + k) & 0x17e | 0x80) >> 1);
            let velocity = (crate::enemy_projectiles::SINE[angle + 64] >> 2) as u16;
            add_velocity(r, active, 0x1b23, 0x1aff, velocity);
            r.set_word(0x1ab7 + active, r.word(0x1ab7 + active).wrapping_add(if address == 0xd83d { 2 } else { 4 }));
            add_velocity(r, active, 0x1a93, 0x1a6f, r.word(0x1adb + active));
            if address == 0xd83d {
                delete_below_camera(r, active);
            } else {
                r.set_word(0x1a4b + active, r.word(0x1b23 + active));
            }
        }
        0xd8df => {
            add_velocity(r, k, 0x1a93, 0x1a6f, r.word(0x1adb + k));
            r.set_word(0x1a4b + k, r.word(0x1b23 + k));
        }
        _ => return Err("unsupported tube preinstruction"),
    }
    Ok(())
}
