//! Bank84 PLM bytecode control and draw-data decoding from pinned snesrev/sm.
use crate::{Wram, audio};
const HEADER: usize = 0x1c37;
const BLOCK: usize = 0x1c87;
const PRE: usize = 0x1cd7;
const LIST: usize = 0x1d27;
const TIMER: usize = 0x1d77;
const ARG: usize = 0x1dc7;
const VARIABLE: usize = 0x1e17;
const INSTR_TIMER: usize = 0xde1c;
const DRAW: usize = 0xde6c;
const LINK: usize = 0xdebc;
const HITS: usize = 0xdf0c;
fn slot(k: u16) -> Result<usize, &'static str> {
    if k > 78 || k & 1 != 0 { Err("invalid PLM slot") } else { Ok(k as usize) }
}
fn rom_bytes(rom: &[u8], bank: u8, pc: u16, n: usize) -> Result<&[u8], &'static str> {
    if pc < 0x8000 || pc as usize + n > 0x10000 {
        return Err("PLM bank extent");
    }
    let off = (bank as usize & 0x7f) * 0x8000 + (pc as usize & 0x7fff);
    rom.get(off..off + n).ok_or("PLM ROM extent")
}
fn word(rom: &[u8], bank: u8, pc: u16) -> Result<u16, &'static str> {
    let b = rom_bytes(rom, bank, pc, 2)?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}
fn next(pc: u16, n: u16) -> Result<u16, &'static str> {
    pc.checked_add(n).filter(|p| *p >= 0x8000).ok_or("PLM continuation extent")
}
fn bit(r: &mut Wram<'_>, value: u16, base: usize, count: usize) -> Result<usize, &'static str> {
    let index = (value >> 3) as usize;
    if index >= count {
        return Err("PLM bit array extent");
    }
    r.set_word(0x5e7, 1 << (value & 7));
    Ok(base + index)
}
fn block(r: &Wram<'_>, k: usize) -> Result<usize, &'static str> {
    let b = r.word(BLOCK + k) as usize;
    if b & 1 != 0 || b >= 0xc800 { Err("PLM block extent") } else { Ok(b) }
}
/// Execute one instruction. None ends the current PLM's frame.
pub fn instruction(r: &mut Wram<'_>, rom: &[u8], opcode: u16, pc: u16, k: u16) -> Result<Option<u16>, &'static str> {
    let k = slot(k)?;
    let w = |delta: u16| word(rom, 0x84, next(pc, delta)?);
    let byte = |delta: u16| -> Result<u8, &'static str> { Ok(rom_bytes(rom, 0x84, next(pc, delta)?, 1)?[0]) };
    let mut advance = 0;
    match opcode {
        0x86b4 => {
            r.set_word(LIST + k, pc.checked_sub(2).ok_or("sleep operand extent")?);
            return Ok(None);
        }
        0x86bc => {
            r.set_word(HEADER + k, 0);
            return Ok(None);
        }
        0x86c1 => {
            r.set_word(PRE + k, w(0)?);
            advance = 2;
        }
        0x86ca => r.set_word(PRE + k, 0x86d0),
        0x8724 => return Ok(Some(w(0)?)),
        0x873f => {
            let old = r.word(TIMER + k);
            r.set_word(TIMER + k, old.wrapping_sub(1));
            if old != 1 {
                return Ok(Some(w(0)?));
            }
            advance = 2;
        }
        0x874e => {
            r.set_byte(TIMER + k, byte(0)?);
            advance = 1;
        }
        0x87e5 => {
            let data = rom_bytes(rom, 0x84, pc, 7)?;
            let tail = r.word(0x330) as usize;
            if tail + 7 > 0x260 {
                return Err("VRAM queue extent");
            }
            for (i, value) in data.iter().enumerate() {
                r.set_byte(0xd0 + tail + i, *value);
            }
            r.set_word(0x330, (tail + 7) as u16);
            advance = 7;
        }
        0x880e => {
            let area = r.word(0x79f) as usize;
            if area >= 8 {
                return Err("boss area extent");
            }
            if r.byte(0xd828 + area) & byte(0)? != 0 {
                return Ok(Some(w(1)?));
            }
            advance = 3;
        }
        0x882d | 0x883e => {
            let p = bit(r, w(0)?, 0xd820, 8)?;
            if opcode == 0x883e {
                r.set_byte(p, r.byte(p) | r.word(0x5e7) as u8);
                advance = 2;
            } else {
                if r.byte(p) & r.word(0x5e7) as u8 != 0 {
                    return Ok(Some(w(2)?));
                }
                advance = 4;
            }
        }
        0x8848 | 0x8865 | 0x887c | 0x8899 | 0x8a72 => {
            let arg = r.word(ARG + k);
            let base = if opcode == 0x8848 || opcode == 0x8865 {
                0xd830
            } else if opcode == 0x8a72 {
                0xd8b0
            } else {
                0xd870
            };
            let sets = opcode == 0x8865 || opcode == 0x8899;
            // Chozo/item branches compute bitmask even for negative arguments.
            let p = if arg & 0x8000 == 0 {
                Some(bit(r, arg, base, 64)?)
            } else {
                if !sets && opcode != 0x8a72 {
                    r.set_word(0x5e7, 1 << (arg & 7));
                }
                None
            };
            if let Some(p) = p {
                if sets {
                    r.set_byte(p, r.byte(p) | r.word(0x5e7) as u8);
                } else if r.byte(p) & r.word(0x5e7) as u8 != 0 {
                    return Ok(Some(w(0)?));
                }
            }
            advance = if sets { 0 } else { 2 };
        }
        0x8a24 => {
            r.set_word(LINK + k, w(0)?);
            advance = 2;
        }
        0x8a2e => {
            r.set_word(LINK + k, next(pc, 2)?);
            return Ok(Some(w(0)?));
        }
        0x8a3a => return Ok(Some(r.word(LINK + k))),
        0x8a91 => {
            let hit = r.word(HITS + k).wrapping_add(1);
            r.set_word(HITS + k, hit);
            if (hit as u8) < byte(0)? {
                advance = 3;
            } else {
                let arg = r.word(ARG + k);
                if arg & 0x8000 == 0 {
                    let p = bit(r, arg, 0xd8b0, 64)?;
                    r.set_byte(p, r.byte(p) | r.word(0x5e7) as u8);
                    r.set_word(ARG + k, r.word(p) | 0x8000);
                }
                r.set_word(PRE + k, 0x8aa6);
                return Ok(Some(w(1)?));
            }
        }
        0x8acd => {
            let a = r.byte(ARG + k).wrapping_add(1);
            if a >= byte(0)? {
                r.set_word(ARG + k, 0xffff);
                r.set_word(PRE + k, 0x8ae0);
                return Ok(Some(w(1)?));
            }
            r.set_word(ARG + k, a as u16);
            advance = 3;
        }
        0x8af1 => {
            let b = block(r, k)?;
            r.set_byte(0x16402 + b / 2, byte(0)?);
            advance = 1;
        }
        0x8b55 | 0x8b93 => {
            let mut p = r.word(ARG + k);
            let b = block(r, k)?;
            r.set_word(VARIABLE + k, 0);
            for n in 0..129 {
                let data = rom_bytes(rom, 0x8f, p, 1)?;
                if data[0] & 0x80 != 0 {
                    r.set_word(
                        0x10002 + b,
                        r.word(0x10002 + b) & 0xfff | if opcode == 0x8b55 { 0x3000 } else { 0xb000 },
                    );
                    return Ok(Some(pc));
                }
                let data = rom_bytes(rom, 0x8f, p, 2)?;
                r.set_byte(0xcd20 + data[0] as usize, data[1]);
                p = next(p, 2)?;
                if n == 128 {
                    return Err("unterminated scroll updates");
                }
            }
        }
        0x8bd1 | 0x8bdd => {
            if opcode == 0x8bdd {
                for off in (0x619..0x639).step_by(2) {
                    r.set_word(off, 0);
                }
                r.set_word(0x63b, r.word(0x639));
                r.set_word(0x63f, 0);
                r.set_word(0x63d, 0);
            }
            audio::queue_music(r, byte(0)? as u16, 8, true)?;
            advance = 1;
        }
        0x8c07..=0x8c85 if (opcode - 0x8c07).is_multiple_of(9) => {
            let index = (opcode - 0x8c07) / 9;
            let quotas = [6, 15, 3, 9, 1];
            audio::queue_sfx(r, (index % 3) as usize, ((byte(0)? as u16) << 8) | quotas[(index / 3) as usize])?;
            advance = 1;
        }
        0x8d41 => {
            let b = block(r, k)? / 2;
            let width = r.word(0x7a5) as usize;
            if width == 0 {
                return Err("PLM room width zero");
            }
            r.set_word(0x1c29, (b % width) as u16);
            r.set_word(0x1c2b, (b / width) as u16);
            let abs = |a: u16, b: u16| {
                let delta = a.wrapping_sub(b) as i16;
                delta.wrapping_abs() as u8
            };
            if abs(r.word(0xaf6) >> 4, r.word(0x1c29)) <= byte(0)?
                && abs(r.word(0xafa) >> 4, r.word(0x1c2b)) <= byte(1)?
            {
                return Ok(Some(w(2)?));
            }
            advance = 4;
        }
        _ => return Err("unsupported PLM instruction"),
    }
    Ok(Some(next(pc, advance)?))
}
/// Decode the original row/column tile runs, including signed origin offsets.
pub fn process_draw_data(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<(), &'static str> {
    let k = slot(k)?;
    let origin = block(r, k)?;
    let mut dst = origin as i32;
    let mut pc = r.word(DRAW + k);
    let read = |r: &Wram<'_>, pc: u16| -> Result<u16, &'static str> {
        if pc & 0x8000 != 0 {
            word(rom, 0x84, pc)
        } else if pc < 0x1fff {
            Ok(r.word(pc as usize))
        } else {
            Err("PLM RAM draw extent")
        }
    };
    for _ in 0..4096 {
        let run = read(r, pc)?;
        pc = pc.checked_add(2).ok_or("draw pointer overflow")?;
        let stride = if run & 0x8000 != 0 { i32::from(r.word(0x7a5)) * 2 } else { 2 };
        for i in 0..(run as u8) {
            let target = dst + i32::from(i) * stride;
            if !(0..0xc800).contains(&target) || target & 1 != 0 {
                return Err("draw tile extent");
            }
            let tile = read(r, pc)?;
            pc = pc.checked_add(2).ok_or("draw pointer overflow")?;
            r.set_word(0x10002 + target as usize, tile);
        }
        let offset = read(r, pc)?;
        if offset == 0 {
            return Ok(());
        }
        dst = origin as i32
            + 2 * (i32::from(offset as u8 as i8) + i32::from((offset >> 8) as u8 as i8) * i32::from(r.word(0x7a5)));
        pc = pc.checked_add(2).ok_or("draw pointer overflow")?;
    }
    Err("unterminated draw data")
}
/// Bounded control-only frame path. Rendering/message/preinstruction gaps reject.
pub fn process_control_frame(r: &mut Wram<'_>, rom: &[u8], limit: usize) -> Result<(), &'static str> {
    if r.byte(0x613) != 0 {
        return Err("active PLM coroutine requires host");
    }
    if r.word(0x1c23) & 0x8000 == 0 {
        return Ok(());
    }
    if r.word(0xdc8) != 0 {
        return Err("queued message requires coroutine host");
    }
    r.set_word(0x1c25, 0);
    for k in (0..80).step_by(2).rev() {
        r.set_word(0x1c27, k as u16);
        if r.word(HEADER + k) == 0 {
            continue;
        }
        if ![0x84e6, 0x853d, 0x86d0, 0x8aa6, 0x8ae0, 0xbba3].contains(&r.word(PRE + k)) {
            crate::plm::preinstruction(r, r.word(PRE + k), k as u16)?;
            if r.word(HEADER + k) == 0 {
                continue;
            }
        }
        let timer = r.word(INSTR_TIMER + k).wrapping_sub(1);
        r.set_word(INSTR_TIMER + k, timer);
        if timer != 0 {
            continue;
        }
        let mut pc = r.word(LIST + k);
        let mut ended = false;
        for _ in 0..limit {
            let opcode = word(rom, 0x84, pc)?;
            if opcode & 0x8000 == 0 {
                return Err("PLM frame rendering requires draw host");
            }
            match instruction(r, rom, opcode, next(pc, 2)?, k as u16)? {
                Some(p) => pc = p,
                None => {
                    ended = true;
                    break;
                }
            }
        }
        if !ended {
            return Err("PLM instruction budget exceeded");
        }
    }
    r.set_word(0x1c27, 0);
    Ok(())
}
pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], symbol: &str, a: i32, b: i32) -> Result<u32, &'static str> {
    if symbol == "PlmControlFrame" {
        process_control_frame(r, rom, 4096)?;
        return Ok(0);
    }
    if symbol == "ProcessPlmDrawInstruction" {
        process_draw_data(r, rom, a as u16)?;
        return Ok(0);
    }
    if symbol != "PlmInstruction" {
        return Err("unknown PLM seam");
    }
    let pc = b as u16;
    let opcode = word(rom, 0x84, pc)?;
    Ok(instruction(r, rom, opcode, next(pc, 2)?, a as u16)?.map_or(0, |p| 0x840000 | p as u32))
}
