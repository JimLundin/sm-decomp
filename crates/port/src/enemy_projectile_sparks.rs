//! Bank86 sparks: enemy-linked allocation, fractional gravity, bounce and trails.
use crate::{Wram, enemy_projectile_collision, enemy_projectiles, sprite_objects};

fn slot(k: u16) -> Result<usize, &'static str> {
    if k > 34 || k & 1 != 0 { Err("invalid sparks projectile slot") } else { Ok(usize::from(k)) }
}

pub fn initialize(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<(), &'static str> {
    let k = slot(k)?;
    let enemy = r.word(0xe54);
    if enemy > 0x7c0 || enemy & 63 != 0 {
        return Err("invalid sparks enemy slot");
    }
    // Eight pairs are addressed, including the ROM instructions after the
    // fourteen-word local C constant. Preserve that native ROM-defined pair.
    let table = rom.get(0x373d4..0x373f4).ok_or("missing sparks velocity ROM")?;
    let enemy = usize::from(enemy);
    r.set_word(0x1b47 + k, 0xf353);
    r.set_word(0x1a4b + k, r.word(0xf7a + enemy));
    r.set_word(0x1a27 + k, r.word(0xf7c + enemy));
    r.set_word(0x1a93 + k, r.word(0xf7e + enemy).wrapping_add(8));
    r.set_word(0x1a6f + k, r.word(0xf80 + enemy));
    r.set_word(0x1ab7 + k, 0);
    r.set_word(0x1adb + k, 0);
    let offset = usize::from(enemy_projectiles::random(r) & 0x1c);
    r.set_word(0x1b23 + k, u16::from_le_bytes([table[offset], table[offset + 1]]));
    r.set_word(0x1aff + k, u16::from_le_bytes([table[offset + 2], table[offset + 3]]));
    Ok(())
}

fn add_position(r: &mut Wram<'_>, position: usize, fraction: usize, high: u16, low: u16) {
    let value = (u32::from(r.word(position)) << 16) | u32::from(r.word(fraction));
    let velocity = (u32::from(high) << 16) | u32::from(low);
    let value = value.wrapping_add(velocity);
    r.set_word(position, (value >> 16) as u16);
    r.set_word(fraction, value as u16);
}

pub fn update(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<(), &'static str> {
    let index = slot(k)?;
    if r.word(0x1adb + index) & 0x8000 == 0 {
        if enemy_projectile_collision::vertical(r, rom, k)? & 1 != 0 {
            r.set_word(0x1b47 + index, 0xf363);
            r.set_word(0x1b8f + index, 1);
            let horizontal =
                ((u32::from(r.word(0x1b23 + index)) << 16) | u32::from(r.word(0x1aff + index))).wrapping_mul(4);
            r.set_word(0x1b23 + index, (horizontal >> 16) as u16);
            r.set_word(0x1aff + index, horizontal as u16);
            r.set_word(0x1ab7 + index, 0x8000);
            r.set_word(0x1adb + index, 0xffff);
            r.set_word(0x1a93 + index, r.word(0x1a93 + index).wrapping_sub(2));
            return Ok(());
        }
        let low = r.word(0x1ab7 + index);
        r.set_word(0x1ab7 + index, low.wrapping_add(0x4000));
        let high = r.word(0x1adb + index).wrapping_add(u16::from(low >= 0xc000));
        if high < 4 {
            r.set_word(0x1adb + index, high);
        }
    }
    // The collision probe already performs its own 8.8 movement. C then adds
    // the full 16.16 velocity too; retaining both is observable in falling sparks.
    let (vertical_high, vertical_low) = (r.word(0x1adb + index), r.word(0x1ab7 + index));
    add_position(r, 0x1a93 + index, 0x1a6f + index, vertical_high, vertical_low);
    let (horizontal_high, horizontal_low) = (r.word(0x1b23 + index), r.word(0x1aff + index));
    add_position(r, 0x1a4b + index, 0x1a27 + index, horizontal_high, horizontal_low);
    if r.byte(0x5b5) & 3 == 0 {
        let (x, y, palette) = (r.word(0x1a4b + index), r.word(0x1a93 + index), r.word(0x19bb + index));
        sprite_objects::create(r, rom, x, y, 48, palette)?;
    }
    Ok(())
}
