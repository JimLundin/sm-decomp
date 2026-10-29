//! Bomb and Golden Torizo: walking, the low-health drool and Golden Torizo's
//! shot reactions.
#![forbid(unsafe_code)]
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::{self, api as actor_api};
use crate::game::constants::bank_86::addr_kEproj_BombTorizoLowHealthInitialDrool;
use crate::game::constants::enemy_properties::kEnemyProps_Tangible;
use crate::game::enemy_layouts::EnemyLayout40 as Torizo;
use crate::game::enemy_projectiles::SpawnEprojWithGfx;
use crate::game::enemy_script::EnemyScript;
use crate::game::session_runtime::Runtime;
use crate::game::sm_aa::{
    addr_kTorizo_Ilist_B1C8, addr_kTorizo_Ilist_CDE1, addr_kTorizo_Ilist_CE43,
    addr_kTorizo_Ilist_CEA5, addr_kTorizo_Ilist_CEFF, addr_kTorizo_Ilist_D1F1,
    addr_kTorizo_Ilist_D2AD,
};
use crate::game_state::Game;
use crate::ram_map::{slot, word};

/// Walking speeds by step index, shared by the normal and faceless walks.
/// Was `g_word_AAC4BD`.
const WALK_SPEEDS: [i16; 20] = [
    -5, 0, -5, -19, -16, -7, 0, -7, -17, -18, 5, 0, 5, 19, 16, 7, 0, 7, 17, 18,
];

/// One walking step at the speed for the step index in `jp`. A wall jumps to
/// the facing's blocked instruction list; otherwise the Torizo snaps to slopes
/// and starts `pause` when it faces away from Samus. Was `torizo_stride`.
pub(crate) fn walk(
    k: u16,
    jp: EnemyScript,
    blocked_left: u16,
    blocked_right: u16,
    pause: u16,
    game: &mut Game,
) -> EnemyScript {
    let e = i32::from(k);
    slot!(
        game,
        Torizo[e].var_A = WALK_SPEEDS[(jp.word(0) >> 1) as usize] as u16
    );
    let amount = (slot!(game, Torizo[e].var_A) as i16 as i32) << 16;
    let blocked = actor_api::move_horizontal(
        &mut Runtime::new(game),
        Key::from_legacy(k),
        amount,
        actor_api::SlopePolicy::Ignore,
    ) == actor_api::MovementResult::Blocked;
    if blocked {
        slot!(game, Torizo[e].var_03 = 0);
        if slot!(game, Torizo[e].parameter_1) & 0x8000 != 0 {
            EnemyScript::jump(blocked_left)
        } else {
            EnemyScript::jump(blocked_right)
        }
    } else {
        actor_api::snap_to_slopes(&mut Runtime::new(game), Key::from_legacy(k));
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
pub(crate) fn drool_at_low_health(k: u16, game: &mut Game) {
    if word!(game.area_index) == 0 && word!(game.random_number) & 0x8142 == 0 {
        let health: u16 = slot!(game, Torizo[k as i32].base.health);
        if health != 0 && health < 0x15e {
            SpawnEprojWithGfx(
                health,
                k,
                addr_kEproj_BombTorizoLowHealthInitialDrool as u16,
                game,
            );
        }
    }
}

/// The facing word XOR the Torizo's offset from Samus: bit 15 is clear when it
/// faces her. Was `Torizo_Func_12`.
pub(crate) fn facing_against_samus(k: u16, game: &mut Game) -> u16 {
    let e = i32::from(k);
    (slot!(game, Torizo[e].parameter_1) as i32
        ^ slot!(game, Torizo[e].base.x_pos) as i32 - word!(game.samus_x_pos) as i32) as u16
}

/// Golden Torizo shot reaction, damaged: on death, start the death
/// instruction list. Was `Torizo_D6A6`.
pub(crate) fn golden_damaged_shot_reaction(game: &mut Game) {
    actors::combat::shot_reaction(
        &mut Runtime::new(game),
        actors::api::ShotHandling::ReactionOnly,
    );
    let e = word!(game.cur_enemy_index) as i32;
    if slot!(game, Torizo[e].base.health) == 0 {
        slot!(
            game,
            Torizo[e].base.current_instruction = addr_kTorizo_Ilist_B1C8 as u16
        );
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

/// Golden Torizo shot reaction, missile: catch it. Was `Torizo_D6D1`.
pub(crate) fn golden_missile_reaction(k: u16, j: u16, game: &mut Game) {
    word!(game.projectile_dir[(j >> 1)] &= !(0x10));
    let e = i32::from(k);
    slot!(game, Torizo[e].var_E = 0xaac6abi32 as u16);
    slot!(game, Torizo[e].base.instruction_timer = 1);
    let list = if slot!(game, Torizo[e].parameter_1) & 0x8000 != 0 {
        addr_kTorizo_Ilist_D2AD as u16
    } else {
        addr_kTorizo_Ilist_D1F1 as u16
    };
    slot!(game, Torizo[e].base.current_instruction = list);
}

/// Golden Torizo shot reaction, super missile: catch it when facing Samus,
/// otherwise take damage. Was `Torizo_D6F7`.
pub(crate) fn golden_super_missile_reaction(k: u16, j: u16, game: &mut Game) {
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
            (true, true) => addr_kTorizo_Ilist_CEFF,
            (true, false) => addr_kTorizo_Ilist_CE43,
            (false, true) => addr_kTorizo_Ilist_CEA5,
            (false, false) => addr_kTorizo_Ilist_CDE1,
        } as u16;
        slot!(game, Torizo[e].base.current_instruction = list);
    } else {
        golden_damaged_shot_reaction(game);
    }
}
