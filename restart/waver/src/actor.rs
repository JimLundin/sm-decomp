use crate::{Services, memory::*};

fn sign_extend_8(value: u16) -> u16 {
    if value & 0x80 != 0 { value | 0xff00 } else { value }
}
pub(crate) fn select_animation(ram: &mut [u8], rom: &[u8]) -> Result<(), String> {
    let (_, base) = current(ram)?;
    let selector = word(ram, base + F);
    if selector != word(ram, base + C) {
        if selector > 3 {
            return Err("Waver service produced unsupported animation selector".into());
        }
        let instruction = word(rom, TABLE + usize::from(selector) * 2);
        put(ram, base + C, selector);
        put(ram, base + INSTRUCTION, instruction);
        put(ram, base + INSTRUCTION_TIMER, 1);
        put(ram, base + TIMER, 0);
    }
    Ok(())
}
pub(crate) fn init(ram: &mut [u8], rom: &[u8]) -> Result<(), String> {
    let (_, base) = current(ram)?;
    put(ram, base + B, 1);
    put(ram, base + A, 0x8000);
    if word(ram, base + PARAMETER) & 1 == 0 {
        put(ram, base + B, sign_extend_8(0xfe));
        put(ram, base + A, sign_extend_8(0x8000));
    }
    put(ram, base + F, 0);
    put(ram, base + C, 0);
    put(ram, base + E, 0);
    put(ram, base + INSTRUCTION, 0x86a7);
    put(ram, base + F, word(ram, base + PARAMETER) & 1);
    select_animation(ram, rom)
}
fn collision(value: u8) -> Result<bool, String> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err("invalid Waver native collision result".into()),
    }
}
pub(crate) fn main(ram: &mut [u8], rom: &[u8], services: &mut impl Services) -> Result<(), String> {
    // Native C keeps this E pointer across services, but reads its fields again.
    let (k, base) = current(ram)?;
    let amount = ((u32::from(word(ram, base + B)) << 16) | u32::from(word(ram, base + A))) as i32;
    if collision(services.move_horizontal(ram, k, amount)?)? {
        let packed = (word(ram, base + A) >> 8) | (word(ram, base + B) << 8);
        let negated = packed.wrapping_neg();
        put(ram, base + B, sign_extend_8((negated & 0xff00) >> 8));
        let fraction = (-(i32::from(packed as u8 as i8)) * 256) as u16;
        put(ram, base + A, fraction);
        put(ram, base + F, (word(ram, base + F) ^ 1) & 1);
        select_animation(ram, rom)?;
    } else {
        let angle = word(ram, base + D) & 0xff;
        let sine = services.sine(ram, angle, 4)?;
        let k = current(ram)?.0;
        let amount = i32::from(sine as i16) * 65536;
        if collision(services.move_vertical(ram, k, amount)?)? {
            put(ram, base + D, word(ram, base + D).wrapping_add(0x80) & 0xff);
        } else {
            put(ram, base + D, word(ram, base + D).wrapping_add(2));
        }
    }
    if word(ram, base + D) & 0x7f == 56 {
        put(ram, base + F, word(ram, base + F) | 2);
        select_animation(ram, rom)?;
    }
    if word(ram, base + E) != 0 {
        put(ram, base + E, 0);
        put(ram, base + F, word(ram, base + F) & 1);
        select_animation(ram, rom)?;
    }
    Ok(())
}
