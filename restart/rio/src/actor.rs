use crate::{Services, memory::*, sine::SINE};
fn amount(ram: &[u8], at: usize) -> i32 {
    i32::from(word(ram, at) as i16) * 256
}
fn negate(ram: &mut [u8], at: usize) {
    let value = word(ram, at).wrapping_neg();
    put(ram, at, value);
}
fn horizontal(services: &mut impl Services, ram: &mut [u8], k: u16, base: usize) -> Result<u8, String> {
    let delta = amount(ram, base + D);
    collision(services.move_horizontal(ram, k, delta)?)
}
fn vertical(services: &mut impl Services, ram: &mut [u8], k: u16, base: usize) -> Result<u8, String> {
    let delta = amount(ram, base + C);
    collision(services.move_vertical(ram, k, delta)?)
}
fn collision(value: u8) -> Result<u8, String> {
    if value > 1 {
        return Err("Rio movement service returned an invalid collision result".into());
    }
    Ok(value)
}
pub(crate) fn state(
    state: usize,
    k: u16,
    base: usize,
    ram: &mut [u8],
    rom: &[u8],
    services: &mut impl Services,
) -> Result<(), String> {
    match state {
        0 => {
            if near(ram, base) {
                put(ram, base + C, rom_word(rom, 0x113bbb)?);
                put(ram, base + D, rom_word(rom, 0x113bbf)?);
                if (word(ram, 0xaf6).wrapping_sub(word(ram, base + 2)) as i16) < 0 {
                    negate(ram, base + D);
                }
                animation(ram, 0xbb7f)?;
                put(ram, base + B, STATES[2]);
                if on_screen(ram, current(ram)?) {
                    sound(ram);
                }
            }
        }
        1 => {
            if word(ram, base + E) != 0 {
                put(ram, base + E, 0);
                animation(ram, 0xbb53)?;
                put(ram, base + B, STATES[0]);
            }
        }
        2 => {
            let hit_x = horizontal(services, ram, k, base)? != 0;
            if hit_x {
                negate(ram, base + D);
            }
            // Preserve C's horizontal short circuit; reread C after full copyback.
            if hit_x || vertical(services, ram, k, base)? != 0 {
                negate(ram, base + C);
                put(ram, base + B, STATES[3]);
            } else {
                let speed = word(ram, base + C).wrapping_sub(24);
                put(ram, base + C, speed);
                if (speed as i16) < 0 {
                    let old_d = word(ram, base + D);
                    put(ram, base + A, old_d);
                    put(ram, base + D, 0);
                    put(ram, base + C, 0);
                    put(ram, base + B, STATES[4]);
                } else if word(ram, base + E) != 0 {
                    put(ram, base + E, 0);
                    animation(ram, 0xbb97)?;
                }
            }
        }
        3 => {
            if horizontal(services, ram, k, base)? != 0 {
                negate(ram, base + D);
            }
            if vertical(services, ram, k, base)? != 0 {
                animation(ram, 0xbba3)?;
                put(ram, base + B, STATES[1]);
            } else {
                let value = word(ram, base + C).wrapping_sub(24);
                put(ram, base + C, value);
            }
        }
        4 => {
            if (word(ram, base + 6).wrapping_sub(word(ram, 0xafa)) as i16) >= 0 {
                let old_a = word(ram, base + A);
                put(ram, base + D, old_a);
                put(ram, base + C, 0xffff);
                put(ram, base + B, STATES[3]);
            } else {
                let angle = services.angle(ram, k)?;
                if angle > 255 {
                    return Err("Rio angle service returned an invalid table index".into());
                }
                put(ram, base + D, SINE[usize::from(angle) + 64] as u16);
                put(ram, base + C, SINE[usize::from(angle)] as u16);
                horizontal(services, ram, k, base)?;
                vertical(services, ram, k, base)?;
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
