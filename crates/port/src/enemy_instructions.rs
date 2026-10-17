//! Generic enemy bytecode handlers from pinned sm_a0.c:104–197.
//! Pointer results use banked ROM addresses; zero means terminate this frame.
use crate::Wram;

const ENEMY: usize = 0xf78;

fn record(k: i32) -> Result<usize, &'static str> {
    if !(0..0x800).contains(&k) || k & 0x3f != 0 {
        return Err("aligned enemy record below 0x800 required");
    }
    Ok(ENEMY + k as usize)
}

fn rom_index(rom: &[u8], pointer: u32, extent: usize) -> Result<usize, &'static str> {
    if pointer > 0xffffff || pointer & 0xffff < 0x8000 {
        return Err("banked LoROM pointer required");
    }
    if (pointer & 0xffff) as usize + extent > 0x10000 {
        return Err("instruction read crosses bank boundary");
    }
    let offset = ((pointer as usize >> 16) & 0x7f) * 0x8000 + (pointer as usize & 0x7fff);
    if offset.checked_add(extent).is_none_or(|end| end > rom.len()) {
        return Err("instruction read outside ROM");
    }
    Ok(offset)
}

pub fn dispatch_rom(ram: &mut Wram<'_>, rom: &[u8], symbol: &str, k: i32, pointer: i32) -> Result<u32, &'static str> {
    if symbol == "ProcessEnemyInstructions" {
        process_generic_instructions(ram, rom, 4096)?;
        return Ok(0);
    }
    let e = record(k)?;
    let pointer = u32::try_from(pointer).map_err(|_| "banked LoROM pointer required")?;
    if pointer >> 16 != ram.byte(e + 0x2e) as u32 {
        return Err("instruction pointer must match enemy bank");
    }
    let extent = match symbol {
        "EnemyInstr_SetAiPreInstr" | "EnemyInstr_SetTimer" | "EnemyInstr_Goto" | "EnemyInstr_DecTimerAndGoto" => 2,
        "EnemyInstr_GotoRel" | "EnemyInstr_DecTimerAndGotoRel" => 1,
        "EnemyInstr_Skip2bytes"
        | "EnemyInstr_Sleep"
        | "EnemyInstr_StopScript"
        | "EnemyInstr_EnableOffScreenProcessing"
        | "EnemyInstr_DisableOffScreenProcessing" => 0,
        "EnemyInstr_WaitNframes" => 2,
        "EnemyInstr_CopyToVram" => 7,
        "EnemyInstr_ClearAiPreInstr" => 0,
        _ => return Err("unsupported enemy instruction"),
    };
    let p = rom_index(rom, pointer, extent)?;
    let advance = |delta: i32| -> Result<u32, &'static str> {
        let addr = (pointer & 0xffff) as i32 + delta;
        if !(0x8000..=0xffff).contains(&addr) {
            return Err("instruction result crosses bank boundary");
        }
        Ok((pointer & 0xff0000) | addr as u32)
    };
    let word = || u16::from_le_bytes([rom[p], rom[p + 1]]);
    match symbol {
        "EnemyInstr_SetAiPreInstr" => {
            let next = advance(2)?;
            ram.set_word(e + 0x3a, word());
            Ok(next)
        }
        "EnemyInstr_ClearAiPreInstr" => {
            ram.set_word(e + 0x3a, 0x807b);
            Ok(pointer)
        }
        "EnemyInstr_SetTimer" => {
            let result = advance(2)?;
            ram.set_word(e + 0x18, word());
            Ok(result)
        }
        "EnemyInstr_Skip2bytes" => advance(2),
        "EnemyInstr_Goto" | "EnemyInstr_DecTimerAndGoto" => {
            let old = ram.word(e + 0x18);
            let result = if symbol == "EnemyInstr_DecTimerAndGoto" && old == 1 {
                advance(2)?
            } else {
                if word() < 0x8000 {
                    return Err("goto target below LoROM bank");
                }
                (pointer & 0xff0000) | word() as u32
            };
            if symbol == "EnemyInstr_DecTimerAndGoto" {
                ram.set_word(e + 0x18, old.wrapping_sub(1));
            }
            Ok(result)
        }
        "EnemyInstr_GotoRel" | "EnemyInstr_DecTimerAndGotoRel" => {
            let old = ram.byte(e + 0x18);
            let delta = if symbol == "EnemyInstr_DecTimerAndGotoRel" && old == 1 { 1 } else { rom[p] as i8 as i32 };
            let result = advance(delta)?;
            if symbol == "EnemyInstr_DecTimerAndGotoRel" {
                ram.set_byte(e + 0x18, old.wrapping_sub(1));
            }
            Ok(result)
        }
        "EnemyInstr_StopScript" => {
            ram.set_word(e + 0x0e, ram.word(e + 0x0e) | 0x200);
            Ok(0)
        }
        "EnemyInstr_Sleep" => {
            let previous = advance(-2)?;
            ram.set_word(e + 0x1a, previous as u16);
            Ok(0)
        }
        "EnemyInstr_WaitNframes" => {
            let next = advance(2)?;
            ram.set_word(e + 0x1c, word());
            ram.set_word(e + 0x1a, next as u16);
            Ok(0)
        }
        "EnemyInstr_CopyToVram" => {
            let next = advance(7)?;
            let tail = ram.word(0x330) as usize;
            if tail > 0x260 - 7 {
                return Err("packed VRAM write queue allocation exceeded");
            }
            for byte in 0..7 {
                ram.set_byte(0xd0 + tail + byte, rom[p + byte]);
            }
            ram.set_word(0x330, (tail as u16).wrapping_add(7));
            Ok(next)
        }
        "EnemyInstr_EnableOffScreenProcessing" => {
            ram.set_word(e + 0x0e, ram.word(e + 0x0e) | 0x800);
            Ok(pointer)
        }
        "EnemyInstr_DisableOffScreenProcessing" => {
            ram.set_word(e + 0x0e, ram.word(e + 0x0e) & !0x800);
            Ok(pointer)
        }
        _ => unreachable!(),
    }
}

// Only addresses mapped to generic handlers by actual CallEnemyInstr are admitted.
fn generic_symbol(bank: u8, opcode: u16) -> Option<&'static str> {
    match (bank, opcode) {
        (0xa5 | 0xaa, 0x807c) => Some("EnemyInstr_StopScript"),
        (0xa2 | 0xa3 | 0xa4 | 0xa5 | 0xa6 | 0xa7 | 0xa8 | 0xaa | 0xb2 | 0xb3, 0x80ed) => Some("EnemyInstr_Goto"),
        (0xa2 | 0xa5 | 0xa7 | 0xa8 | 0xaa | 0xb2 | 0xb3, 0x8110) => Some("EnemyInstr_DecTimerAndGoto"),
        (0xa2 | 0xa5 | 0xa7 | 0xa8 | 0xaa | 0xb2 | 0xb3, 0x8123) => Some("EnemyInstr_SetTimer"),
        (0xa2 | 0xa3 | 0xa4 | 0xa5 | 0xa6 | 0xa7 | 0xa8 | 0xa9 | 0xaa | 0xb2 | 0xb3, 0x812f) => {
            Some("EnemyInstr_Sleep")
        }
        (0xa5 | 0xaa | 0xb2, 0x813a) => Some("EnemyInstr_WaitNframes"),
        (0xaa, 0x814b) => Some("EnemyInstr_CopyToVram"),
        (0xa2 | 0xa3 | 0xa8, 0x8173) => Some("EnemyInstr_EnableOffScreenProcessing"),
        (0xa2 | 0xa3 | 0xa8, 0x817d) => Some("EnemyInstr_DisableOffScreenProcessing"),
        _ => None,
    }
}

/// ProcessEnemyInstructions control flow for generic instruction scripts only.
/// Unsupported dispatch and execution-limit errors may follow prior valid writes;
/// callers must retain failure evidence rather than retry as a live C fallback.
pub fn process_generic_instructions(ram: &mut Wram<'_>, rom: &[u8], limit: usize) -> Result<(), &'static str> {
    let k = ram.word(0xe54) as i32;
    let e = record(k)?;
    if ram.word(e + 0x12) & 4 != 0 {
        return Ok(());
    }
    let timer = ram.word(e + 0x1c);
    ram.set_word(e + 0x1c, timer.wrapping_sub(1));
    if timer != 1 {
        ram.set_word(e + 0x10, ram.word(e + 0x10) & !0x8000);
        return Ok(());
    }
    let bank = ram.byte(e + 0x2e);
    let mut pc = (bank as u32) << 16 | ram.word(e + 0x1a) as u32;
    let mut steps = 0;
    loop {
        let p = rom_index(rom, pc, 2)?;
        let opcode = u16::from_le_bytes([rom[p], rom[p + 1]]);
        if opcode & 0x8000 == 0 {
            let p = rom_index(rom, pc, 4)?;
            if pc & 0xffff > 0xfffb {
                return Err("animation frame crosses bank boundary");
            }
            ram.set_word(e + 0x1c, opcode);
            ram.set_word(e + 0x16, u16::from_le_bytes([rom[p + 2], rom[p + 3]]));
            ram.set_word(e + 0x1a, pc.wrapping_add(4) as u16);
            ram.set_word(e + 0x10, ram.word(e + 0x10) | 0x8000);
            return Ok(());
        }
        let symbol = generic_symbol(bank, opcode).ok_or("unsupported bank-specific enemy instruction")?;
        if steps >= limit {
            return Err("enemy instruction execution limit reached");
        }
        if pc & 0xffff > 0xfffd {
            return Err("instruction argument crosses bank boundary");
        }
        pc = dispatch_rom(ram, rom, symbol, k, (pc + 2) as i32)?;
        if pc == 0 {
            return Ok(());
        }
        steps += 1;
    }
}
