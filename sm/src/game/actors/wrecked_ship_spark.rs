//! Wrecked Ship Spark animation, emission and randomized delays.
//! Native actor words and the inherited VRAM-queue alias remain authoritative.
#![forbid(unsafe_code)]
use crate::game::enemy_layouts::EnemyLayout8 as Spark;
use crate::game::enemy_projectiles::{SpawnEprojWithGfx, host::Effects};
use crate::game::gameplay_flag_runtime;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use sm_game::enemy_script::EnemyScript;

fn reload_delay(k: u16, bias: u16, game: &mut Game) {
    let E = k as i32;
    let mut delay = slot!(game, Spark[E].var_E);
    if delay & 0x8000 != 0 {
        delay = (game.next_random() & 0x3f) + 4;
    }
    slot!(game, Spark[E].var_F = bias.wrapping_add(delay));
}
pub(crate) fn initialize(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    let kind = (slot!(game, Spark[E].parameter_1) & 3) as isize;
    slot!(
        game,
        Spark[E].var_B = crate::rtl::rom_table_word!(game, 0xa8e688, kind)
    );
    slot!(game, Spark[E].var_E = slot!(game, Spark[E].parameter_2));
    reload_delay(word!(game.cur_enemy_index), 0, game);
    slot!(game, Spark[E].base; instruction_timer = 1,
        current_instruction = crate::rtl::rom_table_word!(game, 0xa8e682, kind), timer = 0, instruction_timer = 1);
    if gameplay_flag_runtime::boss_byte_at(word!(game.area_index), game) & 1 == 0 {
        // Seventh packed VRAM entry's destination high byte and the next byte.
        slot!(game, Spark[E].base.properties, |old| old
            | u16::from_ne_bytes([
                game.ram().read_byte(0x100),
                game.ram().read_byte(0x101)
            ]));
    }
}
pub(crate) fn advance(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    let state = slot!(game, Spark[E].var_B);
    let k = word!(game.cur_enemy_index);
    let E = k as i32;
    match state {
        0xe694 => return,
        0xe695 | 0xe6b7 | 0xe6dc => (),
        _ => {
            Runtime::new(game).unreachable();
            return;
        }
    }
    if slot!(game, Spark[E].var_F) != 1 {
        slot!(game, Spark[E].var_F, |old| old.wrapping_sub(1));
        return;
    }
    if state == 0xe6dc {
        let target = word!(game.cur_enemy_index);
        SpawnEprojWithGfx(0, word!(game.cur_enemy_index), 0xf498, game);
        reload_delay(target, 0, game);
    } else {
        let active = state == 0xe695;
        slot!(game, Spark[E].var_B = if active { 0xe6b7 } else { 0xe695 });
        slot!(game, Spark[E].base; current_instruction = if active { 0xe5a7 } else { 0xe5e5 }, instruction_timer = 1);
        reload_delay(k, if active { 0 } else { 8 }, game);
    }
}
pub(crate) fn instruction(tangible: bool, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let E = word!(game.cur_enemy_index) as i32;
    slot!(game, Spark[E].base.properties, |old| if tangible {
        old | 0x400
    } else {
        old & !0x400
    });
    script
}
