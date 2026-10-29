//! Phantoon's fireball spawns, speed approach and debug sprite-map toggle.
#![forbid(unsafe_code)]
use crate::game::constants::bank_86::addr_kEproj_DestroyableFireballs;
use crate::game::enemy_layouts::EnemyLayout8 as Phantoon;
use crate::game::enemy_projectiles::SpawnEprojWithGfx;
use crate::game::music::QueueSfx3_Max6;
use crate::game::session_runtime::native_record;
use crate::game_state::Game;
use crate::ram_map::{slot, word};

/// The high byte of the ypos word in the ExtendedSpriteMap at $A7:902D.
const SPRITE_MAP: u16 = 0x902d;
const YPOS_HIGH: usize = 3;
/// Starting fireball slots per pattern, read as bytes from $A7:CFC2.
const PATTERN_STARTS: u16 = 0xcfc2;

fn fireball(parameter: u16, k: u16, game: &mut Game) {
    SpawnEprojWithGfx(parameter, k, addr_kEproj_DestroyableFireballs as u16, game);
}

/// Debug: controller 1's newly pressed 0x4000 flips the cartridge word in place.
pub(crate) fn debug_toggle_sprite_map(k: u16, game: &mut Game) {
    if k == 0 && word!(game.joypad1_newkeys) & 0x4000 != 0 {
        let map = native_record(0xa7, SPRITE_MAP, game);
        map.set_word(YPOS_HIGH, u16::from(map.word(YPOS_HIGH) == 0));
    }
}

/// Move part 0xC0's speed toward the target in `amount`'s high word, or down
/// by its low word while decelerating; 1 once deceleration reaches zero.
pub(crate) fn approach_speed(amount: i32, game: &mut Game) -> u8 {
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

pub(crate) fn spawn_fireball(k: u16, game: &mut Game) {
    fireball(0, k, game);
    QueueSfx3_Max6(0x1d, game);
}

pub(crate) fn spawn_fireball_circle(k: u16, game: &mut Game) {
    for i in (0..=7u16).rev() {
        fireball(i | 0x600, k, game);
    }
    QueueSfx3_Max6(0x28, game);
}

/// Eight fireballs whose slot numbers count up from the pattern's start byte,
/// restarting at zero after a slot of 8 or more.
pub(crate) fn spawn_fireball_pattern(k: u16, pattern: u16, game: &mut Game) {
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
