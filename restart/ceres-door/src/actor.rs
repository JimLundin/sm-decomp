//! Full six-state CeresDoor main operation and initialization, bank A6 F6C5..F900.
use crate::{Host, Result, helpers, memory::*};

pub(crate) const STATES: [u16; 6] = [0xf76b, 0xf770, 0xf7a5, 0xf7bd, 0xf7dc, 0xf850];

pub(crate) fn init(ram: &mut Wram<'_>, rom: &Rom<'_>, enemy: usize) -> Result<()> {
    let parameter = ram.word(enemy + PARAMETER);
    let state = rom.word(0xf72b + 2 * parameter)?;
    let instruction = rom.word(0xf52c + 2 * parameter)?;
    if !STATES.contains(&state) || instruction < 0x8000 {
        return Err("invalid Ceres door initialization table".into());
    }
    let tail = usize::from(ram.word(0x330));
    if parameter == 2 && tail + 7 > 0x200 {
        return Err("Ceres door VRAM queue exceeds its WRAM allocation".into());
    }
    let escape = ram.word(0x93f) != 0;
    let source = if escape { 0xf50e } else { 0xf4ee };
    let palette = rom.bytes(source, 30)?;
    let special_palette = !escape && parameter == 3;
    let destination = if special_palette { 0xc342 } else { 0xc3e2 };

    ram.set(enemy + 0x16, 0xfac7);
    ram.set(enemy + 0x1c, 1);
    ram.set(enemy + 0x18, 0);
    ram.set(enemy + 0x20, 0);
    ram.set(enemy + PALETTE, 1024);
    ram.set(enemy + VAR_A, state);
    ram.set(enemy + 0x1a, instruction);
    ram.set(enemy + VAR_B, 0);
    if parameter == 2 {
        ram.0[0xd0 + tail..0xd7 + tail].copy_from_slice(&[0, 4, 0, 0xc4, 0xb0, 0, 0x70]);
        ram.set(0x330, (tail + 7) as u16);
    }
    if !special_palette {
        ram.set(enemy + PALETTE, 3584);
    }
    ram.0[destination..destination + 30].copy_from_slice(palette);
    Ok(())
}

pub(crate) fn main(ram: &mut Wram<'_>, rom: &Rom<'_>, enemy: usize, host: &mut impl Host) -> Result<()> {
    match ram.word(enemy + VAR_A) {
        0xf76b => helpers::quake(ram, 0x14),
        0xf770 => helpers::quake(ram, 0x1d),
        0xf7a5 => {
            ram.mask(enemy + PROPERTIES, 0, 0x100);
            if ram.word(0x93f) & 1 != 0 {
                ram.set(enemy + PALETTE, 3584);
                ram.mask(enemy + PROPERTIES, 0x100, 0);
            }
        }
        0xf7bd => {
            helpers::animation(ram, rom)?.apply(ram);
            if ram.word(0x93f) >= 2 {
                ram.set(enemy + VAR_A, 0xf7dc);
                ram.set(enemy + VAR_D, 48);
                // The original deliberately shares these counters with slot 0.
                ram.set(ENEMY + VAR_E, 0);
                ram.set(ENEMY + VAR_F, 0);
            }
        }
        0xf7dc => escape(ram, rom, enemy, host)?,
        0xf850 => helpers::animation(ram, rom)?.apply(ram),
        _ => return Err("unsupported Ceres door main state".into()),
    }
    Ok(())
}

fn escape(ram: &mut Wram<'_>, rom: &Rom<'_>, enemy: usize, host: &mut impl Host) -> Result<()> {
    let countdown = ram.word(enemy + VAR_D).wrapping_sub(1);
    if countdown & 0x8000 != 0 {
        ram.set(enemy + VAR_D, countdown);
        ram.mask(enemy + PROPERTIES, 0, 0x100);
        ram.set(enemy + VAR_A, 0xf850);
        if ram.word(0x93f) != 0 {
            ram.set(0x93f, 0x8000);
        }
        return Ok(());
    }
    let delay = ram.word(ENEMY + VAR_E).wrapping_sub(1);
    if delay & 0x8000 == 0 {
        ram.set(enemy + VAR_D, countdown);
        ram.set(ENEMY + VAR_E, delay);
        return Ok(());
    }
    let previous = ram.word(ENEMY + VAR_F).wrapping_sub(1);
    let puff = if previous & 0x8000 != 0 { 3 } else { previous };
    if puff > 3 {
        return Err("Ceres door puff coordinate index exceeds four positions".into());
    }
    let x = ram.word(enemy + 2).wrapping_add(rom.word(0xf840 + 4 * puff)?);
    let y = ram.word(enemy + 6).wrapping_add(rom.word(0xf842 + 4 * puff)?);
    helpers::sound_preflight(ram, 1)?;
    let random = helpers::next_random(ram.word(0x5e5));
    let parameter = if random < 0x4000 { 12 } else { 3 };

    ram.set(enemy + VAR_D, countdown);
    ram.set(ENEMY + VAR_E, 4);
    ram.set(ENEMY + VAR_F, puff);
    ram.set(0x5e5, random);
    host.spawn_eproj_with_room_gfx(ram.0, 0xe509, parameter, [x, y])?;
    helpers::sound(ram, 1, 0x25)
}
