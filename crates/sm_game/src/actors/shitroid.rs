//! The Shitroid's (baby Metroid's) hitbox separation, screen and camera tests,
//! steering and cry.
#![forbid(unsafe_code)]
use crate::actors::layouts::EnemyLayout40 as Shitroid;
use crate::effects::SoundEffects;
use crate::ram::{CompatibilityRam, RawRam as _, WorkRamAccess, slot, word};

/// What the Shitroid needs beyond work RAM and sound: hardcoded PLM spawns.
pub trait Host: WorkRamAccess + CompatibilityRam + SoundEffects {
    /// Spawn the PLM with `header` at block (`block_x`, `block_y`).
    fn spawn_hardcoded_plm(&mut self, block_x: u8, block_y: u8, header: u16);
}

fn abs16(value: u16) -> u16 {
    (value as i16).unsigned_abs()
}

/// 1 when actor `k`'s and actor `j`'s hitboxes do not overlap. Was `Shitroid_Func_1`.
pub fn separated_from_enemy(k: u16, j: u16, game: &mut impl Host) -> u8 {
    let e = i32::from(j);
    let g = i32::from(k);
    let width = (slot!(game, Shitroid[g].base.x_width) as i32
        + slot!(game, Shitroid[e].base.x_width) as i32
        + 1) as u16;
    let dx = abs16(
        slot!(game, Shitroid[e].base.x_pos).wrapping_sub(slot!(game, Shitroid[g].base.x_pos)),
    );
    if dx < width {
        let height = (slot!(game, Shitroid[g].base.y_height) as i32
            + slot!(game, Shitroid[e].base.y_height) as i32
            + 1) as u16;
        return (abs16(
            slot!(game, Shitroid[e].base.y_pos).wrapping_sub(slot!(game, Shitroid[g].base.y_pos)),
        ) >= height) as u8;
    }
    1
}

/// 1 when actor `k`'s hitbox does not overlap `rect`. Was `Shitroid_Func_2`.
/// `rect` is (x, y, half width, half height).
pub fn separated_from_rect(k: u16, rect: (u16, u16, u16, u16), game: &mut impl Host) -> u8 {
    let (x, y, w, h) = rect;
    let e = i32::from(k);
    let width = (slot!(game, Shitroid[e].base.x_width) as i32 + w as i32 + 1) as u16;
    let dx = abs16(x.wrapping_sub(slot!(game, Shitroid[e].base.x_pos)));
    if dx < width {
        let height = (slot!(game, Shitroid[e].base.y_height) as i32 + h as i32 + 1) as u16;
        return (abs16(y.wrapping_sub(slot!(game, Shitroid[e].base.y_pos))) >= height) as u8;
    }
    1
}

/// Once the camera is left of X 513: pin it at 512, rewrite the first two scroll
/// words from their low bytes, spawn the two $B767 PLMs and continue at
/// $A9:F02B. Was `Shitroid_Func_4`.
pub fn pin_camera_at_512(game: &mut impl Host) {
    if word!(game.layer1_x_pos) as i32 - 513i32 & 0x8000 != 0 {
        word!(game.layer1_x_pos = 512);
        for offset in [0, 2] {
            let value = u16::from(game.ram().read_byte(0xcd20 + offset));
            let bytes = value.to_ne_bytes();
            game.ram().write_byte(0xcd20 + offset, bytes[0]);
            game.ram().write_byte(0xcd20 + offset + 1, bytes[1]);
        }
        game.spawn_hardcoded_plm(0x30, 0x3, 0xb767);
        game.spawn_hardcoded_plm(0x1f, 0x3, 0xb767);
        let e = word!(game.cur_enemy_index) as i32;
        slot!(game, Shitroid[e].var_A = 0xa9f02bi32 as u16);
        slot!(game, Shitroid[e].var_04 = 1);
    }
}

/// 0 while actor `k` is within the screen margins, 1 otherwise. Was
/// `Shitroid_CheckIfOnScreen`.
pub fn is_off_screen(k: u16, game: &mut impl Host) -> u8 {
    let e = i32::from(k);
    let y_pos = slot!(game, Shitroid[e].base.y_pos) as i16;
    if y_pos >= 0 {
        let y = (y_pos as i32 + 96 - word!(game.layer1_y_pos) as i32) as i16;
        if y >= 0 && y as i32 - 416i32 & 0x8000 != 0 {
            let x_pos = slot!(game, Shitroid[e].base.x_pos) as i16;
            if x_pos >= 0 {
                let x = (x_pos as i32 + 16 - word!(game.layer1_x_pos) as i32) as i16;
                if x >= 0 && x as i32 - 288i32 & 0x8000 != 0 {
                    return 0;
                }
            }
        }
    }
    1
}

/// Accelerate the Y velocity by `r22` towards `r20`, capped at 1280; 1 once
/// there. Was `Shitroid_AccelerateTowardsY`.
pub fn accelerate_towards_y(k: u16, r20: u16, r22: u16, game: &mut impl Host) -> u16 {
    let e = i32::from(k);
    let mut result: u16 = 0;
    let offset = (slot!(game, Shitroid[e].base.y_pos) as i16).wrapping_sub(r20 as i16);
    if offset == 0 {
        result = result.wrapping_add(1);
    } else if offset >= 0 {
        let mut velocity = slot!(game, Shitroid[e].var_C).wrapping_sub(r22);
        if velocity as i32 + 1280 & 0x8000 != 0 {
            velocity = -1280i32 as u16;
        }
        slot!(game, Shitroid[e].var_C = velocity);
        if ((slot!(game, Shitroid[e].base.y_pos) as i32 + (velocity >> 8) as i8 as i32 - r20 as i32)
            as i16)
            <= 0
        {
            slot!(game, Shitroid[e].var_C = 0);
            result = result.wrapping_add(1);
        }
    } else {
        let mut velocity = r22.wrapping_add(slot!(game, Shitroid[e].var_C));
        if velocity as i32 - 1280i32 & 0x8000 == 0 {
            velocity = 1280;
        }
        slot!(game, Shitroid[e].var_C = velocity);
        if ((slot!(game, Shitroid[e].base.y_pos) as i32 + (velocity >> 8) as i8 as i32 - r20 as i32)
            as i16)
            >= 0
        {
            slot!(game, Shitroid[e].var_C = 0);
            result = result.wrapping_add(1);
        }
    }
    result
}

/// Accelerate the X velocity by `r22` towards `r18`, capped at 1280; 1 once
/// there. Was `Shitroid_AccelerateTowardsX`.
pub fn accelerate_towards_x(k: u16, r18: u16, r22: u16, game: &mut impl Host) -> u16 {
    let e = i32::from(k);
    let mut result: u16 = 0;
    if (slot!(game, Shitroid[e].base.x_pos) as i16).wrapping_sub(r18 as i16) >= 0 {
        let mut velocity = slot!(game, Shitroid[e].var_B).wrapping_sub(r22);
        if velocity as i32 + 1280 & 0x8000 != 0 {
            velocity = -1280i32 as u16;
        }
        slot!(game, Shitroid[e].var_B = velocity);
        if ((slot!(game, Shitroid[e].base.x_pos) as i32 + (velocity >> 8) as i8 as i32 - r18 as i32)
            as i16)
            <= 0
        {
            slot!(game, Shitroid[e].var_B = 0);
            result = result.wrapping_add(1);
        }
    } else {
        let mut velocity = r22.wrapping_add(slot!(game, Shitroid[e].var_B));
        if velocity as i32 - 1280i32 & 0x8000 == 0 {
            velocity = 1280;
        }
        slot!(game, Shitroid[e].var_B = velocity);
        if ((slot!(game, Shitroid[e].base.x_pos) as i32 + (velocity >> 8) as i8 as i32 - r18 as i32)
            as i16)
            >= 0
        {
            slot!(game, Shitroid[e].var_B = 0);
            result = result.wrapping_add(1);
        }
    }
    result
}

/// On animation step 5, cry every fourth time while crying is enabled; the
/// pitch depends on the attack count. Was `Shitroid_HandleCrySoundEffect`.
pub fn cry_sound_effect(k: u16, a: u16, game: &mut impl Host) -> u16 {
    if a == 5 {
        let e = i32::from(k);
        if slot!(game, Shitroid[e].var_04) != 0 {
            let count = slot!(game, Shitroid[e].var_05).wrapping_add(1);
            slot!(game, Shitroid[e].var_05 = count);
            if count >= 4 {
                slot!(game, Shitroid[e].var_05 = 0);
                let effect = if slot!(game, Shitroid[e].var_E) < 0xa {
                    120
                } else {
                    114
                };
                game.queue_sound(1, effect as u8, 6);
            }
        }
        return 5;
    }
    a
}
