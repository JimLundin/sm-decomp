//! Bank-B4 sprite object script scheduling, preserving the original WRAM aliases.
use crate::Wram;

const INDEX: usize = 0x1844;
const LIST: usize = 0xef78;
const TIMER: usize = 0xeff8;

fn draw_map(
    r: &mut Wram<'_>,
    rom: &[u8],
    pointer: u16,
    x: u16,
    y: u16,
    palette: u16,
    tile: u16,
) -> Result<(), &'static str> {
    if pointer == 0 {
        return Ok(());
    }
    let count = usize::from(word(rom, pointer)?);
    let offset = 0x1a0000 + usize::from(pointer & 0x7fff) + 2;
    let entries = rom.get(offset..offset + count * 5).ok_or("sprite spritemap extent")?;
    let mut index = usize::from(r.word(0x590));
    if index > 508 || index & 3 != 0 {
        return Err("invalid OAM cursor");
    }
    for entry in entries.as_chunks::<5>().0 {
        let dx = u16::from_le_bytes([entry[0], entry[1]]);
        let position = x.wrapping_add(dx);
        r.set_byte(0x370 + index, position as u8);
        r.set_byte(0x371 + index, y.wrapping_add(u16::from(entry[2])) as u8);
        r.set_word(0x372 + index, palette | tile.wrapping_add(u16::from_le_bytes([entry[3], entry[4]])));
        let extension = ((position & 0x100) >> 8) | (u16::from(dx & 0x8000 != 0) * 2);
        let address = 0x570 + (index >> 5) * 2;
        r.set_word(address, r.word(address) | (extension << (2 * ((index >> 2) & 7))));
        index = (index + 4) & 0x1ff;
    }
    r.set_word(0x590, index as u16);
    Ok(())
}

pub fn draw(r: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    for index in (0..64).step_by(2).rev() {
        let pointer = r.word(LIST + index);
        if pointer == 0 {
            continue;
        }
        let x = r.word(0xf0f8 + index).wrapping_sub(r.word(0x911));
        let y = r.word(0xf1f8 + index).wrapping_sub(r.word(0x915));
        if (x.wrapping_add(16) as i16) >= 0
            && (x.wrapping_sub(272) as i16) < 0
            && (y as i16) >= 0
            && (y.wrapping_sub(272) as i16) < 0
        {
            let palette = r.word(0xf078 + index);
            draw_map(r, rom, word(rom, pointer.wrapping_add(2))?, x, y, palette & 0xe00, palette & 0x1ff)?;
        }
    }
    Ok(())
}

pub fn create(r: &mut Wram<'_>, rom: &[u8], x: u16, y: u16, list: u16, palette: u16) -> Result<u16, &'static str> {
    let Some(index) = (0..64).step_by(2).rev().find(|&index| r.word(LIST + index) == 0) else {
        return Ok(0xffff);
    };
    let table_address = 0xbda8u16
        .checked_add(list.checked_mul(2).ok_or("sprite list index extent")?)
        .ok_or("sprite list index extent")?;
    let pointer = word(rom, table_address)?;
    let timer = word(rom, pointer)?;
    r.set_word(0xf178 + index, 0);
    r.set_word(0xf278 + index, 0);
    r.set_word(0xf2f8 + index, 0);
    r.set_word(0xf0f8 + index, x);
    r.set_word(0xf1f8 + index, y);
    r.set_word(0xf078 + index, palette);
    r.set_word(LIST + index, pointer);
    r.set_word(TIMER + index, timer);
    Ok(index as u16)
}

fn slot(r: &Wram<'_>) -> Result<usize, &'static str> {
    let index = r.word(INDEX);
    if index > 62 || index & 1 != 0 {
        return Err("invalid sprite object slot");
    }
    Ok(usize::from(index))
}

fn word(rom: &[u8], address: u16) -> Result<u16, &'static str> {
    if address < 0x8000 || address == 0xffff {
        return Err("sprite object ROM extent");
    }
    let offset = 0x1a0000 + usize::from(address & 0x7fff);
    let bytes = rom.get(offset..offset + 2).ok_or("missing sprite object ROM")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn instruction(r: &mut Wram<'_>, rom: &[u8], opcode: u16) -> Result<(), &'static str> {
    let index = slot(r)?;
    match opcode {
        0xbcf0 => {
            r.set_word(LIST + index, r.word(LIST + index).wrapping_sub(4));
            r.set_word(TIMER + index, 0x7fff);
        }
        0xbd07 => r.set_word(LIST + index, 0),
        0xbd12 => {
            let target = word(rom, r.word(LIST + index).wrapping_add(2))?;
            let timer = word(rom, target)?;
            r.set_word(LIST + index, target);
            r.set_word(TIMER + index, timer);
        }
        _ => return Err("unsupported sprite object instruction"),
    }
    Ok(())
}

pub fn handle(r: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    if r.word(0x185e) | r.word(0xa78) != 0 {
        return Ok(());
    }
    r.set_word(INDEX, 62);
    loop {
        let index = slot(r)?;
        if r.word(LIST + index) != 0 && r.word(0xf2f8 + index) & 1 == 0 {
            let timer = r.word(TIMER + index);
            if timer & 0x8000 != 0 {
                instruction(r, rom, timer)?;
            } else {
                r.set_word(TIMER + index, timer.wrapping_sub(1));
                if timer == 1 {
                    let next = r.word(LIST + index).wrapping_add(4);
                    r.set_word(LIST + index, next);
                    let timer = word(rom, next)?;
                    if timer & 0x8000 != 0 {
                        instruction(r, rom, timer)?;
                    } else {
                        r.set_word(TIMER + index, timer);
                    }
                }
            }
        }
        let next = r.word(INDEX).wrapping_sub(2);
        r.set_word(INDEX, next);
        if next & 0x8000 != 0 {
            return Ok(());
        }
    }
}

pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], name: &str, a: i32, b: i32) -> Result<u32, &'static str> {
    match name {
        "CreateSpriteAtPos" => {
            return create(r, rom, a as u16, (a as u32 >> 16) as u16, b as u16, (b as u32 >> 16) as u16).map(u32::from);
        }
        "HandleSpriteObjects" => handle(r, rom)?,
        "DrawSpriteObjects" => draw(r, rom)?,
        "SpriteObject_Instr_RepeatLast" => instruction(r, rom, 0xbcf0)?,
        "SpriteObject_Instr_Terminate" => instruction(r, rom, 0xbd07)?,
        "SpriteObject_Instr_Goto" => instruction(r, rom, 0xbd12)?,
        "ClearSpriteObjects" => {
            // The original clears words 1..511, overlapping the other sprite arrays.
            // Word zero is deliberately preserved.
            for offset in (2..=1022).step_by(2) {
                r.set_word(LIST + offset, 0);
            }
        }
        _ => return Err("unknown sprite object routine"),
    }
    Ok(0)
}
