//! Yellow pipe bug detection, straight flight, curved turns and home reset.
//! Native actor fields retain authority and original query/ROM failure ordering.
#![forbid(unsafe_code)]
use crate::actors::host::{ActorHost, ActorRuntime, Axis};
use crate::actors::layouts::EnemyLayout72 as Bug;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};
use sm_data::enemy_speeds::SpeedTable;

pub fn initialize<H: ActorHost + ActorRuntime + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Bug[e].var_07 = slot!(game, Bug[e].base.x_pos));
    slot!(game, Bug[e].var_08 = slot!(game, Bug[e].base.y_pos));
    slot!(game, Bug[e].base; x_subpos = 0, y_subpos = 0, instruction_timer = 1, timer = 0, current_instruction = 0x8efc);
    if slot!(game, Bug[e].parameter_1) == 0 {
        slot!(game, Bug[e].base.current_instruction = 0x8f24);
    }
    let index = (8 * i32::from(slot!(game, Bug[e].parameter_2)) >> 1) as isize;
    slot!(game, Bug[e]; var_01 = game.enemy_speed(SpeedTable::Linear, index), var_00 = game.enemy_speed(SpeedTable::Linear, index + 1),
        var_03 = game.enemy_speed(SpeedTable::Linear, index + 2), var_02 = game.enemy_speed(SpeedTable::Linear, index + 3), var_A = 0x8fb5, var_06 = 0);
}
pub fn state_address<H: ActorHost + ActorRuntime + ?Sized>(game: &H) -> u32 {
    let e = i32::from(word!(game.cur_enemy_index));
    0xb30000 | u32::from(slot!(game, Bug[e].var_A))
}
fn reset_home<H: ActorHost + ActorRuntime + ?Sized>(e: i32, game: &mut H) {
    slot!(game, Bug[e].base; x_pos = slot!(game, Bug[e].var_07), y_pos = slot!(game, Bug[e].var_08), x_subpos = 0, y_subpos = 0);
    slot!(game, Bug[e]; var_A = 0x8fb5, var_06 = 0);
}
fn horizontal<H: ActorHost + ActorRuntime + ?Sized>(left: bool, k: u16, game: &mut H) {
    let e = i32::from(k);
    let (high, low) = if left {
        (slot!(game, Bug[e].var_03), slot!(game, Bug[e].var_02))
    } else {
        (slot!(game, Bug[e].var_01), slot!(game, Bug[e].var_00))
    };
    slot!(
        game,
        Bug[e].base(x_pos, x_subpos) += (u32::from(high) << 16) | u32::from(low)
    );
}
fn vertical<H: ActorHost + ActorRuntime + ?Sized>(counting_down: bool, k: u16, game: &mut H) {
    let e = i32::from(k);
    if counting_down {
        slot!(game, Bug[e].var_E, |old| old.wrapping_sub(1));
        if slot!(game, Bug[e].var_E) & 0x8000 != 0 {
            slot!(game, Bug[e]; var_E = 0, var_F = 0);
            return;
        }
    } else {
        slot!(game, Bug[e].var_E, |old| old.wrapping_add(1));
    }
    let index = (8 * i32::from(slot!(game, Bug[e].var_E)) >> 1) as isize;
    let offset = if counting_down { 0 } else { 2 };
    let high = game.enemy_speed(SpeedTable::Quadratic, index + offset + 1);
    let low = game.enemy_speed(SpeedTable::Quadratic, index + offset);
    slot!(
        game,
        Bug[e].base(y_pos, y_subpos) += (u32::from(high) << 16) | u32::from(low)
    );
}
pub fn detect_player<H: ActorHost + ActorRuntime + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let left = slot!(game, Bug[e].parameter_1) != 0;
    let actor = word!(game.cur_enemy_index);
    let dx = game.player_delta(actor, Axis::X) as i16;
    let in_x_range = if left {
        dx < 0 && (i32::from(dx) + 192) & 0x8000 == 0
    } else {
        dx >= 0 && (i32::from(dx) - 192) & 0x8000 != 0
    };
    if !in_x_range {
        return;
    }
    let actor = word!(game.cur_enemy_index);
    let in_y_range = game.player_within(actor, Axis::Y, 0x30);
    if in_y_range {
        slot!(game, Bug[e].base.properties, |old| old & !0x100);
        slot!(game, Bug[e]; var_20 = 24, var_A = 0x8ff5);
    }
}
pub fn launch_delay<H: ActorHost + ActorRuntime + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(k);
    let delay = slot!(game, Bug[e].var_20).wrapping_sub(1);
    slot!(game, Bug[e].var_20 = delay);
    if delay == 0 {
        slot!(game, Bug[e].base; instruction_timer = 1, timer = 0, current_instruction = 0x8efc);
        slot!(game, Bug[e].var_A = 0x9028);
        if slot!(game, Bug[e].parameter_1) == 0 {
            slot!(game, Bug[e].base.current_instruction = 0x8f24);
            slot!(game, Bug[e].var_A = 0x90bd);
        }
    }
}
pub fn fly<H: ActorHost + ActorRuntime + ?Sized>(left: bool, game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    horizontal(left, word!(game.cur_enemy_index), game);
    if game.current_outside_screen() {
        reset_home(e, game);
        slot!(game, Bug[e].base.properties, |old| old | 0x100);
    } else if slot!(game, Bug[e].var_06) == 0
        && if left {
            (i32::from(slot!(game, Bug[e].base.x_pos)) - i32::from(word!(game.samus_x_pos)) - 48)
                & 0x8000
                != 0
        } else {
            (i32::from(word!(game.samus_x_pos)) - i32::from(slot!(game, Bug[e].base.x_pos)) - 48)
                & 0x8000
                != 0
        }
    {
        let (state, instruction) = if left {
            (0x915a, 0x8f10)
        } else {
            (0x91d8, 0x8f38)
        };
        slot!(game, Bug[e]; var_A = state, var_C = 0, var_D = 0, var_E = 40, var_F = 1, var_04 = slot!(game, Bug[e].base.x_pos));
        slot!(game, Bug[e].base; instruction_timer = 1, timer = 0, current_instruction = instruction);
    }
}
pub fn turn<H: ActorHost + ActorRuntime + ?Sized>(left: bool, game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let instruction = if left { 0x8efc } else { 0x8f24 };
    if game.current_outside_screen() {
        reset_home(e, game);
        slot!(game, Bug[e].base; instruction_timer = 1, timer = 0, current_instruction = instruction);
        slot!(game, Bug[e].base.properties, |old| old | 0x100);
    } else {
        horizontal(left, word!(game.cur_enemy_index), game);
        let counting_down = slot!(game, Bug[e].var_F) != 0;
        vertical(counting_down, word!(game.cur_enemy_index), game);
        if !counting_down {
            let finished = if left {
                (slot!(game, Bug[e].base.y_pos) as i16)
                    .wrapping_sub(slot!(game, Bug[e].var_08) as i16)
                    < 0
            } else {
                (slot!(game, Bug[e].var_08) as i16)
                    .wrapping_sub(slot!(game, Bug[e].base.y_pos) as i16)
                    >= 0
            };
            if finished {
                slot!(game, Bug[e]; var_06 = 1, var_F = 1);
                // The two legacy turns publish their next state on opposite sides of animation.
                if left {
                    slot!(game, Bug[e].var_A = 0x9028);
                }
                slot!(game, Bug[e].base; instruction_timer = 1, timer = 0, current_instruction = instruction);
                if !left {
                    slot!(game, Bug[e].var_A = 0x90bd);
                }
            }
        }
    }
}
