//! Ridley's native aiming/fireball attack script from pinned bank A6.
//!
//! Enemy AI keeps its existing ownership. Admission selects a complete native
//! instruction frame before its timer, audio state or projectile pool changes.
pub use crate::enemy_projectiles::ProjectileHost;
use crate::{Wram, audio, enemy, enemy_instructions, enemy_projectiles, observed_projectile_ownership};

const ENEMY: usize = 0xf78;
const CURRENT_ENEMY: usize = 0xe54;
const BANK: u8 = 0xa6;
const ATTACK_START: u16 = 0xe73a;
const ATTACK_WORDS: &[u16] = &[
    0xe517, 0xe7b4, 0xe4d2, 0xe7ac, 0x0008, 0xe983, 0xe4be, 0x0008, 0xe9c7, 0x0002, 0xe9e9, 0xe84d, 0xe904, 0x0005,
    0xe9e9, 0xe84d, 0xe909, 0x0005, 0xe9e9, 0xe84d, 0xe909, 0x0005, 0xe9e9, 0xe84d, 0xe909, 0x0030, 0xe9e9, 0x0008,
    0xe9c7, 0xe4d2, 0xe7ac, 0x0020, 0xe983, 0xe84d, 0xe4be, 0x0008, 0xe9c7, 0x0002, 0xe9e9, 0xe84d, 0xe904, 0x0005,
    0xe9e9, 0xe84d, 0xe909, 0x0005, 0xe9e9, 0xe84d, 0xe909, 0x0005, 0xe9e9, 0xe84d, 0xe909, 0x0030, 0xe9e9, 0x0008,
    0xe9c7, 0xe4ca, 0x0001, 0xe983, 0x812f, 0xe4d2, 0xe820, 0x0008, 0xe9a5, 0xe4be, 0x0008, 0xea0b, 0x0002, 0xea2d,
    0xe84d, 0xe904, 0x0005, 0xea2d, 0xe84d, 0xe909, 0x0005, 0xea2d, 0xe84d, 0xe909, 0x0005, 0xea2d, 0xe84d, 0xe909,
    0x0030, 0xea2d, 0x0008, 0xea0b, 0xe4d2, 0xe820, 0x0020, 0xe9a5, 0xe4be, 0x0008, 0xea0b, 0x0002, 0xea2d, 0xe84d,
    0xe904, 0x0005, 0xea2d, 0xe84d, 0xe909, 0x0005, 0xea2d, 0xe84d, 0xe909, 0x0005, 0xea2d, 0xe84d, 0xe909, 0x0030,
    0xea2d, 0x0008, 0xea0b, 0xe4ca, 0x0001, 0xe9a5, 0x812f,
];

/// Native C distinguishes an immediate bank offset, a ROM data pointer and zero.
/// All nonzero results also carry their canonical banked address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerKind {
    Stop,
    BankOffset,
    Rom,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpcodeExecution {
    pub address: u32,
    pub argument: u32,
    pub returned: u32,
    pub pointer_kind: PointerKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstructionEffect {
    Sound { channel: u8, sound: u16, maximum: u8 },
    RoomProjectile { definition: u16, parameter: u16, source_f: u16, slot: Option<u16> },
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstructionReport {
    pub frozen: bool,
    pub timer_only: bool,
    pub opcodes: Vec<OpcodeExecution>,
    pub effects: Vec<InstructionEffect>,
}

fn record(k: i32) -> Result<usize, &'static str> {
    if !(0..0x800).contains(&k) || k & 0x3f != 0 {
        return Err("aligned enemy record below 0x800 required");
    }
    Ok(ENEMY + k as usize)
}
fn pointer(rom: &[u8], p: i32, extent: usize) -> Result<u32, &'static str> {
    let p = u32::try_from(p).map_err(|_| "bank A6 ROM pointer required")?;
    if p >> 16 != u32::from(BANK) || p & 0xffff < 0x8000 {
        return Err("bank A6 ROM pointer required");
    }
    let low = (p & 0xffff) as usize;
    if low + extent > 0x10000 {
        return Err("Ridley instruction read crosses bank boundary");
    }
    let offset = 0x130000 + (low & 0x7fff);
    if offset.checked_add(extent).is_none_or(|end| end > rom.len()) {
        return Err("Ridley instruction read outside ROM");
    }
    Ok(p)
}
fn word(rom: &[u8], p: u32) -> Result<u16, &'static str> {
    pointer(rom, p as i32, 2)?;
    let offset = 0x130000 + (p as usize & 0x7fff);
    Ok(u16::from_le_bytes([rom[offset], rom[offset + 1]]))
}
fn advance(rom: &[u8], p: u32, bytes: i32) -> Result<u32, &'static str> {
    let low = (p & 0xffff) as i32 + bytes;
    if !(0x8000..=0xffff).contains(&low) {
        return Err("Ridley instruction result crosses bank boundary");
    }
    pointer(rom, ((u32::from(BANK) << 16) | low as u32) as i32, 0)
}
fn branch(rom: &[u8], p: u32) -> Result<u32, &'static str> {
    let target = (u32::from(BANK) << 16) | u32::from(word(rom, p)?);
    pointer(rom, target as i32, 0)
}
fn low_health(ram: &Wram<'_>) -> bool {
    ram.word(0x79f) != 2 && (ram.word(0x9c2).wrapping_sub(30) as i16) < 0
}
fn symbol(opcode: u16) -> Option<&'static str> {
    match opcode {
        0xe517 => Some("Ridley_Instr_1"),
        0xe4be => Some("Ridley_Instr_5"),
        0xe4ca => Some("Ridley_Instr_6"),
        0xe4d2 => Some("Ridley_Instr_10"),
        0xe84d => Some("Ridley_Instr_11"),
        0xe904 => Some("Ridley_Instr_12"),
        0xe909 => Some("Ridley_Instr_13"),
        0x812f => Some("EnemyInstr_Sleep"),
        _ => None,
    }
}
fn opcode_size(opcode: u16) -> usize {
    if opcode & 0x8000 == 0 || matches!(opcode, 0xe517 | 0xe4d2) { 2 } else { 1 }
}
fn script_entry(pc: u16) -> bool {
    let mut i = 0;
    while i < ATTACK_WORDS.len() {
        if ATTACK_START + (i * 2) as u16 == pc {
            return true;
        }
        i += opcode_size(ATTACK_WORDS[i]);
    }
    false
}
fn pinned_attack(rom: &[u8]) -> bool {
    let offset = 0x130000 + usize::from(ATTACK_START & 0x7fff);
    let Some(bytes) = rom.get(offset..offset + ATTACK_WORDS.len() * 2) else { return false };
    bytes.as_chunks::<2>().0.iter().zip(ATTACK_WORDS).all(|(b, word)| u16::from_le_bytes(*b) == *word)
}
fn sound_supported(ram: &Wram<'_>) -> bool {
    ram.byte(0x644) < 16 && ram.byte(0x647) < 16
}
fn validate_callback(ram: &Wram<'_>, rom: &[u8], name: &str, k: i32, p: i32) -> Result<bool, &'static str> {
    let e = record(k)?;
    if ram.byte(e + 0x2e) != BANK {
        return Ok(false);
    }
    let extent = match name {
        "Ridley_Instr_1" | "Ridley_Instr_10" => 2,
        "Ridley_Instr_5" | "Ridley_Instr_6" | "Ridley_Instr_11" | "Ridley_Instr_12" | "Ridley_Instr_13"
        | "EnemyInstr_Sleep" => 0,
        _ => return Ok(false),
    };
    let p = pointer(rom, p, extent)?;
    match name {
        "Ridley_Instr_1" if ram.word(0x7820) != 0 => {
            branch(rom, p)?;
        }
        "Ridley_Instr_1" => {
            advance(rom, p, 2)?;
        }
        "Ridley_Instr_10" if low_health(ram) => {
            branch(rom, p)?;
        }
        "Ridley_Instr_10" => {
            advance(rom, p, 2)?;
        }
        "EnemyInstr_Sleep" => {
            advance(rom, p, -2)?;
        }
        "Ridley_Instr_5" => return Ok(sound_supported(ram)),
        "Ridley_Instr_12" | "Ridley_Instr_13" => {
            return observed_projectile_ownership::supports_call(
                ram,
                rom,
                "SpawnEprojWithRoomGfx",
                0x9642,
                i32::from(ram.word(0x7820)),
            );
        }
        _ => (),
    }
    Ok(true)
}

#[derive(Clone, Copy)]
struct PlannedOpcode {
    opcode: u16,
    argument: u32,
    returned: u32,
    kind: PointerKind,
}
enum End {
    Frozen,
    TimerOnly,
    Frame { pc: u32, duration: u16, spritemap: u16 },
    Sleep,
}
struct FramePlan {
    slot: i32,
    enemy: usize,
    opcodes: Vec<PlannedOpcode>,
    end: End,
}
fn plan(ram: &Wram<'_>, rom: &[u8], limit: usize) -> Result<Option<FramePlan>, &'static str> {
    let slot = i32::from(ram.word(CURRENT_ENEMY));
    let enemy = record(slot)?;
    if ram.byte(enemy + 0x2e) != BANK
        || !matches!(ram.word(enemy), 0xe13f | 0xe17f)
        || !script_entry(ram.word(enemy + 0x1a))
        || !pinned_attack(rom)
    {
        return Ok(None);
    }
    let mut plan = FramePlan { slot, enemy, opcodes: Vec::new(), end: End::TimerOnly };
    if ram.word(enemy + 0x12) & 4 != 0 {
        plan.end = End::Frozen;
        return Ok(Some(plan));
    }
    if ram.word(enemy + 0x1c) != 1 {
        return Ok(Some(plan));
    }
    let mut pc = (u32::from(BANK) << 16) | u32::from(ram.word(enemy + 0x1a));
    loop {
        if !script_entry(pc as u16) {
            return Ok(None);
        }
        let opcode = word(rom, pc)?;
        if opcode & 0x8000 == 0 {
            let spritemap = word(rom, advance(rom, pc, 2)?)?;
            advance(rom, pc, 4)?;
            plan.end = End::Frame { pc, duration: opcode, spritemap };
            return Ok(Some(plan));
        }
        if plan.opcodes.len() >= limit {
            return Err("Ridley enemy instruction execution limit reached");
        }
        let Some(name) = symbol(opcode) else { return Ok(None) };
        let argument = advance(rom, pc, 2)?;
        if !validate_callback(ram, rom, name, slot, argument as i32)? {
            return Ok(None);
        }
        let (returned, kind) = match opcode {
            0xe517 if ram.word(0x7820) != 0 => (branch(rom, argument)?, PointerKind::BankOffset),
            0xe4d2 if low_health(ram) => (branch(rom, argument)?, PointerKind::BankOffset),
            0xe517 | 0xe4d2 => (advance(rom, argument, 2)?, PointerKind::Rom),
            0x812f => (0, PointerKind::Stop),
            _ => (argument, PointerKind::Rom),
        };
        plan.opcodes.push(PlannedOpcode { opcode, argument, returned, kind });
        if returned == 0 {
            plan.end = End::Sleep;
            return Ok(Some(plan));
        }
        pc = returned;
    }
}

/// Immutable ownership decision. A refusal preserves WRAM and all host words.
/// An error after successful admission belongs to Rust; a live caller must stop.
pub fn supports_call(ram: &Wram<'_>, rom: &[u8], name: &str, a: i32, b: i32) -> Result<bool, &'static str> {
    if name == "ProcessEnemyInstructions" {
        return Ok(plan(ram, rom, 4096)?.is_some());
    }
    validate_callback(ram, rom, name, a, b)
}

fn aim(ram: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    let orientation = ram.word(0x7820);
    let mouth_x = ram.word(0xf7a).wrapping_add(if orientation != 0 { 25 } else { (-25_i16) as u16 });
    let mouth_y = ram.word(0xf7e).wrapping_sub(43);
    let dx = ram.word(0xaf6).wrapping_sub(mouth_x);
    let dy = ram.word(0xafa).wrapping_sub(mouth_y);
    let angle = enemy::dispatch_rom(ram, rom, "CalculateAngleFromXY", i32::from(dx), i32::from(dy))? as u8;
    let raw = angle.wrapping_add(0x80).wrapping_neg();
    let clamped = if orientation != 0 {
        match raw {
            0..=0x14 | 0xc0..=0xff => 0x15,
            0x15..=0x4f => raw,
            _ => 0x50,
        }
    } else {
        match raw {
            0..=0x3f | 0xeb..=0xff => 0xeb,
            0x40..=0xaf => 0xb0,
            _ => raw,
        }
    };
    ram.set_word(0x7832, enemy_projectiles::beam_component(0x500, clamped));
    ram.set_word(0x7834, enemy_projectiles::beam_component(0x500, clamped.wrapping_add(64)));
    Ok(())
}
fn callback(
    ram: &mut Wram<'_>,
    rom: &[u8],
    host: &mut ProjectileHost,
    name: &str,
    k: i32,
    p: u32,
    effects: &mut Vec<InstructionEffect>,
) -> Result<u32, &'static str> {
    match name {
        "Ridley_Instr_1" => {
            if ram.word(0x7820) != 0 {
                branch(rom, p)
            } else {
                advance(rom, p, 2)
            }
        }
        "Ridley_Instr_5" => {
            ram.set_word(0x781e, 89);
            audio::queue_sfx(ram, 1, 0x5906)?;
            effects.push(InstructionEffect::Sound { channel: 1, sound: 0x59, maximum: 6 });
            Ok(p)
        }
        "Ridley_Instr_6" => {
            ram.set_word(0x781e, 0);
            Ok(p)
        }
        "Ridley_Instr_10" => {
            if low_health(ram) {
                let target = branch(rom, p)?;
                ram.set_word(0x7800, 8);
                Ok(target)
            } else {
                advance(rom, p, 2)
            }
        }
        "Ridley_Instr_11" => {
            aim(ram, rom)?;
            Ok(p)
        }
        "Ridley_Instr_12" | "Ridley_Instr_13" => {
            let source_f = if name == "Ridley_Instr_12" { 0 } else { 14 };
            let parameter = ram.word(0x7820);
            let free = (0..18).rev().map(|i| i * 2).find(|slot| ram.word(0x1997 + slot) == 0).map(|slot| slot as u16);
            ram.set_word(0x1995, source_f);
            observed_projectile_ownership::dispatch_host(
                ram,
                rom,
                host,
                "SpawnEprojWithRoomGfx",
                0x9642,
                i32::from(parameter),
            )?;
            effects.push(InstructionEffect::RoomProjectile { definition: 0x9642, parameter, source_f, slot: free });
            Ok(p)
        }
        "EnemyInstr_Sleep" => enemy_instructions::dispatch_rom(ram, rom, name, k, p as i32),
        _ => Err("unsupported Ridley enemy instruction"),
    }
}

/// The complete admitted native frame, including its aiming/spawning callbacks.
pub fn process_instructions(
    ram: &mut Wram<'_>,
    rom: &[u8],
    host: &mut ProjectileHost,
    limit: usize,
) -> Result<InstructionReport, &'static str> {
    let plan = plan(ram, rom, limit)?.ok_or("unsupported Ridley enemy instruction frame")?;
    let mut report = InstructionReport::default();
    if matches!(plan.end, End::Frozen) {
        report.frozen = true;
        return Ok(report);
    }
    ram.set_word(plan.enemy + 0x1c, ram.word(plan.enemy + 0x1c).wrapping_sub(1));
    if matches!(plan.end, End::TimerOnly) {
        ram.set_word(plan.enemy + 0x10, ram.word(plan.enemy + 0x10) & !0x8000);
        report.timer_only = true;
        return Ok(report);
    }
    for step in plan.opcodes {
        let name = symbol(step.opcode).ok_or("unsupported Ridley enemy instruction")?;
        let returned = callback(ram, rom, host, name, plan.slot, step.argument, &mut report.effects)?;
        if returned != step.returned {
            return Err("Ridley enemy instruction left its admitted control flow");
        }
        report.opcodes.push(OpcodeExecution {
            address: (u32::from(BANK) << 16) | u32::from(step.opcode),
            argument: step.argument,
            returned,
            pointer_kind: step.kind,
        });
    }
    if let End::Frame { pc, duration, spritemap } = plan.end {
        ram.set_word(plan.enemy + 0x1c, duration);
        ram.set_word(plan.enemy + 0x16, spritemap);
        ram.set_word(plan.enemy + 0x1a, advance(rom, pc, 4)? as u16);
        ram.set_word(plan.enemy + 0x10, ram.word(plan.enemy + 0x10) | 0x8000);
    }
    Ok(report)
}

/// Exact offline signatures use the supplied enemy offset and banked ROM pointer.
pub fn dispatch_host(
    ram: &mut Wram<'_>,
    rom: &[u8],
    host: &mut ProjectileHost,
    name: &str,
    a: i32,
    b: i32,
) -> Result<u32, &'static str> {
    if name == "ProcessEnemyInstructions" {
        process_instructions(ram, rom, host, 4096)?;
        return Ok(0);
    }
    if !validate_callback(ram, rom, name, a, b)? {
        return Err("unsupported Ridley enemy instruction");
    }
    callback(ram, rom, host, name, a, b as u32, &mut Vec::new())
}
pub fn dispatch_rom(ram: &mut Wram<'_>, rom: &[u8], name: &str, a: i32, b: i32) -> Result<u32, &'static str> {
    dispatch_host(ram, rom, &mut ProjectileHost::default(), name, a, b)
}
