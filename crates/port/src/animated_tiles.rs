//! Bank-87 animated tile allocation and control.
use crate::Wram;

fn palette_word(rom: &[u8], address: u16) -> Result<u16, &'static str> {
    if address < 0x8000 || address == 0xffff {
        return Err("palette instruction ROM extent");
    }
    let offset = 0x68000 + usize::from(address & 0x7fff);
    let bytes = rom.get(offset..offset + 2).ok_or("palette instruction ROM extent")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

pub fn palette_instruction(
    r: &mut Wram<'_>,
    rom: &[u8],
    opcode: u16,
    k: u16,
    pc: u16,
) -> Result<(u16, u16), &'static str> {
    let slot = usize::from(k);
    let needs_slot = matches!(opcode, 0xc5cf | 0xc5d4 | 0xc5dd | 0xc639 | 0xc648 | 0xc655);
    if needs_slot && (k > 14 || k & 1 != 0) {
        return Err("invalid palette instruction slot");
    }
    let next = match opcode {
        0xc595 => {
            let index = r.word(0x1e7b);
            if index > 14 || index & 1 != 0 {
                return Err("invalid palette effect index");
            }
            r.set_word(0x1ebd + usize::from(index), pc.wrapping_add(4));
            return Ok((0, 0));
        }
        0xc599 | 0xc5a2 | 0xc5ab | 0xc5b4 | 0xc5bd | 0xc5c6 => {
            let colors = match opcode {
                0xc599 => 2,
                0xc5a2 => 3,
                0xc5ab => 4,
                0xc5b4 => 8,
                0xc5bd => 9,
                _ => 15,
            };
            return Ok((k.wrapping_add(colors * 2), pc.wrapping_add(2)));
        }
        0xc5cf => {
            r.set_word(0x1e7d + slot, 0);
            0
        }
        0xc5d4 => {
            r.set_word(0x1ead + slot, palette_word(rom, pc)?);
            pc.wrapping_add(2)
        }
        0xc5dd => {
            r.set_word(0x1ead + slot, 0xc5e3);
            pc
        }
        0xc61e => palette_word(rom, pc)?,
        0xc639 => {
            let timer = r.word(0x1edd + slot);
            r.set_word(0x1edd + slot, timer.wrapping_sub(1));
            if timer == 1 { pc.wrapping_add(2) } else { palette_word(rom, pc)? }
        }
        0xc648 | 0xf1c6 | 0xc65e => {
            if pc < 0x8000 {
                return Err("palette instruction ROM extent");
            }
            let value = *rom.get(0x68000 + usize::from(pc & 0x7fff)).ok_or("palette instruction ROM extent")?;
            match opcode {
                0xc648 => r.set_byte(0x1edd + slot, value),
                0xf1c6 => r.set_word(0x1eed, u16::from(value)),
                _ => crate::audio::queue_music(r, u16::from(value), 8, true)?,
            }
            pc.wrapping_add(1)
        }
        0xc655 => {
            r.set_word(0x1e8d + slot, palette_word(rom, pc)?);
            pc.wrapping_add(2)
        }
        0xc66a | 0xc673 | 0xc67c => {
            let channel = match opcode {
                0xc66a => 0,
                0xc673 => 1,
                _ => 2,
            };
            crate::audio::queue_sfx(r, channel, (palette_word(rom, pc)? << 8) | 6)?;
            pc.wrapping_add(1)
        }
        _ => return Err("unsupported palette instruction"),
    };
    Ok((k, next))
}

fn byte(rom: &[u8], address: u16) -> Result<u8, &'static str> {
    if address < 0x8000 {
        return Err("animated tile ROM extent");
    }
    rom.get(0x38000 + usize::from(address & 0x7fff)).copied().ok_or("missing animated tile ROM")
}

pub fn spawn_palette_effect(r: &mut Wram<'_>, rom: &[u8], header: u16) -> Result<(), &'static str> {
    let Some(index) = (0..16).step_by(2).rev().find(|&index| r.word(0x1e7d + index) == 0) else {
        return Ok(());
    };
    let offset = 0x68000 + usize::from(header & 0x7fff);
    if header < 0x8000 {
        return Err("palette effect header extent");
    }
    let bytes = rom.get(offset..offset + 4).ok_or("palette effect ROM extent")?;
    let initializer = u16::from_le_bytes([bytes[0], bytes[1]]);
    let mut list = u16::from_le_bytes([bytes[2], bytes[3]]);
    let mut id = header;
    let mut preinstruction = 0xc526;
    match initializer {
        0xc685 => {}
        0xe204 => preinstruction = 0xe20b,
        0xe440 => {
            let items = r.word(0x9a2);
            list = if items & 0x20 != 0 {
                0xe8b6
            } else if items & 1 != 0 {
                0xe68a
            } else {
                0xe45e
            };
        }
        0xf730 => {
            let area = usize::from(r.word(0x79f));
            if area >= 8 {
                return Err("palette effect boss area extent");
            }
            if r.word(0xd828 + area) & 2 != 0 {
                id = 0;
            }
        }
        _ => return Err("unsupported palette effect initializer"),
    }
    for (base, value) in [(0x1e7d, id), (0x1e8d, 0), (0x1ead, preinstruction), (0x1ebd, list), (0x1ecd, 1), (0x1edd, 0)]
    {
        r.set_word(base + index, value);
    }
    Ok(())
}

fn word(rom: &[u8], address: u16) -> Result<u16, &'static str> {
    if address < 0x8000 || address == 0xffff {
        return Err("animated tile ROM extent");
    }
    let offset = 0x38000 + usize::from(address & 0x7fff);
    let bytes = rom.get(offset..offset + 2).ok_or("missing animated tile ROM")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

pub fn spawn(r: &mut Wram<'_>, rom: &[u8], header: u16) -> Result<(), &'static str> {
    let Some(index) = (0..12).step_by(2).rev().find(|&index| r.word(0x1ef5 + index) == 0) else {
        return Ok(());
    };
    let list = word(rom, header)?;
    let size = word(rom, header.checked_add(2).ok_or("animated tile header extent")?)?;
    let vram = word(rom, header.checked_add(4).ok_or("animated tile header extent")?)?;
    for (base, value) in
        [(0x1ef5, header), (0x1f01, 0), (0x1f0d, list), (0x1f25, 0), (0x1f31, size), (0x1f3d, vram), (0x1f19, 1)]
    {
        r.set_word(base + index, value);
    }
    Ok(())
}

pub fn process(r: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    let index = r.word(0x1ef3);
    if index > 10 || index & 1 != 0 {
        return Err("invalid animated tile slot");
    }
    let slot = usize::from(index);
    let timer = r.word(0x1f19 + slot);
    r.set_word(0x1f19 + slot, timer.wrapping_sub(1));
    if timer != 1 {
        return Ok(());
    }
    let mut pc = r.word(0x1f0d + slot);
    for _ in 0..65536 {
        let opcode = word(rom, pc)?;
        if opcode & 0x8000 == 0 {
            let source = word(rom, pc.wrapping_add(2))?;
            r.set_word(0x1f19 + slot, opcode);
            r.set_word(0x1f25 + slot, source);
            r.set_word(0x1f0d + slot, pc.wrapping_add(4));
            return Ok(());
        }
        r.set_word(0x1f49, opcode);
        let name = match opcode {
            0x80b2 => "AnimtilesInstr_Delete",
            0x80b7 => "AnimtilesInstr_Goto",
            0x80bc => "AnimtilesInstr_GotoRel",
            0x80d4 => "AnimtilesInstr_DecrementTimerAndGoto",
            0x80dc => "AnimtilesInstr_DecrementTimerAndGotoRel",
            0x80e3 => "AnimtilesInstr_SetTimer",
            0x80f0 => "AnimtilesInstr_QueueMusic",
            0x80fc => "AnimtilesInstr_QueueSfx1",
            0x8108 => "AnimtilesInstr_QueueSfx2",
            0x8114 => "AnimtilesInstr_QueueSfx3",
            0x8120 => "AnimtilesInstr_GotoIfBossBitSet",
            0x8133 => "AnimtilesInstr_SetBossBit",
            0x813f => "AnimtilesInstr_GotoIfEventHappened",
            0x8150 => "AnimtilesInstr_SetEventHappened",
            0x81ba => "AnimtilesInstr_WaitUntilAreaBossDead_DoubleRet",
            0x8303 => "AnimtilesInstr_GotoIfBossBitSetInArea",
            0x8320 => "AnimtilesInstr_SpawnTourianStatueEyeGlow",
            0x832f => "AnimtilesInstr_SpawnTourianStatueSoul",
            0x833e => "AnimtilesInstr_GotoIfTourianStatueBusy",
            0x8349 => "AnimtilesInstr_TourianStatueSetState",
            0x8352 => "AnimtilesInstr_TourianStatueClearState",
            0x835b => "AnimtilesInstr_Clear3PaletteColors",
            0x8372 => "AnimtilesInstr_SpawnPalfxObj",
            0x837f => "AnimtilesInstr_Write8PaletteColors",
            _ => return Err("unsupported animated tile opcode"),
        };
        pc = dispatch_rom(r, rom, name, i32::from(index), i32::from(pc.wrapping_add(2)))? as u16;
        if pc == 0 {
            return Ok(());
        }
    }
    Err("animated tile instruction cycle")
}

pub fn handle(r: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    if r.word(0x1ef1) & 0x8000 == 0 {
        return Ok(());
    }
    let mut index = 10u16;
    loop {
        r.set_word(0x1ef3, index);
        if r.word(0x1ef5 + usize::from(index)) != 0 {
            process(r, rom)?;
        }
        index = r.word(0x1ef3).wrapping_sub(2);
        if index & 0x8000 != 0 {
            return Ok(());
        }
        if index > 10 || index & 1 != 0 {
            return Err("invalid animated tile scheduler slot");
        }
    }
}

/// Run one palette effect, including its separate control and color streams.
pub fn process_palette(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<(), &'static str> {
    if k > 14 || k & 1 != 0 {
        return Err("invalid palette effect slot");
    }
    let name = match r.word(0x1ead + usize::from(k)) {
        0xc526 => "PalPreInstr_nullsub_129",
        0xe20b => "PalPreInstr_E1BC",
        0xe2e0 => "PalPreInstr_CheckEnemy0Health",
        0xe379 => "PalPreInstr_SamusInHeat",
        0xec59 => "PalPreInstr_SwitchIfYpos",
        0xed84 => "PalPreInstr_SwitchIfYpos2",
        0xeec5 => "PalPreInstr_DeletePalfxIfMinibossDead",
        0xf621 => "PalPreInstr_F621",
        _ => return Err("unsupported palette effect preinstruction"),
    };
    dispatch_rom(r, rom, name, i32::from(k), 0)?;
    let mut slot = r.word(0x1e7b);
    if slot > 14 || slot & 1 != 0 {
        return Err("invalid palette effect slot");
    }
    let timer_address = 0x1ecd + usize::from(slot);
    let timer = r.word(timer_address);
    r.set_word(timer_address, timer.wrapping_sub(1));
    if timer != 1 {
        return Ok(());
    }
    let mut pc = r.word(0x1ebd + usize::from(slot));
    let mut steps = 0;
    loop {
        steps += 1;
        if steps > 65536 {
            return Err("palette instruction cycle");
        }
        let instruction = palette_word(rom, pc)?;
        if instruction & 0x8000 == 0 {
            if slot > 14 || slot & 1 != 0 {
                return Err("invalid palette effect slot");
            }
            r.set_word(0x1ecd + usize::from(slot), instruction);
            break;
        }
        (slot, pc) = palette_instruction(r, rom, instruction, slot, pc.wrapping_add(2))?;
        if pc == 0 {
            return Ok(());
        }
    }
    let mut color = r.word(0x1e8d + usize::from(slot));
    loop {
        steps += 1;
        if steps > 65536 {
            return Err("palette color stream cycle");
        }
        let value = palette_word(rom, pc.wrapping_add(2))?;
        if value & 0x8000 == 0 {
            // Native indexing discards the low bit of the byte cursor.
            r.set_word(0xc000 + usize::from(color & !1), value);
            color = color.wrapping_add(2);
            pc = pc.wrapping_add(2);
        } else {
            (color, pc) = palette_instruction(r, rom, value, color, pc)?;
            if pc == 0 {
                return Ok(());
            }
        }
    }
}

pub fn handle_palette(r: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    if r.word(0x1e79) & 0x8000 == 0 {
        return Ok(());
    }
    let mut index = 14u16;
    loop {
        r.set_word(0x1e7b, index);
        if r.word(0x1e7d + usize::from(index)) != 0 {
            process_palette(r, rom, index)?;
            index = r.word(0x1e7b);
            if index > 14 || index & 1 != 0 {
                return Err("invalid palette effect slot");
            }
        }
        if index == 0 {
            return Ok(());
        }
        index -= 2;
    }
}

pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], name: &str, a: i32, b: i32) -> Result<u32, &'static str> {
    if name.starts_with("PalInstr_") {
        let opcode = match name {
            "PalInstr_Wait" => 0xc595,
            "PalInstr_ColorPlus2" => 0xc599,
            "PalInstr_ColorPlus3" => 0xc5a2,
            "PalInstr_ColorPlus4" => 0xc5ab,
            "PalInstr_ColorPlus8" => 0xc5b4,
            "PalInstr_ColorPlus9" => 0xc5bd,
            "PalInstr_ColorPlus15" => 0xc5c6,
            "PalInstr_Delete" => 0xc5cf,
            "PalInstr_SetPreInstr" => 0xc5d4,
            "PalInstr_ClearPreInstr" => 0xc5dd,
            "PalInstr_Goto" => 0xc61e,
            "PalInstr_DecTimerGoto" => 0xc639,
            "PalInstr_SetTimer" => 0xc648,
            "PalInstr_SetColorIndex" => 0xc655,
            "PalInstr_QueueMusic" => 0xc65e,
            "PalInstr_QueueSfx1" => 0xc66a,
            "PalInstr_QueueSfx2" => 0xc673,
            "PalInstr_QueueSfx3" => 0xc67c,
            "PalInstr_SetPalfxIndex" => 0xf1c6,
            _ => return Err("unknown palette instruction"),
        };
        let (k, j) = palette_instruction(r, rom, opcode, a as u16, b as u16)?;
        return Ok(u32::from(k) | (u32::from(j) << 16));
    }
    if name.starts_with("PalPreInstr_") {
        let slot = a as u16;
        if slot > 14 || slot & 1 != 0 {
            return Err("invalid palette effect slot");
        }
        let slot = usize::from(slot);
        match name {
            "PalPreInstr_SamusInHeat" => {
                let items = r.word(0x9a2);
                if items & 0x21 == 0 {
                    let damage = (u32::from(r.word(0xa50)) << 16) | u32::from(r.word(0xa4e));
                    let damage = damage.wrapping_add(0x4000);
                    r.set_word(0xa50, (damage >> 16) as u16);
                    r.set_word(0xa4e, damage as u16);
                    if r.word(0x5b6) & 7 == 0 && r.word(0x9c2) > 0x46 {
                        crate::audio::queue_sfx(r, 2, 0x2d06)?;
                    }
                }
                let index = r.word(0x1eed);
                if index != r.word(0x1eef) {
                    let table: u16 = if items & 0x20 != 0 {
                        0xe3e0
                    } else if items & 1 != 0 {
                        0xe400
                    } else {
                        0xe420
                    };
                    let address = table
                        .checked_add(index.checked_mul(2).ok_or("heat palette index extent")?)
                        .ok_or("heat palette index extent")?;
                    let offset = 0x68000 + usize::from(address & 0x7fff);
                    let bytes = rom.get(offset..offset + 2).ok_or("heat palette ROM extent")?;
                    let list = u16::from_le_bytes([bytes[0], bytes[1]]);
                    r.set_word(0x1eef, index);
                    r.set_word(0x1ecd + slot, 1);
                    r.set_word(0x1ebd + slot, list);
                }
            }
            "PalPreInstr_nullsub_129" => {}
            "PalPreInstr_E1BC" => {
                if r.word(0x1f51) == 0xb35f {
                    r.set_word(0x1ebd + slot, 0xe192);
                    r.set_word(0x1ecd + slot, 1);
                }
            }
            "PalPreInstr_CheckEnemy0Health" => {
                if r.word(0xf8c) == 0 {
                    r.set_word(0x1e7d + slot, 0);
                }
            }
            "PalPreInstr_SwitchIfYpos" | "PalPreInstr_SwitchIfYpos2" => {
                if r.word(0xafa) < 0x380 {
                    r.set_word(0x1ecd + slot, 1);
                    r.set_word(0x1ebd + slot, if name.ends_with('2') { 0xec76 } else { 0xeb43 });
                }
            }
            "PalPreInstr_DeletePalfxIfMinibossDead" => {
                let area = usize::from(r.word(0x79f));
                if area >= 8 {
                    return Err("palette effect boss area extent");
                }
                if r.word(0xd828 + area) & 2 != 0 {
                    r.set_word(0x1e7d + slot, 0);
                }
            }
            "PalPreInstr_F621" => {
                if r.word(0x1e79 + slot) != 0 {
                    r.set_word(0x1e7d + slot, 0);
                }
            }
            _ => return Err("unsupported palette effect preinstruction"),
        }
        return Ok(0);
    }
    if name.starts_with("AnimtilesInstr_") {
        let slot = a as u16;
        if slot > 10 || slot & 1 != 0 {
            return Err("invalid animated tile slot");
        }
        let slot = usize::from(slot);
        let pc = b as u16;
        let result = match name {
            "AnimtilesInstr_SpawnTourianStatueEyeGlow" | "AnimtilesInstr_SpawnTourianStatueSoul" => {
                let definition = if name.ends_with("EyeGlow") { 0xba6a } else { 0xba94 };
                crate::enemy_projectiles::spawn_room(r, rom, definition, word(rom, pc)?)?;
                pc.wrapping_add(2)
            }
            "AnimtilesInstr_SpawnPalfxObj" => {
                spawn_palette_effect(r, rom, word(rom, pc)?)?;
                pc.wrapping_add(2)
            }
            "AnimtilesInstr_Delete" => {
                r.set_word(0x1ef5 + slot, 0);
                0
            }
            "AnimtilesInstr_Goto" => word(rom, pc)?,
            "AnimtilesInstr_GotoRel" => {
                let displacement = byte(rom, pc)? as i8;
                r.set_word(0x1f49, pc);
                pc.wrapping_add(displacement as u16)
            }
            "AnimtilesInstr_DecrementTimerAndGoto" | "AnimtilesInstr_DecrementTimerAndGotoRel" => {
                let timer = r.word(0x1f01 + slot);
                r.set_word(0x1f01 + slot, timer.wrapping_sub(1));
                let relative = name.ends_with("GotoRel");
                if timer == 1 {
                    pc.wrapping_add(if relative { 1 } else { 2 })
                } else if relative {
                    let displacement = byte(rom, pc)? as i8;
                    r.set_word(0x1f49, pc);
                    pc.wrapping_add(displacement as u16)
                } else {
                    word(rom, pc)?
                }
            }
            "AnimtilesInstr_SetTimer" => {
                r.set_byte(0x1f01 + slot, byte(rom, pc)?);
                pc.wrapping_add(1)
            }
            "AnimtilesInstr_QueueMusic" => {
                crate::audio::queue_music(r, u16::from(byte(rom, pc)?), 8, true)?;
                pc.wrapping_add(1)
            }
            "AnimtilesInstr_GotoIfTourianStatueBusy" => {
                if r.word(0x1e6f) & 0x8000 != 0 {
                    word(rom, pc)?
                } else {
                    pc.wrapping_add(2)
                }
            }
            "AnimtilesInstr_TourianStatueSetState" | "AnimtilesInstr_TourianStatueClearState" => {
                let bits = word(rom, pc)?;
                let state = r.word(0x1e6f);
                r.set_word(0x1e6f, if name.ends_with("SetState") { state | bits } else { state & !bits });
                pc.wrapping_add(2)
            }
            "AnimtilesInstr_Clear3PaletteColors" => {
                let offset = usize::from(word(rom, pc)? & !1);
                if offset > 0x1fa {
                    return Err("animated tile palette extent");
                }
                for index in (0..6).step_by(2) {
                    r.set_word(0xc000 + offset + index, 0);
                }
                pc.wrapping_add(2)
            }
            "AnimtilesInstr_Write8PaletteColors" => {
                let offset = usize::from(word(rom, pc)? & !1);
                if offset > 0x1f0 {
                    return Err("animated tile target palette extent");
                }
                for (index, color) in
                    [0x3800, 0x7f58, 0x6ed5, 0x5a71, 0x49ee, 0x356a, 0x24e7, 0x1083].into_iter().enumerate()
                {
                    r.set_word(0xc200 + offset + index * 2, color);
                }
                pc.wrapping_add(2)
            }
            "AnimtilesInstr_GotoIfBossBitSet" => {
                let bit = u16::from(byte(rom, pc)?);
                if crate::system::dispatch(r, "CheckBossBitForCurArea", bit, 0)? & 1 != 0 {
                    word(rom, pc.wrapping_add(1))?
                } else {
                    pc.wrapping_add(3)
                }
            }
            "AnimtilesInstr_SetBossBit" => {
                crate::system::dispatch(r, "SetBossBitForCurArea", u16::from(byte(rom, pc)?), 0)?;
                pc.wrapping_add(1)
            }
            "AnimtilesInstr_GotoIfEventHappened" => {
                if crate::system::dispatch(r, "CheckEventHappened", word(rom, pc)?, 0)? & 1 != 0 {
                    word(rom, pc.wrapping_add(2))?
                } else {
                    pc.wrapping_add(4)
                }
            }
            "AnimtilesInstr_SetEventHappened" => {
                crate::system::dispatch(r, "SetEventHappened", word(rom, pc)?, 0)?;
                pc.wrapping_add(2)
            }
            "AnimtilesInstr_WaitUntilAreaBossDead_DoubleRet" => {
                if crate::system::dispatch(r, "CheckBossBitForCurArea", 1, 0)? & 1 == 0 {
                    r.set_word(0x1f19 + slot, 1);
                    0
                } else {
                    pc
                }
            }
            "AnimtilesInstr_GotoIfBossBitSetInArea" => {
                let operand = word(rom, pc)?;
                let area = usize::from(operand >> 8);
                if area >= 8 {
                    return Err("animated tile boss area extent");
                }
                if r.byte(0xd828 + area) & operand as u8 != 0 {
                    word(rom, pc.wrapping_add(2))?
                } else {
                    pc.wrapping_add(4)
                }
            }
            "AnimtilesInstr_QueueSfx1" | "AnimtilesInstr_QueueSfx2" | "AnimtilesInstr_QueueSfx3" => {
                let channel = match name {
                    "AnimtilesInstr_QueueSfx1" => 0,
                    "AnimtilesInstr_QueueSfx2" => 1,
                    _ => 2,
                };
                crate::audio::queue_sfx(r, channel, ((u16::from(byte(rom, pc)?)) << 8) | 6)?;
                pc.wrapping_add(1)
            }
            _ => return Err("unsupported animated tile instruction"),
        };
        return Ok(u32::from(result));
    }
    match name {
        "EnableAnimtiles" => r.set_word(0x1ef1, r.word(0x1ef1) | 0x8000),
        "DisableAnimtiles" => r.set_word(0x1ef1, r.word(0x1ef1) & 0x7fff),
        "ClearAnimtiles" => {
            for index in (0..12).step_by(2) {
                r.set_word(0x1ef5 + index, 0);
            }
        }
        "ProcessAnimtilesObject" => process(r, rom)?,
        "AnimtilesHandler" => handle(r, rom)?,
        "SpawnAnimtiles" => spawn(r, rom, a as u16)?,
        "SpawnPalfxObject" => spawn_palette_effect(r, rom, a as u16)?,
        "PalFx_ProcessOne" => process_palette(r, rom, a as u16)?,
        "PaletteFxHandler" => handle_palette(r, rom)?,
        "EnablePaletteFx" => r.set_word(0x1e79, r.word(0x1e79) | 0x8000),
        "DisablePaletteFx" => r.set_word(0x1e79, r.word(0x1e79) & 0x7fff),
        "ClearPaletteFXObjects" => {
            for slot in (0..16).step_by(2) {
                r.set_word(0x1e7d + slot, 0);
            }
        }
        _ => return Err("unknown animated tile routine"),
    }
    Ok(0)
}
