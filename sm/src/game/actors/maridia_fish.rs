//! Maridia Fish share their two swim legs and retain live collision callbacks.
#![forbid(unsafe_code)]
use crate::game::enemy_layouts::EnemyLayout40 as Fish;
use crate::game::enemy_speeds::enemy_speed;
use crate::game::session_runtime::{Runtime, actor_sine};
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use sm_game::enemy_script::EnemyScript;
use {crate::game::actors::api, sm_game::actors::access::ActorKey as Key};

fn wave_sample(key: i32, game: &mut Game) -> u16 {
    actor_sine(
        slot!(game, Fish[key].var_F),
        slot!(game, Fish[key].var_00),
        game,
    )
}
pub(crate) fn initialize(game: &mut Game) {
    let key = i32::from(word!(game.cur_enemy_index));
    slot!(game, Fish[key].base.current_instruction = 0x902a);
    slot!(game, Fish[key].var_A = 0x9132);
    if slot!(byte game, Fish[key].parameter_1[1]) == 0 {
        slot!(game, Fish[key].base.current_instruction = 0x9060);
        slot!(game, Fish[key].var_A = 0x91ab);
    }
    let index = 4 * isize::from(slot!(byte game, Fish[key].parameter_1[0]));
    let value = enemy_speed!(game, Linear, index);
    slot!(game, Fish[key].var_C = value);
    let value = enemy_speed!(game, Linear, index + 1);
    slot!(game, Fish[key].var_B = value);
    let value = enemy_speed!(game, Linear, index + 2);
    slot!(game, Fish[key].var_E = value);
    let value = enemy_speed!(game, Linear, index + 3);
    slot!(game, Fish[key].var_D = value);
    let value = u16::from(slot!(byte game, Fish[key].parameter_2[0]));
    slot!(game, Fish[key].var_00 = value);
    let value = u16::from(slot!(byte game, Fish[key].parameter_2[1]));
    slot!(game, Fish[key].var_02 = value);
    slot!(game, Fish[key].var_F = 0);
    slot!(game, Fish[key].var_01 = 0);
    let sine = wave_sample(key, game);
    slot!(game, Fish[key].var_03 = sine);
}
fn select_instruction(key: i32, instruction: u16, game: &mut Game) {
    slot!(game, Fish[key].base.instruction_timer = 1);
    slot!(game, Fish[key].base.timer = 0);
    slot!(game, Fish[key].base.current_instruction = instruction);
}
fn turn(key: i32, first_leg: bool, game: &mut Game) {
    let state = if first_leg { 0x9224 } else { 0x9256 };
    slot!(game, Fish[key].var_A = state);
    select_instruction(key, if first_leg { 0x903c } else { 0x9072 }, game);
}
fn swim(key: i32, first_leg: bool, game: &mut Game) {
    let actor = Key::from_legacy(word!(game.cur_enemy_index));
    let amount = if first_leg {
        (u32::from(slot!(game, Fish[key].var_E)) << 16) | u32::from(slot!(game, Fish[key].var_D))
    } else {
        (u32::from(slot!(game, Fish[key].var_C)) << 16) | u32::from(slot!(game, Fish[key].var_B))
    } as i32;
    if api::move_horizontal(
        &mut Runtime::new(game),
        actor,
        amount,
        api::SlopePolicy::Ignore,
    ) == api::MovementResult::Blocked
    {
        turn(key, first_leg, game);
    } else {
        let sine = wave_sample(key, game);
        slot!(game, Fish[key].var_04 = sine);
        let actor = Key::from_legacy(word!(game.cur_enemy_index));
        let amount =
            i32::from((sine as i16).wrapping_sub(slot!(game, Fish[key].var_03) as i16)) << 16;
        if api::move_vertical(&mut Runtime::new(game), actor, amount)
            == api::MovementResult::Blocked
        {
            turn(key, first_leg, game);
        } else {
            let phase = slot!(byte game, Fish[key].var_02[0])
                .wrapping_add(slot!(game, Fish[key].var_F) as u8);
            slot!(game, Fish[key].var_F = u16::from(phase));
        }
    }
    slot!(game, Fish[key].var_03 = slot!(game, Fish[key].var_04));
}
fn resume(key: i32, first_leg: bool, game: &mut Game) {
    if slot!(game, Fish[key].var_01) != 0 {
        slot!(game, Fish[key].var_01 = 0);
        let state = if first_leg { 0x91ab } else { 0x9132 };
        slot!(game, Fish[key].var_A = state);
        slot!(game, Fish[key].var_02, |old| old.wrapping_neg());
        select_instruction(key, if first_leg { 0x9060 } else { 0x902a }, game);
    }
}
pub(crate) fn advance(game: &mut Game) {
    let key = i32::from(word!(game.cur_enemy_index));
    match slot!(game, Fish[key].var_A) {
        0x9132 => swim(key, true, game),
        0x91ab => swim(key, false, game),
        0x9224 => resume(key, true, game),
        0x9256 => resume(key, false, game),
        _ => {
            crate::rtl::Unreachable(game);
        }
    }
}
pub(crate) fn instruction(ea: u32, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let key = i32::from(word!(game.cur_enemy_index));
    match ea {
        10719382 => slot!(game, Fish[key].base.layer = 6),
        10719392 => slot!(game, Fish[key].base.layer = 2),
        10719402 => slot!(game, Fish[key].var_01 = 1),
        _ => unreachable!("unrecognized Maridia Fish instruction"),
    }
    script
}
