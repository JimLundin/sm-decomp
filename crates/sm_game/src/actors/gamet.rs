//! Gamet's ordered five-part emergence and flight.
//! Native actor words remain authoritative across the existing query boundaries.
use crate::actors::host::{ActorHost, ActorRuntime, Axis};
use crate::actors::layouts::EnemyLayout72 as Bug;
use crate::actors::session_slots::SlotAccess;
use crate::ram::word;
use crate::session_slot;
use sm_data::enemy_speeds::SpeedTable;

pub fn initialize<H: ActorHost + ActorRuntime + SlotAccess + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.CUR_ENEMY_INDEX));
    session_slot!(game, Bug[e]; var_d = session_slot!(game, Bug[e].base.x_pos),
        var_e = session_slot!(game, Bug[e].base.y_pos));
    session_slot!(game, Bug[e].base.properties, |old| old | 0x100);
    session_slot!(
        game,
        Bug[e].var_b =
            8u16.wrapping_mul(u16::from(session_slot!(byte game, Bug[e].parameter_2[1])))
    );
    session_slot!(game, Bug[e].var_c = 0);
    session_slot!(game, Bug[e].base; instruction_timer = 1, timer = 0, current_instruction = 0x8ae1);
    session_slot!(game, Bug[e].var_a = 0x8bcd);
}
pub fn state_address<H: ActorHost + ActorRuntime + SlotAccess + ?Sized>(game: &H) -> u32 {
    let e = i32::from(word!(game.CUR_ENEMY_INDEX));
    0xb30000 | u32::from(session_slot!(game, Bug[e].var_a))
}
pub fn reset_offscreen<H: ActorHost + ActorRuntime + SlotAccess + ?Sized>(game: &mut H) {
    if game.current_outside_screen() {
        let e = i32::from(word!(game.CUR_ENEMY_INDEX));
        session_slot!(game, Bug[e].base.properties, |old| old | 0x100);
        session_slot!(game, Bug[e].var_a = 0x8bcd);
        session_slot!(game, Bug[e].base; x_pos = session_slot!(game, Bug[e].var_d), y_pos = session_slot!(game, Bug[e].var_e));
    }
}
fn speed<H: ActorHost + ActorRuntime + SlotAccess + ?Sized>(index: isize, game: &H) -> u32 {
    let high = game.enemy_speed(SpeedTable::Linear, index);
    let low = game.enemy_speed(SpeedTable::Linear, index + 1);
    (u32::from(high) << 16) | u32::from(low)
}
fn facing_animation<H: ActorHost + ActorRuntime + SlotAccess + ?Sized>(
    actor: u16,
    game: &mut H,
) -> u16 {
    if game.player_delta(actor, Axis::X) & 0x8000 == 0 {
        0x8b21
    } else {
        0x8ae1
    }
}
fn plan_group<H: ActorHost + ActorRuntime + SlotAccess + ?Sized>(k: i32, game: &mut H) {
    // Finish each field's five stores before starting the next field.
    for offset in [0, 64, 128, 192, 256] {
        session_slot!(game, Bug[k + offset].var_a = 0x8ca6);
    }
    for (part, delay) in [104, 96, 88, 112, 120].into_iter().enumerate() {
        session_slot!(game, Bug[k + 64 * part as i32].var_02 = delay);
    }
    for (part, state) in [0x8cff, 0x8d0c, 0x8d4e, 0x8d90, 0x8dd2]
        .into_iter()
        .enumerate()
    {
        session_slot!(game, Bug[k + 64 * part as i32].var_f = state);
    }
}
pub fn await_group<H: ActorHost + ActorRuntime + SlotAccess + ?Sized>(game: &mut H) {
    let current = i32::from(word!(game.CUR_ENEMY_INDEX));
    if session_slot!(byte game, Bug[current].parameter_2[0]) != 0
        && [64, 128, 192, 256].into_iter().all(|offset| {
            let e = i32::from(word!(game.CUR_ENEMY_INDEX)) + offset;
            session_slot!(game, Bug[e].var_a) == 0x8bcd
        })
    {
        let e = i32::from(word!(game.CUR_ENEMY_INDEX));
        session_slot!(game, Bug[e].var_a = 0x8bff);
    }
}
pub fn await_player<H: ActorHost + ActorRuntime + SlotAccess + ?Sized>(game: &mut H) {
    let k = word!(game.CUR_ENEMY_INDEX);
    let e = i32::from(word!(game.CUR_ENEMY_INDEX));
    let actor = word!(game.CUR_ENEMY_INDEX);
    let distance = u16::from(session_slot!(byte game, Bug[e].parameter_2[0]));
    let in_range = game.player_within(actor, Axis::X, distance);
    if in_range
        && (session_slot!(game, Bug[e].base.y_pos) as i16).wrapping_sub(game.player_y() as i16) >= 0
    {
        session_slot!(game, Bug[e].var_c, |old| old.wrapping_add(1));
        session_slot!(game, Bug[e].base; instruction_timer = 1, timer = 0);
        let instruction = facing_animation(k, game);
        for offset in [0, 64, 128, 192, 256] {
            session_slot!(
                game,
                Bug[i32::from(k) + offset].base.current_instruction = instruction
            );
        }
        plan_group(i32::from(k), game);
    }
}
pub fn rise<H: ActorHost + ActorRuntime + SlotAccess + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.CUR_ENEMY_INDEX));
    session_slot!(game, Bug[e].base.properties, |old| old & !0x100);
    session_slot!(game, Bug[e].base(y_pos, y_subpos) += speed(66, game));
    if (session_slot!(game, Bug[e].base.y_pos) as i16).wrapping_sub(game.player_y() as i16) < 0 {
        session_slot!(game, Bug[e]; var_a = session_slot!(game, Bug[e].var_f),
            var_01 = session_slot!(game, Bug[e].base.y_pos));
        session_slot!(game, Bug[e].base; instruction_timer = 1, timer = 0);
        let actor = word!(game.CUR_ENEMY_INDEX);
        let instruction = facing_animation(actor, game);
        session_slot!(game, Bug[e].base.current_instruction = instruction);
    }
}
pub fn settle<H: ActorHost + ActorRuntime + SlotAccess + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.CUR_ENEMY_INDEX));
    session_slot!(game, Bug[e].var_c, |old| old.wrapping_add(1));
    session_slot!(game, Bug[e].var_a = 0x8e5a);
}
pub fn move_to_limit<H: ActorHost + ActorRuntime + SlotAccess + ?Sized>(offset: i32, game: &mut H) {
    let e = i32::from(word!(game.CUR_ENEMY_INDEX));
    session_slot!(game, Bug[e].var_c, |old| old.wrapping_add(1));
    session_slot!(
        game,
        Bug[e].base(y_pos, y_subpos) += speed(if offset < 0 { 66 } else { 64 }, game)
    );
    let delta = (i32::from(session_slot!(game, Bug[e].base.y_pos))
        - offset
        - i32::from(session_slot!(game, Bug[e].var_01))) as i16;
    if if offset < 0 { delta < 0 } else { delta >= 0 } {
        session_slot!(
            game,
            Bug[e].base.y_pos = session_slot!(game, Bug[e].var_01).wrapping_add(offset as u16)
        );
        session_slot!(game, Bug[e].base.y_subpos = 0);
        session_slot!(game, Bug[e].var_a = 0x8e5a);
    }
}
pub fn fly<H: ActorHost + ActorRuntime + SlotAccess + ?Sized>(speed_offset: isize, game: &mut H) {
    let e = i32::from(word!(game.CUR_ENEMY_INDEX));
    let index = (session_slot!(game, Bug[e].var_b) >> 1) as isize;
    session_slot!(
        game,
        Bug[e].base(x_pos, x_subpos) += speed(index + speed_offset, game)
    );
}
pub fn start_flight<H: ActorHost + ActorRuntime + SlotAccess + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.CUR_ENEMY_INDEX));
    session_slot!(game, Bug[e].var_c, |old| old.wrapping_add(1));
    if (session_slot!(game, Bug[e].var_c) as i16)
        .wrapping_sub(session_slot!(game, Bug[e].var_02) as i16)
        >= 0
    {
        session_slot!(game, Bug[e].var_c = 0);
        session_slot!(game, Bug[e].base; instruction_timer = 1, timer = 0);
        session_slot!(game, Bug[e].var_a = 0x8e14);
        session_slot!(game, Bug[e].base.current_instruction = 0x8b05);
        let actor = word!(game.CUR_ENEMY_INDEX);
        if game.player_delta(actor, Axis::X) & 0x8000 == 0 {
            session_slot!(game, Bug[e].var_a = 0x8e35);
            session_slot!(game, Bug[e].base.current_instruction = 0x8b45);
        }
    }
}
