//! Complete E23F Ceres door actor and its BA48 solid-block setup.
//! Recovered-C source revision and exact retained dependencies are in README.md.
mod actor;
mod helpers;
mod memory;

pub use memory::WRAM_SIZE;
use memory::{ENEMY, PARAMETER, PROPERTIES, Rom, VAR_B, Wram};
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Entry {
    Init = 0,
    Main,
    Instr1,
    Instr2,
    Instr3,
    Instr4,
    Instr5,
    Instr6,
    Instr7,
    Instr8,
    SetTangible,
    SetInvisible,
    SolidSetup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Done,
    InstructionOffset(u16),
    SetupStatus(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Handled(Output),
    Refused,
}

/// Sole synchronous foreign gameplay operation, admitted only for E509 puffs.
/// Publish supplied WRAM and the external spawn point before calling actual C;
/// copy WRAM back before returning. Failure is owned and prohibits C retry.
pub trait Host {
    fn spawn_eproj_with_room_gfx(&mut self, ram: &mut [u8], header: u16, parameter: u16, point: [u16; 2])
    -> Result<()>;
}

/// Execute one selected native dispatcher entry against its private WRAM copy.
/// Index is an enemy byte index, or a PLM byte index for SolidSetup. Instruction
/// is an A6 offset for the ten instruction callbacks and ignored for other roots.
/// Wrong identities/parameters refuse unchanged; malformed admitted state errors.
/// All checks that can refuse or detect malformed input precede gameplay effects.
pub fn run(
    entry: Entry,
    index: u16,
    instruction: u16,
    ram: &mut [u8],
    rom: &[u8],
    host: &mut impl Host,
) -> Result<Outcome> {
    let mut ram = Wram::new(ram)?;
    if entry == Entry::SolidSetup {
        return solid_setup(&mut ram, index);
    }
    if index >= 0x800 || index & 0x3f != 0 {
        return Err("invalid Ceres door enemy byte index".into());
    }
    let enemy = ENEMY + usize::from(index);
    if ram.word(enemy) != 0xe23f || ram.0[enemy + 0x2e] != 0xa6 || ram.word(enemy + PARAMETER) > 6 {
        return Ok(Outcome::Refused);
    }
    let rom = Rom(rom);
    if matches!(entry, Entry::Init | Entry::Main) {
        if ram.word(0xe54) != index {
            return Err("Ceres door current enemy differs from supplied index".into());
        }
        match entry {
            Entry::Init => actor::init(&mut ram, &rom, enemy)?,
            Entry::Main => actor::main(&mut ram, &rom, enemy, host)?,
            _ => unreachable!(),
        }
        return Ok(Outcome::Handled(Output::Done));
    }
    if !(0x8000..=0xfffd).contains(&instruction) {
        return Err("invalid Ceres door instruction offset".into());
    }
    let next = match entry {
        Entry::Instr1 => {
            ram.mask(enemy + PROPERTIES, 0x400, 0);
            instruction
        }
        Entry::Instr2 => {
            ram.mask(enemy + PROPERTIES, 0x100, 0);
            instruction
        }
        Entry::Instr3 => {
            ram.set(enemy + VAR_B, 1);
            instruction
        }
        Entry::Instr4 => {
            let area = usize::from(ram.word(0x79f));
            if area >= 8 {
                return Err("invalid Ceres door area index".into());
            }
            branch(&rom, instruction, ram.word(0xd828 + area) & 1 != 0)?
        }
        Entry::Instr5 => {
            ram.set(enemy + VAR_B, 0);
            ram.mask(enemy + PROPERTIES, 0x100, 0);
            instruction
        }
        Entry::Instr6 => {
            // abs16 receives the wrapped 16-bit coordinate subtraction.
            let dx = (ram.word(enemy + 2).wrapping_sub(ram.word(0xaf6)) as i16).unsigned_abs();
            let dy = (ram.word(enemy + 6).wrapping_sub(ram.word(0xafa)) as i16).unsigned_abs();
            branch(&rom, instruction, dx < 0x30 && dy < 0x30)?
        }
        Entry::Instr7 => {
            helpers::sound(&mut ram, 2, 0x2c)?;
            instruction
        }
        Entry::Instr8 => branch(&rom, instruction, ram.word(0x93f) != 0)?,
        Entry::SetTangible => {
            ram.mask(enemy + PROPERTIES, 0, 0x400);
            instruction
        }
        Entry::SetInvisible => {
            ram.mask(enemy + PROPERTIES, 0, 0x100);
            instruction
        }
        _ => unreachable!(),
    };
    Ok(Outcome::Handled(Output::InstructionOffset(next)))
}

fn branch(rom: &Rom<'_>, instruction: u16, skip: bool) -> Result<u16> {
    if skip {
        return Ok(instruction + 2);
    }
    let next = rom.word(instruction)?;
    if next < 0x8000 {
        return Err("Ceres door branch target outside bank A6 ROM".into());
    }
    Ok(next)
}

fn solid_setup(ram: &mut Wram<'_>, index: u16) -> Result<Outcome> {
    if index >= 80 || index & 1 != 0 {
        return Err("invalid Ceres door PLM byte index".into());
    }
    if ram.word(0x1c37 + usize::from(index)) != 0xba48 {
        return Ok(Outcome::Refused);
    }
    let stride = ram.word(0x7a5).wrapping_mul(2);
    let start = ram.word(0x1c87 + usize::from(index));
    let offsets: [usize; 4] =
        std::array::from_fn(|i| 0x10002 + usize::from(start.wrapping_add(stride.wrapping_mul(i as u16)) & !1));
    if offsets.iter().any(|&at| at + 2 > WRAM_SIZE) {
        return Err("Ceres door solid setup exceeds complete WRAM".into());
    }
    for at in offsets {
        ram.mask(at, 0xf000, 0x8000);
    }
    Ok(Outcome::Handled(Output::SetupStatus(0)))
}

#[cfg(test)]
mod tests;
