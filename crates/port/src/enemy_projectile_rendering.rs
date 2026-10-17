//! Enemy projectile priority passes and bank-81 OAM spritemap emission.
//! Translated from pinned snesrev/sm bank86 8390..846B and bank81 8C0A/8C7F.
use crate::Wram;

const OAM: usize = 0x370;
const OAM_EXT: usize = 0x570;
const OAM_NEXT: usize = 0x590;

fn rom_offset(bank: u8, pointer: u16) -> usize {
    ((usize::from(bank) << 15) | usize::from(pointer & 0x7fff)) & 0x3fffff
}

fn word(rom: &[u8], offset: usize) -> Result<u16, &'static str> {
    let bytes = rom.get(offset..offset + 2).ok_or("enemy projectile rendering ROM extent")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

/// Bank86 8427 returns X/Y offsets without modifying the earthquake timer.
/// The signed comparison and wrapped table multiplication also preserve high
/// earthquake-type values, which can select entries beyond the usual 36 pairs.
pub fn screen_shake(r: &Wram<'_>, rom: &[u8]) -> Result<(u16, u16), &'static str> {
    let timer = r.word(0x1840);
    let kind = r.word(0x183e);
    if timer == 0 || r.word(0xa78) != 0 || (kind.wrapping_sub(36) as i16) >= 0 {
        return Ok((0, 0));
    }
    let offset = rom_offset(0x86, 0x846b) + usize::from(kind.wrapping_mul(4));
    let x = word(rom, offset)?;
    let y = word(rom, offset + 2)?;
    Ok(if timer & 2 == 0 { (x, y) } else { (x.wrapping_neg(), y.wrapping_neg()) })
}

/// Emit bank81 8C0A/8C7F entries. Vertical offsets are unsigned additions whose
/// sign bit selects the permitted carry, rather than ordinary signed clipping.
#[allow(clippy::too_many_arguments)]
pub fn draw_spritemap(
    r: &mut Wram<'_>,
    rom: &[u8],
    bank: u8,
    pointer: u16,
    x: u16,
    y: u16,
    tile: u16,
    attributes: u16,
    offscreen: bool,
) -> Result<(), &'static str> {
    if pointer < 0x8000 {
        return Err("enemy projectile spritemap below banked ROM");
    }
    let offset = rom_offset(bank, pointer);
    let count = usize::from(word(rom, offset)?);
    let entries = rom.get(offset + 2..offset + 2 + count * 5).ok_or("enemy projectile spritemap extent")?;
    let mut index = usize::from(r.word(OAM_NEXT));
    for entry in entries.as_chunks::<5>().0 {
        let dx = u16::from_le_bytes([entry[0], entry[1]]);
        let position = x.wrapping_add(dx);
        r.set_byte(OAM + index, position as u8);
        let extension = ((position & 0x100) >> 8) | (u16::from(dx & 0x8000 != 0) * 2);
        let address = OAM_EXT + (index >> 5) * 2;
        r.set_word(address, r.word(address) | (extension << (2 * ((index >> 2) & 7))));
        let vertical = u16::from(entry[2]) + u16::from(y as u8);
        let carry = vertical & 0x100 != 0;
        let negative = entry[2] & 0x80 != 0;
        let visible = (carry == negative) != offscreen;
        r.set_byte(OAM + index + 1, if visible { vertical as u8 } else { 0xf0 });
        let character = u16::from_le_bytes([entry[3], entry[4]]);
        r.set_word(OAM + index + 2, attributes | tile.wrapping_add(character));
        index = (index + 4) & 0x1ff;
    }
    r.set_word(OAM_NEXT, index as u16);
    Ok(())
}

/// Bank86 83D6 draws a slot with the pass's already-captured quake offsets.
pub fn draw_projectile(r: &mut Wram<'_>, rom: &[u8], k: u16, shake: (u16, u16)) -> Result<(), &'static str> {
    // C indexes word arrays with k >> 1, including odd direct-call arguments.
    let index = usize::from(k & !1);
    let pointer = r.word(0x1b6b + index);
    let gfx = r.word(0x19bb + index);
    let x = shake.0.wrapping_add(r.word(0x1a4b + index)).wrapping_sub(r.word(0x911));
    if x.wrapping_add(128) & 0xfe00 != 0 {
        return Ok(());
    }
    let y = shake.1.wrapping_add(r.word(0x1a93 + index)).wrapping_sub(r.word(0x915));
    let offscreen = y & 0xff00 != 0;
    if offscreen && y.wrapping_add(128) & 0xfe00 != 0 {
        return Ok(());
    }
    draw_spritemap(r, rom, 0x8d, pointer, x, y, gfx & 0xff, gfx & 0xff00, offscreen)
}

/// Capture quake once and traverse the eighteen-slot pool from high to low.
pub fn draw_priority(r: &mut Wram<'_>, rom: &[u8], low: bool) -> Result<(), &'static str> {
    let shake = screen_shake(r, rom)?;
    for index in (0..36).step_by(2).rev() {
        if r.word(0x1997 + index) != 0 && (r.word(0x1bd7 + index) & 0x1000 != 0) == low {
            draw_projectile(r, rom, index as u16, shake)?;
        }
    }
    Ok(())
}

pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], symbol: &str, a: i32, b: i32) -> Result<u32, &'static str> {
    match symbol {
        "DrawLowPriorityEprojs" => draw_priority(r, rom, true)?,
        "DrawHighPriorityEprojs" => draw_priority(r, rom, false)?,
        "PortScreenShake" => {
            let (x, y) = screen_shake(r, rom)?;
            return Ok(u32::from(x) | (u32::from(y) << 16));
        }
        "PortDrawEprojs" => draw_projectile(r, rom, a as u16, (b as u16, (b as u32 >> 16) as u16))?,
        "PortDrawEprojSpritemap" | "PortDrawEprojSpritemapOffscreen" => {
            let gfx = (b as u32 >> 16) as u16;
            draw_spritemap(
                r,
                rom,
                0x8d,
                b as u16,
                a as u16,
                (a as u32 >> 16) as u16,
                gfx & 0xff,
                gfx & 0xff00,
                symbol == "PortDrawEprojSpritemapOffscreen",
            )?;
        }
        _ => return Err("unknown enemy projectile rendering routine"),
    }
    Ok(0)
}
