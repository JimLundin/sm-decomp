//! Complete Skree actor boundary from pinned recovered C bank A3.
//! Runtime services retain their existing implementation and synchronous copyback.
#![forbid(unsafe_code)]

const WRAM_SIZE: usize = 0x20000;
const BASE: usize = 0xf78;
const CURRENT: usize = 0xe54;
const SAMUS_X: usize = 0xaf6;
const COLLISION: usize = 0x18a6;
const PROJECTILE_TYPE: usize = 0xc18;
const SPAWN: usize = 0x7000;
const X: usize = 2;
const Y: usize = 6;
const PROPERTIES: usize = 0xe;
const HEALTH: usize = 0x14;
const TIMER: usize = 0x18;
const ANIMATION: usize = 0x1a;
const ANIMATION_TIMER: usize = 0x1c;
const PALETTE: usize = 0x1e;
const VRAM: usize = 0x20;
const A: usize = 0x30;
const B: usize = 0x32;
const C: usize = 0x34;
const D: usize = 0x36;
const E: usize = 0x38;
const STATES: [u16; 4] = [0xc6d5, 0xc6f7, 0xc716, 0xc77f];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    /// C ignores k and returns the unchanged instruction pointer jp.
    Instruction {
        k: u16,
        jp: u16,
    },
    Init,
    Call(u32),
    Main,
    Func1,
    Func2(u16),
    Func3,
    Func4,
    Func5,
    Shot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Done,
    Instruction(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Particle {
    DownRight,
    UpRight,
    DownLeft,
    UpLeft,
}

impl Particle {
    pub const fn definition(self) -> u16 {
        match self {
            Self::DownRight => 0x8bc2,
            Self::UpRight => 0x8bd0,
            Self::DownLeft => 0x8bde,
            Self::UpLeft => 0x8bec,
        }
    }
}

/// Admission is immutable and covers every service the entry can reach, including
/// complete WRAM copyback, ROM dependencies, effect ordering, generation, aliasing
/// and reentry guards. It must guarantee valid current enemy/projectile indices
/// after callbacks; no callback may silently fall back after taking ownership.
/// The request is the original root entry, before dispatch resolution. Admission
/// must account for local writes before services (Func3 sets A=21/properties|3;
/// Func4 decrements A; Func2 changes E/C/B) and callback-dependent subsequent reads.
/// Calls receive the complete canonical WRAM and must synchronously copy back all
/// mutations before returning. Err is terminal; the caller must not replay in C.
pub trait Services {
    fn admitted(&self, entry: Entry, ram: &[u8], rom: &[u8]) -> bool;
    /// EnemyFunc_BF8A(k, properties, signed 16.16 amount), preserving its u8 result.
    fn terrain(&mut self, ram: &mut [u8], k: u16, properties: u16, amount: i32) -> Result<u8, String>;
    /// QueueSfx2_Max6, preserving the queue's complete state and ordered effects.
    fn sound(&mut self, ram: &mut [u8], sound: u16) -> Result<(), String>;
    /// SpawnEprojWithGfx(parameter, k, particle.definition()) in bank 86.
    fn spawn(&mut self, ram: &mut [u8], parameter: u16, k: u16, particle: Particle) -> Result<(), String>;
    /// Enemy_NormalShotAI_SkipSomeParts_A3.
    fn shot(&mut self, ram: &mut [u8]) -> Result<(), String>;
    /// EnemyDeathAnimation(projectile-byte-offset, animation), NOT enemy index.
    fn death(&mut self, ram: &mut [u8], projectile_offset: u16, animation: u16) -> Result<(), String>;
}

/// None is an unchanged refusal. Validation and service admission precede writes.
/// The admitted ROM domain contains all four A3:C69C animation-table words; the
/// four-entry bound excludes recovered C reads into the following instruction.
pub fn execute(
    entry: Entry,
    ram: &mut [u8],
    rom: &[u8],
    services: &mut impl Services,
) -> Result<Option<Output>, String> {
    if ram.len() != WRAM_SIZE || record(word(ram, CURRENT)).is_none() {
        return Ok(None);
    }
    let request = entry;
    let entry = match entry {
        Entry::Main => match decode(
            0xa30000 | u32::from(word(ram, BASE + usize::from(word(ram, CURRENT)) + B)),
            word(ram, CURRENT),
        ) {
            Some(entry) => entry,
            None => return Ok(None),
        },
        Entry::Call(address) => match decode(address, word(ram, CURRENT)) {
            Some(entry) => entry,
            None => return Ok(None),
        },
        entry => entry,
    };
    let current = BASE + usize::from(word(ram, CURRENT));
    if let Entry::Func2(k) = entry {
        let Some(e) = record(k) else { return Ok(None) };
        // Func2(k) increments k's C, while Func5 rereads CURRENT.
        if word(ram, e + C) >= 3 || word(ram, current + C) >= 4 {
            return Ok(None);
        }
    }
    if matches!(entry, Entry::Func1) && word(ram, current + C) >= 3 {
        return Ok(None);
    }
    if matches!(entry, Entry::Func5) && word(ram, current + C) >= 4 {
        return Ok(None);
    }
    if matches!(entry, Entry::Func1 | Entry::Func2(_) | Entry::Func5) && (0..4).any(|i| animation(rom, i).is_none()) {
        return Ok(None);
    }
    if matches!(entry, Entry::Shot) && word(ram, COLLISION) >= 10 {
        return Ok(None);
    }
    if !services.admitted(request, ram, rom) {
        return Ok(None);
    }
    match entry {
        Entry::Instruction { jp, .. } => {
            set(ram, current + E, 1);
            return Ok(Some(Output::Instruction(jp)));
        }
        Entry::Init => {
            for field in [C, D, E] {
                set(ram, current + field, 0);
            }
            set(ram, current + ANIMATION, 0xc65e);
            set(ram, current + B, STATES[0]);
        }
        Entry::Func1 => func1(ram, rom, current)?,
        Entry::Func2(k) => func2(ram, rom, services, BASE + usize::from(k))?,
        Entry::Func3 => func3(ram, services, current)?,
        Entry::Func4 => func4(ram, services, current)?,
        Entry::Func5 => func5(ram, rom)?,
        Entry::Shot => shot(ram, services)?,
        Entry::Main | Entry::Call(_) => unreachable!("dispatch resolved before admission"),
    }
    Ok(Some(Output::Done))
}

fn decode(address: u32, current: u16) -> Option<Entry> {
    match address {
        0xa3c6d5 => Some(Entry::Func1),
        0xa3c6f7 => Some(Entry::Func2(current)),
        0xa3c716 => Some(Entry::Func3),
        0xa3c77f => Some(Entry::Func4),
        _ => None,
    }
}

fn record(k: u16) -> Option<usize> {
    (k < 0x800 && k & 63 == 0).then_some(BASE + usize::from(k))
}

fn current_record(ram: &[u8]) -> Result<usize, String> {
    record(word(ram, CURRENT)).ok_or_else(|| "Skree service returned invalid current enemy index; terminal".into())
}

fn word(ram: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([ram[offset], ram[offset + 1]])
}

fn set(ram: &mut [u8], offset: usize, value: u16) {
    ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn animation(rom: &[u8], index: u16) -> Option<u16> {
    let offset = 0x23 * 0x8000 + 0x469c + usize::from(index) * 2;
    let bytes = rom.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn func1(ram: &mut [u8], rom: &[u8], e: usize) -> Result<(), String> {
    let difference = word(ram, e + X).wrapping_sub(word(ram, SAMUS_X));
    let distance = if difference & 0x8000 != 0 { difference.wrapping_neg() } else { difference };
    if distance < 0x30 {
        let c = word(ram, e + C).wrapping_add(1);
        set(ram, e + C, c);
        func5(ram, rom)?;
        set(ram, e + B, STATES[1]);
    }
    Ok(())
}

fn func2(ram: &mut [u8], rom: &[u8], services: &mut impl Services, e: usize) -> Result<(), String> {
    if word(ram, e + E) != 0 {
        set(ram, e + E, 0);
        let c = word(ram, e + C).wrapping_add(1);
        set(ram, e + C, c);
        func5(ram, rom)?;
        set(ram, e + B, STATES[2]);
        services.sound(ram, 0x5b)?;
    }
    Ok(())
}

fn func3(ram: &mut [u8], services: &mut impl Services, e: usize) -> Result<(), String> {
    set(ram, e + A, 21);
    let properties = word(ram, e + PROPERTIES) | 3;
    set(ram, e + PROPERTIES, properties);
    let k = word(ram, CURRENT);
    if services.terrain(ram, k, properties, 6 << 16)? & 1 != 0 {
        set(ram, e + ANIMATION_TIMER, 1);
        set(ram, e + TIMER, 0);
        set(ram, e + B, STATES[3]);
        services.sound(ram, 0x5c)?;
    } else {
        let y = word(ram, e + Y).wrapping_add(6);
        set(ram, e + Y, y);
        let difference = word(ram, e + X).wrapping_sub(word(ram, SAMUS_X)) as i16;
        let step = if difference >= 0 { u16::MAX } else { 1 };
        let x = word(ram, e + X).wrapping_add(step);
        set(ram, e + X, x);
    }
    Ok(())
}

fn func4(ram: &mut [u8], services: &mut impl Services, e: usize) -> Result<(), String> {
    let old = word(ram, e + A);
    set(ram, e + A, old.wrapping_sub(1));
    if old == 1 {
        // C retains E but rereads current index for the spawn-data overlay.
        let k = word(ram, CURRENT);
        record(k).ok_or_else(|| "Skree deletion returned invalid enemy index; terminal".to_string())?;
        let tiles = word(ram, e + VRAM) | word(ram, e + PALETTE);
        set(ram, SPAWN + usize::from(k) + 6, tiles);
        set(ram, e + PALETTE, 2560);
        set(ram, e + VRAM, 0);
        let properties = word(ram, e + PROPERTIES) | 0x200;
        set(ram, e + PROPERTIES, properties);
    } else {
        if word(ram, e + A) == 8 {
            spawn_four(ram, services, 8)?;
        }
        let y = word(ram, e + Y).wrapping_add(1);
        set(ram, e + Y, y);
    }
    Ok(())
}

fn func5(ram: &mut [u8], rom: &[u8]) -> Result<(), String> {
    let e = current_record(ram)?;
    let c = word(ram, e + C);
    if c != word(ram, e + D) {
        let instruction =
            animation(rom, c).ok_or_else(|| "Skree animation unavailable after ownership; terminal".to_string())?;
        set(ram, e + D, c);
        set(ram, e + ANIMATION, instruction);
        set(ram, e + ANIMATION_TIMER, 1);
        set(ram, e + TIMER, 0);
    }
    Ok(())
}

fn spawn_four(ram: &mut [u8], services: &mut impl Services, parameter: u16) -> Result<(), String> {
    let k = word(ram, CURRENT);
    current_record(ram)?;
    services.spawn(ram, parameter, k, Particle::DownRight)?;
    services.spawn(ram, 0, k, Particle::UpRight)?;
    services.spawn(ram, 0, k, Particle::DownLeft)?;
    services.spawn(ram, 0, k, Particle::UpLeft)?;
    Ok(())
}

fn shot(ram: &mut [u8], services: &mut impl Services) -> Result<(), String> {
    services.shot(ram)?;
    let e = current_record(ram)?;
    if word(ram, e + HEALTH) == 0 {
        let parameter = word(ram, e + A);
        spawn_four(ram, services, parameter)?;
        // C rereads collision_detection_index after the complete spawn sequence.
        let collision = word(ram, COLLISION);
        if collision >= 10 {
            return Err("Skree service returned invalid projectile index; terminal".into());
        }
        let category = word(ram, PROJECTILE_TYPE + usize::from(collision) * 2) & 0xf00;
        services.death(ram, collision * 2, if category == 0x200 { 2 } else { 0 })?;
    }
    Ok(())
}
