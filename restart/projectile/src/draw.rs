//! Bank $93 passes and bank $81 OAM emission. PPU transfer stays in the host.
use crate::{Result, Wram, core::*, rw, trails};

fn spritemap(r: &mut Wram<'_>, rom: &[u8], pointer: u16, x: u16, y: u16) -> Result<()> {
    if pointer < 0x8000 {
        return Err("projectile spritemap below banked ROM".into());
    }
    let offset = 0x98000 + usize::from(pointer & 0x7fff);
    let count = usize::from(rw(rom, 0x930000 | u32::from(pointer))?);
    let entries = rom.get(offset + 2..offset + 2 + count * 5).ok_or("spritemap records outside ROM")?;
    let mut cursor = usize::from(r.word(0x590));
    for entry in entries.as_chunks::<5>().0 {
        let dx = u16::from_le_bytes([entry[0], entry[1]]);
        let screen_x = x.wrapping_add(dx);
        r.set_byte(0x370 + cursor, screen_x as u8);
        let extension = ((screen_x & 0x100) >> 8) | (u16::from(dx & 0x8000 != 0) * 2);
        let address = 0x570 + (cursor >> 5) * 2;
        r.set_word(address, r.word(address) | (extension << (2 * ((cursor >> 2) & 7))));
        r.set_byte(0x370 + cursor + 1, y.wrapping_add(u16::from(entry[2])) as u8);
        r.set_word(0x370 + cursor + 2, u16::from_le_bytes([entry[3], entry[4]]));
        cursor = (cursor + 4) & 0x1ff;
    }
    r.set_word(0x590, cursor as u16);
    Ok(())
}

pub(crate) fn player(r: &mut Wram<'_>, rom: &[u8]) -> Result<()> {
    let mut raw = 8u16;
    r.set_word(INDEX, raw);
    for _ in 0..1024 {
        let k = slot(raw)?;
        let kind = r.word(TYPE + k);
        let visible = kind & 0xf00 == 0 && ((raw & 2 != 0) != (r.word(0x5b6) & 1 != 0));
        if r.word(INSTRUCTION + k) != 0 && visible {
            let x = r.word(X + k).wrapping_sub(r.word(0x911));
            let y = r.word(Y + k).wrapping_sub(r.word(0x915));
            let map = r.word(SPRITEMAP + k);
            if y & 0xff00 == 0 && map & 0x8000 != 0 {
                spritemap(r, rom, map, x, y)?;
            }
            raw = r.word(INDEX);
        }
        raw = raw.wrapping_sub(2);
        r.set_word(INDEX, raw);
        if raw & 0x8000 != 0 {
            return trails::frame(r, rom);
        }
    }
    Err("cyclic projectile draw pass".into())
}

pub(crate) fn explosions(r: &mut Wram<'_>, rom: &[u8]) -> Result<()> {
    let mut raw = 18u16;
    r.set_word(INDEX, raw);
    for _ in 0..1024 {
        let k = slot(raw)?;
        if r.word(INSTRUCTION + k) != 0 && r.word(TYPE + k) & 0xf00 == 0x700 {
            let x = r.word(X + k).wrapping_sub(r.word(0x911));
            if (x.wrapping_sub(304) as i16) < 0 && (x.wrapping_add(48) as i16) >= 0 {
                let y = r.word(Y + k).wrapping_sub(r.word(0x915));
                if y & 0xff00 == 0 {
                    spritemap(r, rom, r.word(SPRITEMAP + k), x, y)?;
                }
                raw = r.word(INDEX);
            }
        }
        raw = raw.wrapping_sub(2);
        r.set_word(INDEX, raw);
        if raw & 0x8000 != 0 {
            return Ok(());
        }
    }
    Err("cyclic projectile explosion draw pass".into())
}
