//! Atomic, the Wrecked Ship's spinning orb, which drifts towards Samus (bank
//! $A8:E230-$A8:E489). Was `wrecked_ship_orbs`.
use crate::actors::boss_lease::BossState;
use crate::actors::host::Axis;
use crate::actors::layouts::part_view;
use crate::actors::session_slots::session_slot;
use sm_data::enemy_speeds::SpeedTable;

/// What Atomic needs beyond the shared actor operations: its instruction lists
/// and the unreachable diagnostic.
pub trait Host: BossState {
    fn unreachable(&mut self);
}

part_view! {
    /// Atomic's view of its slot.
    Atomic: EnemyLayout40 {
        words: [
            y_movement_function, x_movement_function, var_c, var_d, var_e, var_f, direction,
            speed_index,
        ],
        pad1: [u8; 26696],
        words: [
            subspeed, speed, negated_subspeed, negated_speed,
        ],
        _: [u16; 28],
    }
}

/// Bank $A8 Atomic movement functions.
pub mod function {
    pub const MOVE_UP: u16 = 0xe405;
    pub const MOVE_DOWN: u16 = 0xe424;
    pub const MOVE_LEFT: u16 = 0xe443;
    pub const MOVE_RIGHT: u16 = 0xe462;
}

const SIGN: u16 = 0x8000;

fn current(game: &(impl Host + ?Sized)) -> i32 {
    i32::from(game.current_enemy())
}
/// Initialisation AI - Atomic ($A8:E388).
pub fn initialize(game: &mut (impl Host + ?Sized)) {
    let atomic = current(game);
    session_slot!(game, Atomic[atomic].base; instruction_timer = 1, timer = 0,
    current_instruction = crate::enemy_lists::orb_instruction_list(
        game,
        session_slot!(game, Atomic[atomic].direction) as isize
    ));
    let index = 4 * session_slot!(game, Atomic[atomic].speed_index) as isize;
    session_slot!(game, Atomic[atomic];
        speed = game.enemy_speed(SpeedTable::Linear, index),
        subspeed = game.enemy_speed(SpeedTable::Linear, index + 1),
        negated_speed = game.enemy_speed(SpeedTable::Linear, index + 2),
        negated_subspeed = game.enemy_speed(SpeedTable::Linear, index + 3));
}
/// Atomic Y movement function - move up or down ($A8:E405, $A8:E424), or X
/// movement function - move left or right ($A8:E443, $A8:E462).
fn move_atomic(horizontal: bool, function: u16, game: &mut (impl Host + ?Sized)) {
    let atomic = current(game);
    let amount = match (horizontal, function) {
        (false, function::MOVE_UP) | (true, function::MOVE_LEFT) => {
            let high = session_slot!(game, Atomic[atomic].negated_speed);
            let low = session_slot!(game, Atomic[atomic].negated_subspeed);
            (u32::from(high) << 16) | u32::from(low)
        }
        (false, function::MOVE_DOWN) | (true, function::MOVE_RIGHT) => {
            let high = session_slot!(game, Atomic[atomic].speed);
            let low = session_slot!(game, Atomic[atomic].subspeed);
            (u32::from(high) << 16) | u32::from(low)
        }
        _ => {
            game.unreachable();
            return;
        }
    };
    if horizontal {
        session_slot!(game, Atomic[atomic].base(x_pos, x_subpos) += amount);
    } else {
        session_slot!(game, Atomic[atomic].base(y_pos, y_subpos) += amount);
    }
}
/// Main AI - Atomic ($A8:E3C3): set the Y ($A8:E3D9) and X ($A8:E3EF)
/// movement functions towards Samus, then run them. Was `advance`.
pub fn main_ai(game: &mut (impl Host + ?Sized)) {
    let atomic = current(game);
    session_slot!(game, Atomic[atomic].y_movement_function = function::MOVE_UP);
    let k = game.current_enemy();
    if game.player_delta(k, Axis::Y) & SIGN == 0 {
        session_slot!(
            game,
            Atomic[atomic].y_movement_function = function::MOVE_DOWN
        );
    }
    let atomic = current(game);
    session_slot!(
        game,
        Atomic[atomic].x_movement_function = function::MOVE_LEFT
    );
    let k = game.current_enemy();
    if game.player_delta(k, Axis::X) & SIGN == 0 {
        session_slot!(
            game,
            Atomic[atomic].x_movement_function = function::MOVE_RIGHT
        );
    }
    let atomic = current(game);
    move_atomic(
        false,
        session_slot!(game, Atomic[atomic].y_movement_function),
        game,
    );
    move_atomic(
        true,
        session_slot!(game, Atomic[atomic].x_movement_function),
        game,
    );
}
