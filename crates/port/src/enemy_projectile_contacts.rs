//! Bank-A0 contacts between enemy projectiles, Samus and player projectiles.
use crate::{Wram, audio, sprite_objects};

fn enemy_slot(k: u16) -> Result<usize, &'static str> {
    if k <= 34 && k & 1 == 0 { Ok(usize::from(k)) } else { Err("invalid enemy projectile contact slot") }
}

fn definition_word(rom: &[u8], id: u16, field: usize) -> Result<u16, &'static str> {
    if id < 0x8000 || usize::from(id) + field + 1 > 0xffff {
        return Err("enemy projectile contact definition extent");
    }
    let offset = 0x30000 + usize::from(id & 0x7fff) + field;
    let bytes = rom.get(offset..offset + 2).ok_or("missing enemy projectile contact ROM")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn damage(r: &mut Wram<'_>, amount: u16) {
    let items = r.word(0x9a2);
    let amount = if items & 0x20 != 0 {
        amount >> 2
    } else if items & 1 != 0 {
        amount >> 1
    } else {
        amount
    };
    // Samus_DealDamage reserves exactly 300 as a no-damage value, including
    // when suit division produces it. Contact damage is at most 0xfff.
    if amount != 300 && r.word(0xa78) == 0 {
        let health = r.word(0x9c2).wrapping_sub(amount);
        r.set_word(0x9c2, if health & 0x8000 != 0 { 0 } else { health });
    }
}

pub fn hit_samus(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<(), &'static str> {
    let k = enemy_slot(k)?;
    r.set_word(0x18a8, 96);
    r.set_word(0x18aa, 5);
    let list = definition_word(rom, r.word(0x1997 + k), 10)?;
    if list != 0 {
        r.set_word(0x1b47 + k, list);
        r.set_word(0x1b8f + k, 1);
    }
    let properties = r.word(0x1bd7 + k);
    if properties & 0x4000 == 0 {
        r.set_word(0x1997 + k, 0);
    }
    damage(r, properties & 0xfff);
    r.set_word(0xa54, u16::from(r.word(0xaf6).wrapping_sub(r.word(0x1a4b + k)) as i16 >= 0));
    Ok(())
}

fn abs16(value: u16) -> u16 {
    if (value as i16) < 0 { value.wrapping_neg() } else { value }
}

pub fn detect_samus(r: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    r.set_word(0xf378, 11);
    if r.word(0x18a8) != 0 || r.word(0xa6e) != 0 {
        return Ok(());
    }
    for k in (0..36).step_by(2).rev() {
        let radius = r.word(0x1bb3 + k);
        if r.word(0x1997 + k) == 0 || r.word(0x1bd7 + k) & 0x2000 != 0 || radius == 0 {
            continue;
        }
        // abs16 and the radii are uint16, promoted to signed int by C before
        // subtraction. A negative difference remains an overlapping distance.
        if i32::from(abs16(r.word(0xaf6).wrapping_sub(r.word(0x1a4b + k)))) - i32::from(r.word(0xafe))
            < i32::from(radius & 255)
            && i32::from(abs16(r.word(0xafa).wrapping_sub(r.word(0x1a93 + k)))) - i32::from(r.word(0xb00))
                < i32::from(radius >> 8)
        {
            r.set_word(0x18a6, k as u16);
            // Native detection checks invincibility only at entry and keeps
            // visiting the lower slots after a hit changes the timer.
            hit_samus(r, rom, k as u16)?;
        }
    }
    Ok(())
}

pub fn hit_projectile(r: &mut Wram<'_>, rom: &[u8], k: u16, j: u16) -> Result<(), &'static str> {
    let k = enemy_slot(k)?;
    if j > 8 || j & 1 != 0 {
        return Err("invalid player projectile contact slot");
    }
    let j = usize::from(j);
    let projectile_type = r.word(0xc18 + j);
    if projectile_type & 8 == 0 {
        r.set_word(0xc04 + j, r.word(0xc04 + j) | 0x10);
    }
    if r.word(0xf380 + k) == 1 {
        // The pinned C uses enemy k, not player j, for this position read.
        // The arrays are aliases into full WRAM: k=20..34 intentionally reads
        // beyond the nominal ten player projectile positions.
        sprite_objects::create(r, rom, r.word(0xb64 + k), r.word(0xb78 + k), 6, 0)?;
        audio::queue_sfx(r, 0, 0x3d06)?;
    } else {
        r.set_word(0x1bfb + k, projectile_type);
        let list = definition_word(rom, r.word(0x1997 + k), 12)?;
        r.set_word(0x1b47 + k, list);
        r.set_word(0x1b8f + k, 1);
        r.set_word(0x1a03 + k, 0x84fb);
        r.set_word(0x1bd7 + k, r.word(0x1bd7 + k) & 0xfff);
    }
    Ok(())
}

pub fn detect_projectiles(r: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    r.set_word(0xf378, 12);
    if r.word(0xcce) == 0 {
        return Ok(());
    }
    for k in (0..36).step_by(2).rev() {
        if r.word(0x1997 + k) == 0 || r.word(0x1bd7 + k) & 0x8000 == 0 {
            continue;
        }
        for j in (0..10).step_by(2) {
            if r.word(0xf380 + k) == 2 {
                break;
            }
            let projectile_type = r.word(0xc18 + j);
            let kind = projectile_type & 0xf00;
            if projectile_type != 0
                && kind != 0x300
                && kind != 0x500
                && (kind.wrapping_sub(0x700) as i16) < 0
                && r.word(0x1a4b + k) & 0xffe0 == r.word(0xb64 + j) & 0xffe0
                && r.word(0x1a93 + k) & 0xffe0 == r.word(0xb78 + j) & 0xffe0
            {
                hit_projectile(r, rom, k as u16, j as u16)?;
            }
        }
    }
    Ok(())
}

pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], symbol: &str, a: i32, b: i32) -> Result<u32, &'static str> {
    match symbol {
        "EprojSamusCollDetect" => detect_samus(r, rom)?,
        "HandleEprojCollWithSamus" => hit_samus(r, rom, a as u16)?,
        "EprojProjCollDet" => detect_projectiles(r, rom)?,
        "HandleEprojCollWithProj" => hit_projectile(r, rom, a as u16, b as u16)?,
        _ => return Err("unknown enemy projectile contact routine"),
    }
    Ok(0)
}
