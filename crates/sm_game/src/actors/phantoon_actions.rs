//! Phantoon's fireball spawns, speed approach and debug sprite-map toggle.
#![forbid(unsafe_code)]
use crate::actors::layouts::EnemyLayout8 as Phantoon;
use crate::cartridge::native_record;
use crate::cartridge::rom_table_word;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

/// What Phantoon's actions need beyond the common actor host.
pub trait Host: crate::actors::host::ActorHost + crate::effects::SoundEffects {
    /// Spawn a destroyable fireball from actor `k` with `parameter`.
    fn spawn_fireball_projectile(&mut self, parameter: u16, k: u16);
}

/// The high byte of the ypos word in the ExtendedSpriteMap at $A7:902D.
const SPRITE_MAP: u16 = 0x902d;
const YPOS_HIGH: usize = 3;
/// Starting fireball slots per pattern, read as bytes from $A7:CFC2.
const PATTERN_STARTS: u16 = 0xcfc2;

fn fireball(parameter: u16, k: u16, game: &mut (impl Host + ?Sized)) {
    game.spawn_fireball_projectile(parameter, k);
}

/// Debug: controller 1's newly pressed 0x4000 flips the cartridge word in place.
pub fn debug_toggle_sprite_map(k: u16, game: &mut (impl Host + ?Sized)) {
    if k == 0 && word!(game.joypad1_newkeys) & 0x4000 != 0 {
        let map = native_record(0xa7, SPRITE_MAP, game);
        map.set_word(YPOS_HIGH, u16::from(map.word(YPOS_HIGH) == 0));
    }
}

/// Move part 0xC0's speed toward the target in `amount`'s high word, or down
/// by its low word while decelerating; 1 once deceleration reaches zero.
pub fn approach_speed(amount: i32, game: &mut (impl Host + ?Sized)) -> u8 {
    let (step, target) = (amount as u16, (amount >> 16) as u16);
    let speed = slot!(game, Phantoon[0xc0].var_D);
    if slot!(game, Phantoon[0xc0].var_E) != 0 {
        slot!(game, Phantoon[0xc0].var_D = speed.wrapping_sub(step));
        if speed < step {
            slot!(game, Phantoon[0xc0].var_D = 0);
            return 1;
        }
    } else {
        let next = step.wrapping_add(speed);
        slot!(game, Phantoon[0xc0].var_D = next);
        if (i32::from(next >> 8) - i32::from(target >> 8)) & 0x8000 == 0 {
            slot!(game, Phantoon[0xc0].var_D = target);
        }
    }
    0
}

pub fn spawn_fireball(k: u16, game: &mut (impl Host + ?Sized)) {
    fireball(0, k, game);
    game.queue_sound(2, 0x1d, 6);
}

pub fn spawn_fireball_circle(k: u16, game: &mut (impl Host + ?Sized)) {
    for i in (0..=7u16).rev() {
        fireball(i | 0x600, k, game);
    }
    game.queue_sound(2, 0x28, 6);
}

/// Eight fireballs whose slot numbers count up from the pattern's start byte,
/// restarting at zero after a slot of 8 or more.
pub fn spawn_fireball_pattern(k: u16, pattern: u16, game: &mut (impl Host + ?Sized)) {
    let mut slot = i16::from(native_record(0xa7, PATTERN_STARTS, game).byte(usize::from(pattern)));
    for row in 1..=8 {
        let current = slot;
        slot += 1;
        fireball((16 * row | current as i32 | 0x400) as u16, k, game);
        if current.wrapping_sub(8) >= 0 {
            slot = 0;
        }
    }
}

/// The colour entry for the current part's health: one step per 312 HP up to
/// seven, offset by the step table at $A7:DC4A from `base`; returns (base, word).
#[allow(non_snake_case)]
pub fn Phantoon_SetColorBasedOnHp_FindEntry(
    base: u16,
    game: &mut (impl Host + ?Sized),
) -> (u16, u16) {
    let (step, mut bound, mut index) = (312u16, 312u16, 0u16);
    while (bound as i16).wrapping_sub(slot!(
        game,
        Phantoon[i32::from(word!(game.cur_enemy_index))].base.health
    ) as i16)
        < 0
    {
        bound = bound.wrapping_add(step);
        index = index.wrapping_add(1);
        if (i32::from(index) - 7) & 0x8000 == 0 {
            break;
        }
    }
    let entry = base.wrapping_add(rom_table_word!(game, 0xa7dc4a, index as isize));
    (base, native_record(0xa7, entry, game).word(0))
}
