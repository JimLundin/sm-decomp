//! Morph Ball Eye proximity, opening, tracking and closing.
//! Native actor words remain authoritative; the runtime owns beam hardware startup.
#![forbid(unsafe_code)]
use super::access::ActorKey as Key;
use super::api::{self, Axis};
use crate::game::enemy_layouts::EnemyLayout8 as Eye;
use crate::game::music::QueueSfx2_Max6;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::{slot, word};

fn animate(E: i32, clear: u16, set: u16, game: &mut Game) {
    let instruction = if slot!(game, Eye[E].parameter_1) & 1 == 0 {
        clear
    } else {
        set
    };
    slot!(game, Eye[E].base.current_instruction = instruction);
}
fn in_range(k: u16, horizontal: u16, game: &mut Game) -> bool {
    let key = Key::from_legacy(k);
    api::player_within(&mut Runtime::new(game), key, Axis::Y, 128)
        && api::player_within(&mut Runtime::new(game), key, Axis::X, horizontal)
}
fn aim(E: i32, game: &mut Game) -> u16 {
    let angle = api::angle_from_delta(
        word!(game.samus_x_pos).wrapping_sub(slot!(game, Eye[E].base.x_pos)),
        word!(game.samus_y_pos).wrapping_sub(slot!(game, Eye[E].base.y_pos)),
    );
    slot!(game, Eye[E].var_D = angle);
    angle
}
pub(crate) fn initialize(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    slot!(game, Eye[E].base.properties, |old| old | 0x2000);
    slot!(game, Eye[E].base; spritemap_pointer = 0x804d, instruction_timer = 1, timer = 0);
    if slot!(game, Eye[E].parameter_2) & 0x8000 != 0 {
        let index = (slot!(game, Eye[E].parameter_2) & 15) as usize;
        slot!(game, Eye[E].base.x_pos, |old| old
            .wrapping_add([-8i16, 8, 0, 0][index] as u16));
        slot!(game, Eye[E].base.y_pos, |old| old
            .wrapping_add([0i16, 0, -8, 8][index] as u16));
        slot!(game, Eye[E].var_F = 0x91dc);
        slot!(
            game,
            Eye[E].base.current_instruction =
                crate::rtl::rom_table_word!(game, 0xa890da, index as isize)
        );
        word_transfers!(game, descending, 256; native(0x92fe) <- value(255));
    } else {
        slot!(game, Eye[E].var_F = 0x90f1);
        animate(E, 0x8ffc, 0x900e, game);
    }
}
pub(crate) fn state_address(game: &mut Game) -> Option<u32> {
    if word!(game.collected_items) & 4 == 0 {
        return None;
    }
    let E = word!(game.cur_enemy_index) as i32;
    Some(0xa80000 | u32::from(slot!(game, Eye[E].var_F)))
}
/// Returns a beam request after its sound, before hardware startup and continuation.
pub(crate) fn prepare_update(state: u16, k: u16, game: &mut Game) -> bool {
    let E = k as i32;
    match state {
        0x90f1 => {
            if in_range(k, 128, game) {
                slot!(game, Eye[E].var_E = 32);
                slot!(game, Eye[E].base.instruction_timer = 1);
                animate(E, 0x9014, 0x9026, game);
                slot!(game, Eye[E].var_F = 0x912e);
            }
        }
        0x9160 => {
            if in_range(k, 176, game) {
                let angle = aim(E, game);
                slot!(
                    game,
                    Eye[E].base.current_instruction = (((angle as i32 & 0xf0) >> 2) - 28756) as u16
                );
            } else {
                QueueSfx2_Max6(0x71, game);
                slot!(game, Eye[E]; var_C = 0, var_E = 32);
                animate(E, 0x8ff0, 0x9002, game);
                slot!(game, Eye[E].var_F = 0x91ce);
            }
            slot!(game, Eye[E].base.instruction_timer = 1);
        }
        0x912e | 0x91ce => {
            let expired = slot!(game, Eye[E].var_E) == 1;
            slot!(game, Eye[E].var_E, |old| old.wrapping_sub(1));
            let negative = slot!(game, Eye[E].var_E) & 0x8000 != 0;
            if expired || negative {
                if state == 0x912e {
                    QueueSfx2_Max6(0x17, game);
                    return true;
                }
                slot!(game, Eye[E].var_F = 0x90f1);
            }
        }
        _ => unreachable!("Morph Ball Eye state {state:04x}"),
    }
    false
}
pub(crate) fn finish_beam(k: u16, game: &mut Game) {
    let E = k as i32;
    slot!(game, Eye[E].var_F = 0x9160);
    aim(E, game);
}
