//! Norfair Pipe Bug's ordered five-part emergence and flight.
//! Native actor words remain authoritative across the existing query boundaries.
#![forbid(unsafe_code)]
use crate::actors::host::{ActorHost, ActorRuntime, Axis};
use crate::actors::layouts::EnemyLayout72 as Bug;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};
use sm_data::enemy_speeds::SpeedTable;

pub fn initialize<H: ActorHost + ActorRuntime + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Bug[e].var_D = slot!(game, Bug[e].base.x_pos));
    slot!(game, Bug[e].var_E = slot!(game, Bug[e].base.y_pos));
    slot!(game, Bug[e].base.properties, |old| old | 0x100);
    slot!(
        game,
        Bug[e].var_B = 8u16.wrapping_mul(u16::from(slot!(byte game, Bug[e].parameter_2[1])))
    );
    slot!(game, Bug[e].var_C = 0);
    slot!(game, Bug[e].base; instruction_timer = 1, timer = 0, current_instruction = 0x8ae1);
    slot!(game, Bug[e].var_A = 0x8bcd);
}
pub fn state_address<H: ActorHost + ActorRuntime + ?Sized>(game: &H) -> u32 {
    let e = i32::from(word!(game.cur_enemy_index));
    0xb30000 | u32::from(slot!(game, Bug[e].var_A))
}
pub fn reset_offscreen<H: ActorHost + ActorRuntime + ?Sized>(game: &mut H) {
    if game.current_outside_screen() {
        let e = i32::from(word!(game.cur_enemy_index));
        slot!(game, Bug[e].base.properties, |old| old | 0x100);
        slot!(game, Bug[e].var_A = 0x8bcd);
        slot!(game, Bug[e].base; x_pos = slot!(game, Bug[e].var_D), y_pos = slot!(game, Bug[e].var_E));
    }
}
fn speed<H: ActorHost + ActorRuntime + ?Sized>(index: isize, game: &H) -> u32 {
    let high = game.enemy_speed(SpeedTable::Linear, index);
    let low = game.enemy_speed(SpeedTable::Linear, index + 1);
    (u32::from(high) << 16) | u32::from(low)
}
fn facing_animation<H: ActorHost + ActorRuntime + ?Sized>(actor: u16, game: &mut H) -> u16 {
    if game.player_delta(actor, Axis::X) & 0x8000 == 0 {
        0x8b21
    } else {
        0x8ae1
    }
}
fn plan_group<H: ActorHost + ActorRuntime + ?Sized>(k: i32, game: &mut H) {
    // Finish each field's five stores before starting the next field.
    for offset in [0, 64, 128, 192, 256] {
        slot!(game, Bug[k + offset].var_A = 0x8ca6);
    }
    for (part, delay) in [104, 96, 88, 112, 120].into_iter().enumerate() {
        slot!(game, Bug[k + 64 * part as i32].var_02 = delay);
    }
    for (part, state) in [0x8cff, 0x8d0c, 0x8d4e, 0x8d90, 0x8dd2]
        .into_iter()
        .enumerate()
    {
        slot!(game, Bug[k + 64 * part as i32].var_F = state);
    }
}
pub fn await_group<H: ActorHost + ActorRuntime + ?Sized>(game: &mut H) {
    let current = i32::from(word!(game.cur_enemy_index));
    if slot!(byte game, Bug[current].parameter_2[0]) != 0
        && [64, 128, 192, 256].into_iter().all(|offset| {
            let e = i32::from(word!(game.cur_enemy_index)) + offset;
            slot!(game, Bug[e].var_A) == 0x8bcd
        })
    {
        let e = i32::from(word!(game.cur_enemy_index));
        slot!(game, Bug[e].var_A = 0x8bff);
    }
}
pub fn await_player<H: ActorHost + ActorRuntime + ?Sized>(game: &mut H) {
    let k = word!(game.cur_enemy_index);
    let e = i32::from(word!(game.cur_enemy_index));
    let actor = word!(game.cur_enemy_index);
    let distance = u16::from(slot!(byte game, Bug[e].parameter_2[0]));
    let in_range = game.player_within(actor, Axis::X, distance);
    if in_range && (slot!(game, Bug[e].base.y_pos) as i16).wrapping_sub(game.player_y() as i16) >= 0
    {
        slot!(game, Bug[e].var_C, |old| old.wrapping_add(1));
        slot!(game, Bug[e].base; instruction_timer = 1, timer = 0);
        let instruction = facing_animation(k, game);
        for offset in [0, 64, 128, 192, 256] {
            slot!(
                game,
                Bug[i32::from(k) + offset].base.current_instruction = instruction
            );
        }
        plan_group(i32::from(k), game);
    }
}
pub fn rise<H: ActorHost + ActorRuntime + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Bug[e].base.properties, |old| old & !0x100);
    slot!(game, Bug[e].base(y_pos, y_subpos) += speed(66, game));
    if (slot!(game, Bug[e].base.y_pos) as i16).wrapping_sub(game.player_y() as i16) < 0 {
        slot!(game, Bug[e].var_A = slot!(game, Bug[e].var_F));
        slot!(game, Bug[e].var_01 = slot!(game, Bug[e].base.y_pos));
        slot!(game, Bug[e].base; instruction_timer = 1, timer = 0);
        let actor = word!(game.cur_enemy_index);
        let instruction = facing_animation(actor, game);
        slot!(game, Bug[e].base.current_instruction = instruction);
    }
}
pub fn settle<H: ActorHost + ActorRuntime + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Bug[e].var_C, |old| old.wrapping_add(1));
    slot!(game, Bug[e].var_A = 0x8e5a);
}
pub fn move_to_limit<H: ActorHost + ActorRuntime + ?Sized>(offset: i32, game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Bug[e].var_C, |old| old.wrapping_add(1));
    slot!(
        game,
        Bug[e].base(y_pos, y_subpos) += speed(if offset < 0 { 66 } else { 64 }, game)
    );
    let delta = (i32::from(slot!(game, Bug[e].base.y_pos))
        - offset
        - i32::from(slot!(game, Bug[e].var_01))) as i16;
    if if offset < 0 { delta < 0 } else { delta >= 0 } {
        slot!(
            game,
            Bug[e].base.y_pos = slot!(game, Bug[e].var_01).wrapping_add(offset as u16)
        );
        slot!(game, Bug[e].base.y_subpos = 0);
        slot!(game, Bug[e].var_A = 0x8e5a);
    }
}
pub fn fly<H: ActorHost + ActorRuntime + ?Sized>(speed_offset: isize, game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let index = (slot!(game, Bug[e].var_B) >> 1) as isize;
    slot!(
        game,
        Bug[e].base(x_pos, x_subpos) += speed(index + speed_offset, game)
    );
}
pub fn start_flight<H: ActorHost + ActorRuntime + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Bug[e].var_C, |old| old.wrapping_add(1));
    if (slot!(game, Bug[e].var_C) as i16).wrapping_sub(slot!(game, Bug[e].var_02) as i16) >= 0 {
        slot!(game, Bug[e].var_C = 0);
        slot!(game, Bug[e].base; instruction_timer = 1, timer = 0);
        slot!(game, Bug[e].var_A = 0x8e14);
        slot!(game, Bug[e].base.current_instruction = 0x8b05);
        let actor = word!(game.cur_enemy_index);
        if game.player_delta(actor, Axis::X) & 0x8000 == 0 {
            slot!(game, Bug[e].var_A = 0x8e35);
            slot!(game, Bug[e].base.current_instruction = 0x8b45);
        }
    }
}
