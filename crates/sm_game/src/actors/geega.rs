//! Geega detection, straight flight, curved turns and home reset.
//! Native actor fields retain authority and original query/ROM failure ordering.
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{Axis, BossEffects};
use crate::actors::layouts::EnemyLayout72 as Bug;
use crate::session_slot;
use sm_data::enemy_speeds::SpeedTable;

/// A lease on a game with the geegas' effects.
pub trait Lease: BossLease<Game: BossEffects> {}
impl<T: BossLease<Game: BossEffects> + ?Sized> Lease for T {}

/// Main AI - geega ($B3:8FAE): run its state routine.
pub fn main_ai<H: Lease + ?Sized>(game: &mut H) {
    match state_address(game) {
        11767733 => detect_player(game),
        11767797 => launch_delay(game.current_enemy(), game),
        11767848 => fly(true, game),
        11767997 => fly(false, game),
        11768154 => turn(true, game),
        11768280 => turn(false, game),
        _ => game.foreign(|game| game.unreachable()),
    }
}

pub fn initialize<H: Lease + ?Sized>(game: &mut H) {
    let e = i32::from(game.current_enemy());
    session_slot!(game, Bug[e]; var_07 = session_slot!(game, Bug[e].base.x_pos),
        var_08 = session_slot!(game, Bug[e].base.y_pos));
    session_slot!(game, Bug[e].base; x_subpos = 0, y_subpos = 0, instruction_timer = 1, timer = 0, current_instruction = 0x8efc);
    if session_slot!(game, Bug[e].parameter_1) == 0 {
        session_slot!(game, Bug[e].base.current_instruction = 0x8f24);
    }
    let index = ((8 * i32::from(session_slot!(game, Bug[e].parameter_2))) >> 1) as isize;
    session_slot!(game, Bug[e]; var_01 = game.enemy_speed(SpeedTable::Linear, index), var_00 = game.enemy_speed(SpeedTable::Linear, index + 1),
        var_03 = game.enemy_speed(SpeedTable::Linear, index + 2), var_02 = game.enemy_speed(SpeedTable::Linear, index + 3), var_a = 0x8fb5, var_06 = 0);
}
pub fn state_address<H: Lease + ?Sized>(game: &H) -> u32 {
    let e = i32::from(game.current_enemy());
    0xb30000 | u32::from(session_slot!(game, Bug[e].var_a))
}
fn reset_home<H: Lease + ?Sized>(e: i32, game: &mut H) {
    session_slot!(game, Bug[e].base; x_pos = session_slot!(game, Bug[e].var_07), y_pos = session_slot!(game, Bug[e].var_08), x_subpos = 0, y_subpos = 0);
    session_slot!(game, Bug[e]; var_a = 0x8fb5, var_06 = 0);
}
fn horizontal<H: Lease + ?Sized>(left: bool, k: u16, game: &mut H) {
    let e = i32::from(k);
    let (high, low) = if left {
        (
            session_slot!(game, Bug[e].var_03),
            session_slot!(game, Bug[e].var_02),
        )
    } else {
        (
            session_slot!(game, Bug[e].var_01),
            session_slot!(game, Bug[e].var_00),
        )
    };
    session_slot!(
        game,
        Bug[e].base(x_pos, x_subpos) += (u32::from(high) << 16) | u32::from(low)
    );
}
fn vertical<H: Lease + ?Sized>(counting_down: bool, k: u16, game: &mut H) {
    let e = i32::from(k);
    if counting_down {
        session_slot!(game, Bug[e].var_e, |old| old.wrapping_sub(1));
        if session_slot!(game, Bug[e].var_e) & 0x8000 != 0 {
            session_slot!(game, Bug[e]; var_e = 0, var_f = 0);
            return;
        }
    } else {
        session_slot!(game, Bug[e].var_e, |old| old.wrapping_add(1));
    }
    let index = ((8 * i32::from(session_slot!(game, Bug[e].var_e))) >> 1) as isize;
    let offset = if counting_down { 0 } else { 2 };
    let high = game.enemy_speed(SpeedTable::Quadratic, index + offset + 1);
    let low = game.enemy_speed(SpeedTable::Quadratic, index + offset);
    session_slot!(
        game,
        Bug[e].base(y_pos, y_subpos) += (u32::from(high) << 16) | u32::from(low)
    );
}
pub fn detect_player<H: Lease + ?Sized>(game: &mut H) {
    let e = i32::from(game.current_enemy());
    let left = session_slot!(game, Bug[e].parameter_1) != 0;
    let actor = game.current_enemy();
    let dx = game.player_delta(actor, Axis::X) as i16;
    let in_x_range = if left {
        dx < 0 && (i32::from(dx) + 192) & 0x8000 == 0
    } else {
        dx >= 0 && (i32::from(dx) - 192) & 0x8000 != 0
    };
    if !in_x_range {
        return;
    }
    let actor = game.current_enemy();
    let in_y_range = game.player_within(actor, Axis::Y, 0x30);
    if in_y_range {
        session_slot!(game, Bug[e].base.properties, |old| old & !0x100);
        session_slot!(game, Bug[e]; var_20 = 24, var_a = 0x8ff5);
    }
}
pub fn launch_delay<H: Lease + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(k);
    let delay = session_slot!(game, Bug[e].var_20).wrapping_sub(1);
    session_slot!(game, Bug[e].var_20 = delay);
    if delay == 0 {
        session_slot!(game, Bug[e].base; instruction_timer = 1, timer = 0, current_instruction = 0x8efc);
        session_slot!(game, Bug[e].var_a = 0x9028);
        if session_slot!(game, Bug[e].parameter_1) == 0 {
            session_slot!(game, Bug[e].base.current_instruction = 0x8f24);
            session_slot!(game, Bug[e].var_a = 0x90bd);
        }
    }
}
pub fn fly<H: Lease + ?Sized>(left: bool, game: &mut H) {
    let e = i32::from(game.current_enemy());
    horizontal(left, game.current_enemy(), game);
    if game.current_outside_screen() {
        reset_home(e, game);
        session_slot!(game, Bug[e].base.properties, |old| old | 0x100);
    } else if session_slot!(game, Bug[e].var_06) == 0
        && if left {
            (i32::from(session_slot!(game, Bug[e].base.x_pos)) - i32::from(game.player_x()) - 48)
                & 0x8000
                != 0
        } else {
            (i32::from(game.player_x()) - i32::from(session_slot!(game, Bug[e].base.x_pos)) - 48)
                & 0x8000
                != 0
        }
    {
        let (state, instruction) = if left {
            (0x915a, 0x8f10)
        } else {
            (0x91d8, 0x8f38)
        };
        session_slot!(game, Bug[e]; var_a = state, var_c = 0, var_d = 0, var_e = 40, var_f = 1, var_04 = session_slot!(game, Bug[e].base.x_pos));
        session_slot!(game, Bug[e].base; instruction_timer = 1, timer = 0, current_instruction = instruction);
    }
}
pub fn turn<H: Lease + ?Sized>(left: bool, game: &mut H) {
    let e = i32::from(game.current_enemy());
    let instruction = if left { 0x8efc } else { 0x8f24 };
    if game.current_outside_screen() {
        reset_home(e, game);
        session_slot!(game, Bug[e].base; instruction_timer = 1, timer = 0, current_instruction = instruction);
        session_slot!(game, Bug[e].base.properties, |old| old | 0x100);
    } else {
        horizontal(left, game.current_enemy(), game);
        let counting_down = session_slot!(game, Bug[e].var_f) != 0;
        vertical(counting_down, game.current_enemy(), game);
        if !counting_down {
            let finished = if left {
                (session_slot!(game, Bug[e].base.y_pos) as i16).wrapping_sub(session_slot!(
                    game,
                    Bug[e].var_08
                )
                    as i16)
                    < 0
            } else {
                (session_slot!(game, Bug[e].var_08) as i16)
                    .wrapping_sub(session_slot!(game, Bug[e].base.y_pos) as i16)
                    >= 0
            };
            if finished {
                session_slot!(game, Bug[e]; var_06 = 1, var_f = 1);
                // The two legacy turns publish their next state on opposite sides of animation.
                if left {
                    session_slot!(game, Bug[e].var_a = 0x9028);
                }
                session_slot!(game, Bug[e].base; instruction_timer = 1, timer = 0, current_instruction = instruction);
                if !left {
                    session_slot!(game, Bug[e].var_a = 0x90bd);
                }
            }
        }
    }
}
