//! Bomb and Golden Torizo: walking, the low-health drool and Golden Torizo's
//! shot reactions.
#![forbid(unsafe_code)]
use crate::actors::layouts::EnemyLayout40 as Torizo;
use crate::constants::enemy_properties::kEnemyProps_Tangible;
use crate::enemy_script::EnemyScript;
use crate::ram::{CompatibilityRam, RawRam as _, WorkRamAccess, slot, word};

/// What the Torizos need beyond work RAM: actor movement, projectiles and the
/// shared shot reaction, which the compatibility runtime still implements.
pub trait Host: WorkRamAccess + CompatibilityRam {
    /// Move actor `k` by `amount` (16.16) horizontally, ignoring slopes; true when blocked.
    fn move_horizontal_ignoring_slopes(&mut self, k: u16, amount: i32) -> bool;
    fn snap_to_slopes(&mut self, k: u16);
    /// Spawn projectile `definition` from actor `k` with `parameter`.
    fn spawn_projectile(&mut self, parameter: u16, k: u16, definition: u16);
    /// The common shot reaction, without death handling.
    fn shot_reaction_only(&mut self);
}

/// Bank $AA instruction lists and projectile definitions the routines select.
pub const DEATH: u16 = 0xb1c8;
/// Caught super missile, by facing and leg forward.
pub const CAUGHT_SUPER_MISSILE_LEFT_LEFT_LEG: u16 = 0xcde1;
pub const CAUGHT_SUPER_MISSILE_LEFT_RIGHT_LEG: u16 = 0xce43;
pub const CAUGHT_SUPER_MISSILE_RIGHT_RIGHT_LEG: u16 = 0xcea5;
pub const CAUGHT_SUPER_MISSILE_RIGHT_LEFT_LEG: u16 = 0xceff;
/// Golden Torizo's dodge.
pub const DODGE_TURNING_LEFT: u16 = 0xd1f1;
pub const DODGE_TURNING_RIGHT: u16 = 0xd2ad;
/// Bank $86: the Bomb Torizo's low-health initial drool.
pub const LOW_HEALTH_DROOL: u16 = 0xa969;

/// Walking speeds by step index, shared by the normal and faceless walks.
/// Was `g_word_AAC4BD`.
const WALK_SPEEDS: [i16; 20] = [
    -5, 0, -5, -19, -16, -7, 0, -7, -17, -18, 5, 0, 5, 19, 16, 7, 0, 7, 17, 18,
];

/// One walking step at the speed for the step index in `jp`. A wall jumps to
/// the facing's blocked instruction list; otherwise the Torizo snaps to slopes
/// and starts `pause` when it faces away from Samus. Was `torizo_stride`.
pub fn walk(
    k: u16,
    jp: EnemyScript,
    blocked_left: u16,
    blocked_right: u16,
    pause: u16,
    game: &mut impl Host,
) -> EnemyScript {
    let e = i32::from(k);
    slot!(
        game,
        Torizo[e].var_A = WALK_SPEEDS[(jp.word(0) >> 1) as usize] as u16
    );
    let amount = (slot!(game, Torizo[e].var_A) as i16 as i32) << 16;
    let blocked = game.move_horizontal_ignoring_slopes(k, amount);
    if blocked {
        slot!(game, Torizo[e].var_03 = 0);
        if slot!(game, Torizo[e].parameter_1) & 0x8000 != 0 {
            EnemyScript::jump(blocked_left)
        } else {
            EnemyScript::jump(blocked_right)
        }
    } else {
        game.snap_to_slopes(k);
        if (slot!(game, Torizo[e].parameter_1) as i32
            ^ word!(game.samus_x_pos) as i32 - slot!(game, Torizo[e].base.x_pos) as i32)
            & 0x8000
            == 0
            && slot!(game, Torizo[e].var_03) == 0
        {
            slot!(game, Torizo[e].var_03 = pause);
        }
        jp.offset(1)
    }
}

/// In Crateria, now and then drool while health is low. Was `Torizo_C620`.
pub fn drool_at_low_health(k: u16, game: &mut impl Host) {
    if word!(game.area_index) == 0 && word!(game.random_number) & 0x8142 == 0 {
        let health: u16 = slot!(game, Torizo[k as i32].base.health);
        if health != 0 && health < 0x15e {
            game.spawn_projectile(health, k, LOW_HEALTH_DROOL);
        }
    }
}

/// The facing word XOR the Torizo's offset from Samus: bit 15 is clear when it
/// faces her. Was `Torizo_Func_12`.
pub fn facing_against_samus(k: u16, game: &mut impl Host) -> u16 {
    let e = i32::from(k);
    (slot!(game, Torizo[e].parameter_1) as i32
        ^ slot!(game, Torizo[e].base.x_pos) as i32 - word!(game.samus_x_pos) as i32) as u16
}

/// Golden Torizo shot reaction, damaged: on death, start the death
/// instruction list. Was `Torizo_D6A6`.
pub fn golden_damaged_shot_reaction(game: &mut impl Host) {
    game.shot_reaction_only();
    let e = word!(game.cur_enemy_index) as i32;
    if slot!(game, Torizo[e].base.health) == 0 {
        slot!(game, Torizo[e].base.current_instruction = DEATH);
        slot!(game, Torizo[e].base.instruction_timer = 1);
        slot!(
            game,
            Torizo[e].parameter_2 = slot!(game, Torizo[e].parameter_2) | 0xc000
        );
        slot!(
            game,
            Torizo[e].base.properties =
                slot!(game, Torizo[e].base.properties) | kEnemyProps_Tangible as u16
        );
    }
}

/// Golden Torizo shot reaction, missile: dodge it. Was `Torizo_D6D1`.
pub fn golden_missile_reaction(k: u16, j: u16, game: &mut impl Host) {
    word!(game.projectile_dir[(j >> 1)] &= !(0x10));
    let e = i32::from(k);
    slot!(game, Torizo[e].var_E = 0xaac6abi32 as u16);
    slot!(game, Torizo[e].base.instruction_timer = 1);
    let list = if slot!(game, Torizo[e].parameter_1) & 0x8000 != 0 {
        DODGE_TURNING_RIGHT
    } else {
        DODGE_TURNING_LEFT
    };
    slot!(game, Torizo[e].base.current_instruction = list);
}

/// Golden Torizo shot reaction, super missile: catch it when facing Samus,
/// otherwise take damage. Was `Torizo_D6F7`.
pub fn golden_super_missile_reaction(k: u16, j: u16, game: &mut impl Host) {
    let e = word!(game.cur_enemy_index) as i32;
    if facing_against_samus(k, game) & 0x8000 == 0 {
        slot!(
            game,
            Torizo[e].parameter_2 = slot!(game, Torizo[e].parameter_2) | 0x1000
        );
        slot!(game, Torizo[e].var_E = 0xaac6abi32 as u16);
        word!(game.projectile_dir[(j >> 1)] |= 0x10u16);
        slot!(game, Torizo[e].base.instruction_timer = 1);
        let parameter = slot!(game, Torizo[e].parameter_1);
        let list = match (parameter & 0x2000 != 0, parameter & 0x8000 != 0) {
            (true, true) => CAUGHT_SUPER_MISSILE_RIGHT_LEFT_LEG,
            (true, false) => CAUGHT_SUPER_MISSILE_LEFT_RIGHT_LEG,
            (false, true) => CAUGHT_SUPER_MISSILE_RIGHT_RIGHT_LEG,
            (false, false) => CAUGHT_SUPER_MISSILE_LEFT_LEFT_LEG,
        };
        slot!(game, Torizo[e].base.current_instruction = list);
    } else {
        golden_damaged_shot_reaction(game);
    }
}
