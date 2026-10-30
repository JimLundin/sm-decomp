//! Phantoon, the Wrecked Ship's ghost: its flames, wave amplitude, health
//! palette and leftover debug toggle (bank $A7:CF0C-$A7:DC49).
//!
//! Phantoon is four enemies: the body in slot 0, then the eye, tentacles and
//! mouth. The mouth's slot holds the wavy fade's amplitude.
use crate::actors::boss_lease::BossState;
use crate::actors::layouts::{EnemyBase, EnemyLayout8 as Part, same_words};
use crate::actors::session_slots::session_slot;
use crate::cartridge::native_record;
use crate::cartridge::rom_table_word;
use crate::ram::word;

/// What Phantoon's actions need beyond the common actor host.
pub trait Host: crate::actors::host::ActorHost + crate::effects::SoundEffects {}
impl<T: crate::actors::host::ActorHost + crate::effects::SoundEffects + ?Sized> Host for T {}

/// The Phantoon destroyable flames enemy projectile ($86:9C29).
pub const DESTROYABLE_FLAMES: u16 = 0x9c29;

/// The mouth's view of its slot.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Mouth {
    pub base: EnemyBase,
    pub casual_flame_pattern: u16,
    pub casual_flame_timer: u16,
    pub casual_flame_pattern_index: u16,
    pub wave_amplitude: u16,
    pub wave_shrinking: u16,
    pad_0fb2: [u16; 3],
}

same_words!(Mouth: EnemyLayout8 {
    casual_flame_pattern = var_a, casual_flame_timer = var_b, casual_flame_pattern_index = var_c,
    wave_amplitude = var_d, wave_shrinking = var_e,
});

const BANK: u8 = 0xa7;
/// The mouth's slot.
pub const MOUTH: i32 = 0xc0;

/// Controller 1's Y button.
const BUTTON_Y: u16 = 0x4000;
/// $A7:9030, a word the debug toggle flips and nothing reads: the high byte of
/// the Y position in the extended spritemap at $A7:902D.
const UNREAD_WORD: u16 = 0x902d;
const UNREAD_WORD_OFFSET: usize = 3;

/// Flame projectile parameters: the spiral's 600h, flame rain's 400h, with the
/// rain's fall time in bits 4-7.
const SPIRAL: u16 = 0x600;
const RAIN: i32 = 0x400;
const SPIRAL_FLAMES: u16 = 8;
const RAIN_FLAMES: i32 = 8;
const RAIN_POSITIONS: i16 = 9;
/// $A7:CFC2: the position of each flame rain pattern's first flame, a byte each.
const RAIN_FIRST_POSITIONS: u16 = 0xcfc2;
/// Phantoon's flame (1Dh) and materialises attack (28h), library 3.
const FLAME_SOUND: u8 = 0x1d;
const MATERIALISE_SOUND: u8 = 0x28;

/// The health palette steps every 312 health ($9C4 / 8), up to step 7.
const HEALTH_STEP: u16 = 312;
const HEALTH_STEPS: i32 = 7;
/// $A7:DC4A: the palette of each health step.
const HEALTH_PALETTES: u32 = 0xa7_dc4a;
const SIGN: i32 = 0x8000;

/// Broken nothingness ($A7:CF0C): newly pressing Y toggles a ROM word nothing
/// reads; the original forgot its $7E bank. Was `debug_toggle_sprite_map`.
pub fn broken_nothingness(k: u16, game: &mut (impl BossState + ?Sized)) {
    if k == 0 && word!(game.JOYPAD1_NEWKEYS) & BUTTON_Y != 0 {
        let map = native_record(BANK, UNREAD_WORD, game);
        map.set_word(
            UNREAD_WORD_OFFSET,
            u16::from(map.word(UNREAD_WORD_OFFSET) == 0),
        );
    }
}

/// Grow/shrink Phantoon wave amplitude ($A7:CF27) by the delta in `amount`'s
/// low word, growing up to the maximum in its high word; 1 once shrinking has
/// taken it below zero. Was `approach_speed`.
pub fn grow_or_shrink_wave_amplitude(amount: i32, game: &mut (impl BossState + ?Sized)) -> u8 {
    let (delta, max) = (amount as u16, (amount >> 16) as u16);
    let amplitude = session_slot!(game, Mouth[MOUTH].wave_amplitude);
    if session_slot!(game, Mouth[MOUTH].wave_shrinking) != 0 {
        session_slot!(
            game,
            Mouth[MOUTH].wave_amplitude = amplitude.wrapping_sub(delta)
        );
        if amplitude < delta {
            session_slot!(game, Mouth[MOUTH].wave_amplitude = 0);
            return 1;
        }
    } else {
        let next = delta.wrapping_add(amplitude);
        session_slot!(game, Mouth[MOUTH].wave_amplitude = next);
        if (i32::from(next >> 8) - i32::from(max >> 8)) & SIGN == 0 {
            session_slot!(game, Mouth[MOUTH].wave_amplitude = max);
        }
    }
    0
}

/// Spawn casual flame ($A7:CF5E). Was `spawn_fireball`.
pub fn spawn_casual_flame(k: u16, game: &mut (impl BossState + ?Sized)) {
    game.spawn_actor_eproj(0, k, DESTROYABLE_FLAMES);
    game.queue_sound(2, FLAME_SOUND, 6);
}

/// Phantoon materialisation flame spiral ($A7:CF70). Was `spawn_fireball_circle`.
pub fn spawn_flame_spiral(k: u16, game: &mut (impl BossState + ?Sized)) {
    for i in (0..SPIRAL_FLAMES).rev() {
        game.spawn_actor_eproj(i | SPIRAL, k, DESTROYABLE_FLAMES);
    }
    game.queue_sound(2, MATERIALISE_SOUND, 6);
}

/// Spawn flame rain enemy projectiles ($A7:CF8B): eight flames at consecutive
/// positions of nine from the pattern's first, leaving one gap, each falling
/// for longer. Was `spawn_fireball_pattern`.
pub fn spawn_flame_rain(k: u16, pattern: u16, game: &mut (impl BossState + ?Sized)) {
    let mut position =
        i16::from(native_record(BANK, RAIN_FIRST_POSITIONS, game).byte(usize::from(pattern)));
    for flame in 1..=RAIN_FLAMES {
        let current = position;
        position += 1;
        game.spawn_actor_eproj(
            ((16 * flame) | current as i32 | RAIN) as u16,
            k,
            DESTROYABLE_FLAMES,
        );
        if current.wrapping_sub(RAIN_POSITIONS - 1) >= 0 {
            position = 0;
        }
    }
}

/// Get Phantoon health-based palette colour ($A7:DC0F): the colour at `base`
/// in the palette for the current part's health step; returns (base, colour).
/// Was `Phantoon_SetColorBasedOnHp_FindEntry`.
pub fn health_palette_colour(base: u16, game: &mut (impl BossState + ?Sized)) -> (u16, u16) {
    let (mut bound, mut step) = (HEALTH_STEP, 0u16);
    while (bound as i16).wrapping_sub(session_slot!(
        game,
        Part[i32::from(word!(game.CUR_ENEMY_INDEX))].base.health
    ) as i16)
        < 0
    {
        bound = bound.wrapping_add(HEALTH_STEP);
        step = step.wrapping_add(1);
        if (i32::from(step) - HEALTH_STEPS) & SIGN == 0 {
            break;
        }
    }
    let entry = base.wrapping_add(rom_table_word!(game, HEALTH_PALETTES, step as isize));
    (base, native_record(BANK, entry, game).word(0))
}
