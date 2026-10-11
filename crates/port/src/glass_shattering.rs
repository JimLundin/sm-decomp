//! Mother Brain's tank glass: PLM callers, shard arcs and sparkle allocation.
use crate::{Wram, enemy_projectiles};

const SHARD: u16 = 0xcefc;
const SPARKLE: u16 = 0xcf0a;

fn slot(k: u16) -> Result<usize, &'static str> {
    if k <= 34 && k & 1 == 0 { Ok(usize::from(k)) } else { Err("invalid glass projectile slot") }
}

fn word(rom: &[u8], bank: usize, address: u16) -> Result<u16, &'static str> {
    if address < 0x8000 || address == 0xffff {
        return Err("glass ROM address extent");
    }
    let offset = bank + usize::from(address & 0x7fff);
    let value = rom.get(offset..offset + 2).ok_or("missing glass ROM")?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

pub fn validate(r: &Wram<'_>, rom: &[u8], address: u16, parameter: u16) -> Result<(), &'static str> {
    match address {
        0xcdc5 => {
            if parameter >> 1 >= 3 {
                return Err("glass shard offset extent");
            }
            let plm = r.word(0x1c27);
            if plm > 78 || plm & 1 != 0 {
                return Err("invalid PLM slot");
            }
            for address in (0xce41..0xce61).step_by(2) {
                word(rom, 0x30000, address)?;
            }
        }
        // The C initializer divides its source parameter by two, including
        // odd parameters. Preserve that alias while bounding the source array.
        0xce6d if parameter >> 1 < 18 => {}
        0xce6d => return Err("glass sparkle source extent"),
        _ => return Err("unsupported glass initializer"),
    }
    Ok(())
}

pub fn initialize(r: &mut Wram<'_>, rom: &[u8], address: u16, k: u16, parameter: u16) -> Result<(), &'static str> {
    let k = slot(k)?;
    validate(r, rom, address, parameter)?;
    if address == 0xce6d {
        let source = usize::from(parameter & !1);
        let x = r.word(0x1a4b + source).wrapping_add(enemy_projectiles::random(r) & 31).wrapping_sub(16);
        r.set_word(0x1a4b + k, x);
        let y = r.word(0x1a93 + source).wrapping_add(enemy_projectiles::random(r) & 31).wrapping_sub(16);
        r.set_word(0x1a93 + k, y);
        r.set_word(0x19bb + k, 1600);
        return Ok(());
    }

    let phase = enemy_projectiles::random(r).wrapping_mul(2) & 0x1fe;
    r.set_word(0x1aff + k, phase);
    let angle = usize::from(phase >> 1);
    r.set_word(0x1ab7 + k, enemy_projectiles::SINE[angle + 64] as u16);
    r.set_word(0x1adb + k, (enemy_projectiles::SINE[angle] as u16).wrapping_mul(4));
    r.set_word(0x1b47 + k, word(rom, 0x30000, 0xce41 + ((phase >> 4) & 0x1e))?);
    r.set_word(0x19bb + k, 1600);
    crate::plm_render::calculate_block_coords(r, r.word(0x1c27))?;
    let x =
        r.word(0x1c29).wrapping_mul(16).wrapping_add([8, -40i16 as u16, -16i16 as u16][usize::from(parameter >> 1)]);
    let y = r.word(0x1c2b).wrapping_mul(16).wrapping_add(32);
    r.set_word(0x1a4b + k, x);
    r.set_word(0x1a93 + k, y);
    let x = x.wrapping_add(enemy_projectiles::random(r) & 15).wrapping_sub(8);
    r.set_word(0x1a4b + k, x);
    let y = y.wrapping_add(enemy_projectiles::random(r) & 15).wrapping_sub(8);
    r.set_word(0x1a93 + k, y);
    Ok(())
}

fn move_axis(r: &mut Wram<'_>, k: usize, position: usize, fraction: usize, velocity: usize) {
    let current = (u32::from(r.word(position + k)) << 16) | u32::from(r.word(fraction + k));
    let delta = (i32::from(r.word(velocity + k) as i16) * 256) as u32;
    let next = current.wrapping_add(delta);
    r.set_word(position + k, (next >> 16) as u16);
    r.set_word(fraction + k, next as u16);
}

pub fn update(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<(), &'static str> {
    let index = slot(k)?;
    move_axis(r, index, 0x1a4b, 0x1a27, 0x1ab7);
    move_axis(r, index, 0x1a93, 0x1a6f, 0x1adb);
    if r.word(0x1a93 + index) & 0xff00 != 0 {
        r.set_word(0x1997 + index, 0);
    } else {
        r.set_word(0x1adb + index, r.word(0x1adb + index).wrapping_add(32));
        if enemy_projectiles::random(r) & 0x420 == 0 {
            enemy_projectiles::spawn_room(r, rom, SPARKLE, k)?;
        }
    }
    Ok(())
}

pub fn spawn_shard(r: &mut Wram<'_>, rom: &[u8], parameter: u16) -> Result<(), &'static str> {
    enemy_projectiles::spawn_room(r, rom, SHARD, parameter)
}

pub fn spawn_four(r: &mut Wram<'_>, rom: &[u8], parameters: [u16; 4]) -> Result<(), &'static str> {
    crate::audio::dispatch(r, "QueueSfx3_Max15", 0x2e, 0)?;
    for parameter in parameters {
        spawn_shard(r, rom, parameter)?;
    }
    Ok(())
}

/// `pc` points to the four original bank-84 operand words, after the opcode.
pub fn instruction(r: &mut Wram<'_>, rom: &[u8], pc: u16) -> Result<u16, &'static str> {
    let end = pc.checked_add(8).ok_or("glass PLM instruction extent")?;
    let parameters = [
        word(rom, 0x20000, pc)?,
        word(rom, 0x20000, pc + 2)?,
        word(rom, 0x20000, pc + 4)?,
        word(rom, 0x20000, pc + 6)?,
    ];
    spawn_four(r, rom, parameters)?;
    Ok(end)
}
