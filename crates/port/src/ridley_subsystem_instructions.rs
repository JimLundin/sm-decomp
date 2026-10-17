//! Native boss instruction closure; all callbacks intentionally use enemy zero.
use super::*;
pub use crate::ridley_enemy_instructions::{OpcodeExecution, PointerKind};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScriptReport {
    pub frozen: bool,
    pub timer_only: bool,
    pub opcodes: Vec<OpcodeExecution>,
}

fn opcode(symbol: &str) -> Option<u16> {
    Some(match symbol {
        "Ridley_Instr_1" => 0xe517,
        "Ridley_Instr_2" => 0xe501,
        "Ridley_Instr_3" => 0xe4f8,
        "Ridley_Instr_4" => 0xe4ee,
        "Ridley_Instr_5" => 0xe4be,
        "Ridley_Instr_6" => 0xe4ca,
        "Ridley_Instr_7" => 0xe727,
        "Ridley_Instr_8" => 0xe72f,
        "Ridley_Instr_9" => 0xe71c,
        "Ridley_Instr_10" => 0xe4d2,
        "Ridley_Instr_11" => 0xe84d,
        "Ridley_Instr_12" => 0xe904,
        "Ridley_Instr_13" => 0xe909,
        "Ridley_Instr_14" => 0xe51f,
        "Ridley_Instr_15" => 0xe969,
        "Ridley_Instr_16" => 0xe976,
        "EnemyInstr_Goto" => 0x80ed,
        "EnemyInstr_Sleep" => 0x812f,
        _ => return None,
    })
}

fn arguments(code: u16) -> Option<u16> {
    Some(match code {
        0xe517 | 0xe501 | 0xe4f8 | 0xe4d2 | 0x80ed => 1,
        0xe4ee | 0xe51f => 2,
        0xe4be | 0xe4ca | 0xe727 | 0xe72f | 0xe71c | 0xe84d | 0xe904 | 0xe909 | 0xe969 | 0xe976 | 0x812f => 0,
        _ => return None,
    })
}

const SCRIPTS: &[(u16, &[u16])] = &[
    (
        0xe538,
        &[
            0xe517, 0xe542, 0x000c, 0xe983, 0x812f, 0x000c, 0xe9a5, 0x812f, 0xe517, 0xe576, 0xe501, 0x0000, 0x0004,
            0xe983, 0xe501, 0x0002, 0x0006, 0xea4f, 0xe501, 0x0004, 0x0050, 0xea71, 0xe501, 0x0002, 0x0006, 0xea4f,
            0xe501, 0x0000, 0x0004, 0xe983, 0x812f, 0xe501, 0x0000, 0x0004, 0xe9a5, 0xe501, 0x0002, 0x0006, 0xea93,
            0xe501, 0x0004, 0x0050, 0xeab5, 0xe501, 0x0002, 0x0006, 0xea93, 0xe501, 0x0000, 0x0004, 0xe9a5, 0x812f,
            0xe517, 0xe5fe, 0xe501, 0x0000, 0x0002, 0xe983, 0xe4f8, 0xe5b2, 0x812f, 0xe501, 0x0002, 0x0003, 0xea4f,
            0xe4f8, 0xe5c8, 0x0001, 0xea4f, 0xe4ee, 0xe5be, 0xe5f4, 0xe501, 0x0004, 0x0004, 0xea71, 0xe4f8, 0xe5de,
            0x0001, 0xea71, 0xe4ee, 0xe5d4, 0xe5de, 0xe501, 0x0002, 0x0003, 0xea4f, 0xe4f8, 0xe5f4, 0x0001, 0xea4f,
            0xe4ee, 0xe5ea, 0xe5f4, 0xe501, 0x0000, 0x0002, 0xe983, 0x812f, 0xe501, 0x0000, 0x0002, 0xe9a5, 0xe4f8,
            0xe60c, 0x812f, 0xe501, 0x0002, 0x0003, 0xea93, 0xe4f8, 0xe622, 0x0001, 0xea93, 0xe4ee, 0xe618, 0xe64e,
            0xe501, 0x0004, 0x0004, 0xeab5, 0xe4f8, 0xe638, 0x0001, 0xea71, 0xe4ee, 0xe62e, 0xe638, 0xe501, 0x0002,
            0x0003, 0xea93, 0xe4f8, 0xe64e, 0x0001, 0xea4f, 0xe4ee, 0xe644, 0xe64e, 0xe501, 0x0000, 0x0002, 0xe9a5,
            0x812f, 0xe517, 0xe676, 0xe501, 0x0000, 0x0004, 0xe983, 0xe501, 0x0002, 0x0006, 0xea4f, 0xe501, 0x0004,
            0x0001, 0xea71, 0x812f, 0xe501, 0x0000, 0x0004, 0xe9a5, 0xe501, 0x0002, 0x0006, 0xea93, 0xe501, 0x0004,
            0x0001, 0xeab5, 0x812f, 0xe517, 0xe6ae, 0x0006, 0xe983, 0xe4be, 0x0008, 0xe9c7, 0x0060, 0xe9e9, 0x0008,
            0xe9c7, 0xe4ca, 0x0001, 0xe983, 0x812f, 0x0006, 0xe9a5, 0xe4be, 0x0008, 0xea0b, 0x0060, 0xea2d, 0x0008,
            0xea0b, 0xe4ca, 0x0001, 0xe9a5, 0x812f, 0xe517, 0xe6de, 0x0006, 0xe983, 0xe4be, 0x0008, 0xe9c7, 0x0010,
            0xe9e9, 0xe4ca, 0x812f, 0x0006, 0xe9a5, 0xe4be, 0x0008, 0xea0b, 0x0010, 0xea2d, 0xe4ca, 0x812f, 0xe727,
            0x0001, 0xe983, 0x0008, 0xead7, 0xe72f, 0x0001, 0xead7, 0x0001, 0xe9a5, 0x812f, 0xe727, 0x0001, 0xe9a5,
            0x0008, 0xead7, 0xe71c, 0x0001, 0xead7, 0x0001, 0xe983, 0x812f,
        ],
    ),
    (
        0xe73a,
        &[
            0xe517, 0xe7b4, 0xe4d2, 0xe7ac, 0x0008, 0xe983, 0xe4be, 0x0008, 0xe9c7, 0x0002, 0xe9e9, 0xe84d, 0xe904,
            0x0005, 0xe9e9, 0xe84d, 0xe909, 0x0005, 0xe9e9, 0xe84d, 0xe909, 0x0005, 0xe9e9, 0xe84d, 0xe909, 0x0030,
            0xe9e9, 0x0008, 0xe9c7, 0xe4d2, 0xe7ac, 0x0020, 0xe983, 0xe84d, 0xe4be, 0x0008, 0xe9c7, 0x0002, 0xe9e9,
            0xe84d, 0xe904, 0x0005, 0xe9e9, 0xe84d, 0xe909, 0x0005, 0xe9e9, 0xe84d, 0xe909, 0x0005, 0xe9e9, 0xe84d,
            0xe909, 0x0030, 0xe9e9, 0x0008, 0xe9c7, 0xe4ca, 0x0001, 0xe983, 0x812f, 0xe4d2, 0xe820, 0x0008, 0xe9a5,
            0xe4be, 0x0008, 0xea0b, 0x0002, 0xea2d, 0xe84d, 0xe904, 0x0005, 0xea2d, 0xe84d, 0xe909, 0x0005, 0xea2d,
            0xe84d, 0xe909, 0x0005, 0xea2d, 0xe84d, 0xe909, 0x0030, 0xea2d, 0x0008, 0xea0b, 0xe4d2, 0xe820, 0x0020,
            0xe9a5, 0xe4be, 0x0008, 0xea0b, 0x0002, 0xea2d, 0xe84d, 0xe904, 0x0005, 0xea2d, 0xe84d, 0xe909, 0x0005,
            0xea2d, 0xe84d, 0xe909, 0x0005, 0xea2d, 0xe84d, 0xe909, 0x0030, 0xea2d, 0x0008, 0xea0b, 0xe4ca, 0x0001,
            0xe9a5, 0x812f,
        ],
    ),
    (
        0xe91d,
        &[
            0xe517, 0xe945, 0x0003, 0xe983, 0xe51f, 0x0001, 0xfff4, 0x0004, 0xea4f, 0xe51f, 0xfffc, 0xfff8, 0x0005,
            0xea71, 0xe969, 0x0011, 0xea4f, 0x0011, 0xe983, 0x812f, 0x0003, 0xe9a5, 0xe51f, 0xffff, 0xfff4, 0x0004,
            0xea93, 0xe51f, 0x0004, 0xfff8, 0x0005, 0xeab5, 0xe976, 0x0011, 0xea93, 0x0011, 0xe9a5, 0x812f,
        ],
    ),
];

fn entry(pointer: u16) -> Option<(u16, u16)> {
    for (start, words) in SCRIPTS {
        let mut index = 0;
        while index < words.len() {
            let code = words[index];
            let count = if negative(code) { arguments(code)? } else { 1 };
            if *start + (index * 2) as u16 == pointer {
                return Some((code, count));
            }
            index += usize::from(count) + 1;
        }
    }
    None
}

pub(super) fn script_pointer(pointer: u16) -> bool {
    entry(pointer).is_some()
}

fn pinned_scripts(rom: &[u8]) -> bool {
    SCRIPTS.iter().all(|(start, words)| {
        let offset = rom_offset(0xa60000 | u32::from(*start));
        rom.get(offset..offset + words.len() * 2).is_some_and(|bytes| {
            bytes.as_chunks::<2>().0.iter().zip(*words).all(|(bytes, word)| u16::from_le_bytes(*bytes) == *word)
        })
    })
}

/// Visit frame continuations and both outcomes of every native branch. Operand
/// words and the machine code between E71C and E73A are never entries.
fn complete_graph(rom: &[u8], start: u16) -> Result<bool> {
    let mut pending = vec![start];
    let mut visited = Vec::new();
    while let Some(pc) = pending.pop() {
        if visited.contains(&pc) {
            continue;
        }
        let Some((code, count)) = entry(pc) else { return Ok(false) };
        visited.push(pc);
        let argument = pc.wrapping_add(2);
        let next = argument.wrapping_add(count * 2);
        match code {
            0x812f => (),
            0xe4ee => {
                pending.push(rom_word(rom, 0xa60000 | u32::from(argument))?);
                pending.push(rom_word(rom, 0xa60000 | u32::from(argument + 2))?);
            }
            0xe517 | 0xe4f8 | 0xe4d2 => {
                pending.push(rom_word(rom, 0xa60000 | u32::from(argument))?);
                pending.push(next);
            }
            0x80ed => pending.push(rom_word(rom, 0xa60000 | u32::from(argument))?),
            _ => pending.push(next),
        }
    }
    Ok(true)
}

fn sound_supported(ram: &Wram<'_>) -> bool {
    ram.byte(0x644) < 16 && ram.byte(0x647) < 16
}

fn callback_state(ram: &Wram<'_>, rom: &[u8], code: u16) -> Result<bool> {
    match code {
        0xe4be => Ok(sound_supported(ram)),
        0xe904 | 0xe909 => observed_projectile_ownership::supports_call(
            ram,
            rom,
            "SpawnEprojWithRoomGfx",
            0x9642,
            i32::from(ram.word(0x7820)),
        ),
        _ => Ok(true),
    }
}

pub(super) fn admits(ram: &Wram<'_>, rom: &[u8], limit: usize) -> Result<bool> {
    let slot = ram.word(CURRENT);
    let Ok(enemy) = record(slot) else { return Ok(false) };
    let mut pc = ram.word(enemy + 0x1a);
    if !admission::boss_record(ram, slot)
        || !admission::boss_record(ram, 0)
        || !admission::pinned_rom(rom)
        || !pinned_scripts(rom)
        || !complete_graph(rom, pc)?
    {
        return Ok(false);
    }
    if ram.word(enemy + 0x12) & 4 != 0 || ram.word(enemy + 0x1c) != 1 {
        return Ok(true);
    }
    // Callback writes change orientation within this same frame. Other branch
    // inputs (carry, area and signed health) stay fixed in this instruction set.
    let mut orientation = ram.word(0x7820);
    let carrying = ram.word(0x7836) != 0;
    let low_health = ram.word(0x79f) != 2 && negative(ram.word(0x9c2).wrapping_sub(30));
    let mut calls = 0;
    loop {
        let Some((code, count)) = entry(pc) else { return Ok(false) };
        if !negative(code) {
            return Ok(true);
        }
        if calls >= limit || !callback_state(ram, rom, code)? {
            return Ok(false);
        }
        calls += 1;
        let argument = pc.wrapping_add(2);
        let next = argument.wrapping_add(count * 2);
        pc = match code {
            0x812f => return Ok(true),
            0xe517 if orientation != 0 => rom_word(rom, 0xa60000 | u32::from(argument))?,
            0xe4f8 if !carrying => rom_word(rom, 0xa60000 | u32::from(argument))?,
            0xe4ee => rom_word(rom, 0xa60000 | u32::from(argument + u16::from(!carrying) * 2))?,
            0xe4d2 if low_health => rom_word(rom, 0xa60000 | u32::from(argument))?,
            0x80ed => rom_word(rom, 0xa60000 | u32::from(argument))?,
            0xe727 => {
                orientation = 1;
                next
            }
            0xe72f => {
                orientation = 2;
                next
            }
            0xe71c => {
                orientation = 0;
                next
            }
            _ => next,
        };
    }
}

pub(super) fn callback_admits(ram: &Wram<'_>, rom: &[u8], symbol: &str, a: i32, b: i32) -> Result<bool> {
    let Ok(slot) = u16::try_from(a) else { return Ok(false) };
    if !admission::boss_record(ram, slot) || !admission::boss_record(ram, 0) || !admission::pinned_rom(rom) {
        return Ok(false);
    }
    let Some(code) = opcode(symbol) else { return Ok(false) };
    let Ok(pointer) = u32::try_from(b) else { return Ok(false) };
    if pointer >> 16 != 0xa6 || pointer & 0xffff < 0x8002 {
        return Ok(false);
    }
    let pc = (pointer as u16).wrapping_sub(2);
    if entry(pc).map(|(op, _)| op) != Some(code) || !pinned_scripts(rom) || !complete_graph(rom, pc)? {
        return Ok(false);
    }
    callback_state(ram, rom, code)
}

fn callback_pointer(rom: &[u8], symbol: &str, a: i32, b: i32) -> Result<bool> {
    let slot = u16::try_from(a).map_err(|_| "Ridley instruction slot extent")?;
    record(slot)?;
    let Some(code) = opcode(symbol) else { return Ok(false) };
    let pointer = u32::try_from(b).map_err(|_| "Ridley instruction ROM pointer required")?;
    if pointer >> 16 != 0xa6 || pointer & 0xffff < 0x8000 {
        return Ok(false);
    }
    let extent = u32::from(arguments(code).expect("known callback")) * 2;
    if (pointer & 0xffff) + extent > 0x10000 {
        return Ok(false);
    }
    if extent != 0 {
        rom_word(rom, pointer + extent - 2)?;
    }
    Ok(true)
}

impl<H: Host> Context<'_, '_, H> {
    fn callback(&mut self, code: u16, slot: u16, argument: u16) -> Result<(u16, PointerKind)> {
        let rom = PointerKind::Rom;
        let branch = PointerKind::BankOffset;
        let next = |words: u16| argument.wrapping_add(words * 2);
        match code {
            0xe517 => Ok(if self.v(0x10) != 0 { (self.rw(argument)?, branch) } else { (next(1), rom) }),
            0xe501 => {
                self.set(0xb10, self.w(0xaf6));
                self.set(0xb14, self.w(0xafa));
                self.sv(0x1d, self.rw(argument)?);
                Ok((next(1), rom))
            }
            0xe4f8 => Ok(if self.v(0x1b) != 0 { (next(1), rom) } else { (self.rw(argument)?, branch) }),
            0xe4ee => Ok((self.rw(next(u16::from(self.v(0x1b) == 0)))?, branch)),
            0xe4be => {
                self.sv(0x0f, 89);
                self.sound(2, 0x59, 6)?;
                Ok((argument, rom))
            }
            0xe4ca => {
                self.sv(0x0f, 0);
                Ok((argument, rom))
            }
            0xe727 => {
                self.sv(0x10, 1);
                Ok((argument, rom))
            }
            0xe72f | 0xe71c => {
                self.sv(0x10, if code == 0xe72f { 2 } else { 0 });
                self.flip_tail()?;
                Ok((argument, rom))
            }
            0xe4d2 => {
                if self.w(0x79f) != 2 && negative(self.w(0x9c2).wrapping_sub(30)) {
                    self.sv(0, 8);
                    Ok((self.rw(argument)?, branch))
                } else {
                    Ok((next(1), rom))
                }
            }
            0xe84d => {
                let orientation = self.v(0x10);
                let mouth_x = self.w(ENEMY + 2).wrapping_add(if orientation != 0 { 25 } else { (-25i16) as u16 });
                let mouth_y = self.w(ENEMY + 6).wrapping_sub(43);
                let raw = self
                    .angle(self.w(0xaf6).wrapping_sub(mouth_x), self.w(0xafa).wrapping_sub(mouth_y))?
                    .wrapping_add(128)
                    .wrapping_neg();
                let angle = if orientation != 0 {
                    if !(0x15..0xc0).contains(&raw) { 0x15 } else { raw.min(0x50) }
                } else if !(0x40..0xeb).contains(&raw) {
                    0xeb
                } else {
                    raw.max(0xb0)
                };
                self.sv(0x19, self.component(0x500, angle));
                self.sv(0x1a, self.component(0x500, angle.wrapping_add(64)));
                Ok((argument, rom))
            }
            0xe904 | 0xe909 => {
                self.set(0x1995, if code == 0xe904 { 0 } else { 14 });
                self.spawn(0x9642, self.v(0x10))?;
                Ok((argument, rom))
            }
            0xe51f => {
                self.set(ENEMY + 2, self.w(ENEMY + 2).wrapping_add(self.rw(argument)?));
                self.set(ENEMY + 6, self.w(ENEMY + 6).wrapping_add(self.rw(next(1))?));
                Ok((next(2), rom))
            }
            0xe969 | 0xe976 => {
                self.sa(if code == 0xe969 { 0xa6af } else { 0xb2f3 });
                self.sc((-352i16) as u16);
                Ok((argument, rom))
            }
            0x80ed => Ok((self.rw(argument)?, branch)),
            0x812f => {
                let enemy = record(slot)?;
                self.set(enemy + 0x1a, argument.wrapping_sub(2));
                Ok((0, PointerKind::Stop))
            }
            _ => Err("opcode outside whole native Ridley script closure"),
        }
    }

    pub(super) fn instruction_fixture(&mut self, symbol: &str, a: i32, b: i32) -> Result<u32> {
        if !callback_pointer(self.rom, symbol, a, b)? {
            return Err("unsupported Ridley callback fixture");
        }
        let (returned, _) = self.callback(opcode(symbol).expect("admitted callback"), a as u16, b as u16)?;
        Ok(if returned == 0 { 0 } else { 0xa60000 | u32::from(returned) })
    }

    pub(super) fn instructions(&mut self, limit: usize) -> Result<ScriptReport> {
        let slot = self.w(CURRENT);
        let enemy = record(slot)?;
        let mut report = ScriptReport::default();
        if self.w(enemy + 0x12) & 4 != 0 {
            report.frozen = true;
            return Ok(report);
        }
        let previous = self.w(enemy + 0x1c);
        self.set(enemy + 0x1c, previous.wrapping_sub(1));
        if previous != 1 {
            self.set(enemy + 0x10, self.w(enemy + 0x10) & !0x8000);
            report.timer_only = true;
            return Ok(report);
        }
        let mut pc = self.w(enemy + 0x1a);
        loop {
            if !script_pointer(pc) {
                return Err("Ridley instruction left its admitted script closure");
            }
            let code = self.rw(pc)?;
            if !negative(code) {
                self.set(enemy + 0x1c, code);
                self.set(enemy + 0x16, self.rw(pc.wrapping_add(2))?);
                self.set(enemy + 0x1a, pc.wrapping_add(4));
                self.set(enemy + 0x10, self.w(enemy + 0x10) | 0x8000);
                return Ok(report);
            }
            if report.opcodes.len() >= limit {
                return Err("Ridley instruction execution limit");
            }
            let argument = pc.wrapping_add(2);
            let (returned, kind) = self.callback(code, slot, argument)?;
            report.opcodes.push(OpcodeExecution {
                address: 0xa60000 | u32::from(code),
                argument: 0xa60000 | u32::from(argument),
                returned: if returned == 0 { 0 } else { 0xa60000 | u32::from(returned) },
                pointer_kind: kind,
            });
            if returned == 0 {
                return Ok(report);
            }
            pc = returned;
        }
    }
}
