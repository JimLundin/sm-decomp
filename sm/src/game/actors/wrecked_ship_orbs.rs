//! Wrecked Ship Orb pursuit with native split-coordinate movement.
//! Actor state and loaded-ROM velocity pairs remain authoritative.
#![forbid(unsafe_code)]
use super::access::ActorKey as Key;
use super::api::{self, Axis};
use crate::game::enemy_layouts::EnemyLayout40 as Orb;
use crate::game::enemy_projectiles::host::Effects;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rtl::words;

pub(crate) fn initialize(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    slot!(game, Orb[E].base; instruction_timer = 1, timer = 0,
        current_instruction = crate::rtl::rom_table_word!(game, 0xa8e380, slot!(game, Orb[E].parameter_1) as isize));
    let rate = 4 * slot!(game, Orb[E].parameter_2) as isize;
    slot!(game, Orb[E]; var_01 = crate::rtl::rom_table_word!(game, 0xa28187, rate),
        var_00 = crate::rtl::rom_table_word!(game, 0xa28187, rate + 1),
        var_03 = crate::rtl::rom_table_word!(game, 0xa28187, rate + 2),
        var_02 = crate::rtl::rom_table_word!(game, 0xa28187, rate + 3));
}
fn drift(horizontal: bool, state: u16, game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    let amount = match (horizontal, state) {
        (false, 0xe405) | (true, 0xe443) => {
            words!((slot!(game, Orb[E].var_03), slot!(game, Orb[E].var_02)))
        }
        (false, 0xe424) | (true, 0xe462) => {
            words!((slot!(game, Orb[E].var_01), slot!(game, Orb[E].var_00)))
        }
        _ => {
            Runtime::new(game).unreachable();
            return;
        }
    };
    if horizontal {
        slot!(game, Orb[E].base(x_pos, x_subpos) += amount);
    } else {
        slot!(game, Orb[E].base(y_pos, y_subpos) += amount);
    }
}
pub(crate) fn advance(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    slot!(game, Orb[E].var_A = 0xe405);
    let key = Key::from_legacy(word!(game.cur_enemy_index));
    if api::player_delta(&Runtime::new(game), key, Axis::Y) & 0x8000 == 0 {
        slot!(game, Orb[E].var_A = 0xe424);
    }
    let E = word!(game.cur_enemy_index) as i32;
    slot!(game, Orb[E].var_B = 0xe443);
    let key = Key::from_legacy(word!(game.cur_enemy_index));
    if api::player_delta(&Runtime::new(game), key, Axis::X) & 0x8000 == 0 {
        slot!(game, Orb[E].var_B = 0xe462);
    }
    let E = word!(game.cur_enemy_index) as i32;
    drift(false, slot!(game, Orb[E].var_A), game);
    drift(true, slot!(game, Orb[E].var_B), game);
}
