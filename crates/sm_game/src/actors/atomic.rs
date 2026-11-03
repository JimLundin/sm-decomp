//! Atomic, the Wrecked Ship's spinning orb, which drifts towards Samus (bank
//! $A8:E230-$A8:E489). Was `wrecked_ship_orbs`.
use crate::actors::boss_lease::BossState;
use crate::actors::host::Axis;
use crate::actors::slot_view::{SlotState, slot_view};
use sm_data::enemy_speeds::SpeedTable;

/// What Atomic needs beyond the shared actor operations: its instruction lists
/// and the unreachable diagnostic.
pub trait Host: BossState + SlotState {
    fn unreachable(&mut self);
}

slot_view! {
    /// Atomic's view of its slot.
    Atomic, AtomicEdit {
        record: [
            y_movement_function = var_a, x_movement_function = var_b, direction = parameter_1,
            speed_index = parameter_2,
        ],
        extended: [
            subspeed = var_00, speed = var_01, negated_subspeed = var_02, negated_speed = var_03,
        ],
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

fn current(game: &(impl Host + ?Sized)) -> u16 {
    game.current_enemy()
}
/// Initialisation AI - Atomic ($A8:E388).
pub fn initialize(game: &mut (impl Host + ?Sized)) {
    let atomic = current(game);
    let direction = {
        let mut edit = AtomicEdit::of(game, atomic);
        edit.actor().instruction_delay = 1;
        edit.actor().timer = 0;
        *edit.direction()
    };
    let list = crate::enemy_lists::orb_instruction_list(game, direction as isize);
    AtomicEdit::of(game, atomic).actor().instruction = list;
    let index = 4 * Atomic::of(game, atomic).speed_index() as isize;
    let speed = game.enemy_speed(SpeedTable::Linear, index);
    *AtomicEdit::of(game, atomic).speed() = speed;
    let subspeed = game.enemy_speed(SpeedTable::Linear, index + 1);
    *AtomicEdit::of(game, atomic).subspeed() = subspeed;
    let negated_speed = game.enemy_speed(SpeedTable::Linear, index + 2);
    *AtomicEdit::of(game, atomic).negated_speed() = negated_speed;
    let negated_subspeed = game.enemy_speed(SpeedTable::Linear, index + 3);
    *AtomicEdit::of(game, atomic).negated_subspeed() = negated_subspeed;
}
/// Atomic Y movement function - move up or down ($A8:E405, $A8:E424), or X
/// movement function - move left or right ($A8:E443, $A8:E462).
fn move_atomic(horizontal: bool, function: u16, game: &mut (impl Host + ?Sized)) {
    let atomic = current(game);
    let amount = match (horizontal, function) {
        (false, function::MOVE_UP) | (true, function::MOVE_LEFT) => {
            let high = Atomic::of(game, atomic).negated_speed();
            let low = Atomic::of(game, atomic).negated_subspeed();
            (u32::from(high) << 16) | u32::from(low)
        }
        (false, function::MOVE_DOWN) | (true, function::MOVE_RIGHT) => {
            let high = Atomic::of(game, atomic).speed();
            let low = Atomic::of(game, atomic).subspeed();
            (u32::from(high) << 16) | u32::from(low)
        }
        _ => {
            game.unreachable();
            return;
        }
    };
    if horizontal {
        AtomicEdit::of(game, atomic)
            .actor()
            .x
            .advance(amount.cast_signed());
    } else {
        AtomicEdit::of(game, atomic)
            .actor()
            .y
            .advance(amount.cast_signed());
    }
}
/// Main AI - Atomic ($A8:E3C3): set the Y ($A8:E3D9) and X ($A8:E3EF)
/// movement functions towards Samus, then run them. Was `advance`.
pub fn main_ai(game: &mut (impl Host + ?Sized)) {
    let atomic = current(game);
    *AtomicEdit::of(game, atomic).y_movement_function() = function::MOVE_UP;
    let k = game.current_enemy();
    if game.player_delta(k, Axis::Y) & SIGN == 0 {
        *AtomicEdit::of(game, atomic).y_movement_function() = function::MOVE_DOWN;
    }
    let atomic = current(game);
    *AtomicEdit::of(game, atomic).x_movement_function() = function::MOVE_LEFT;
    let k = game.current_enemy();
    if game.player_delta(k, Axis::X) & SIGN == 0 {
        *AtomicEdit::of(game, atomic).x_movement_function() = function::MOVE_RIGHT;
    }
    let atomic = current(game);
    move_atomic(false, Atomic::of(game, atomic).y_movement_function(), game);
    move_atomic(true, Atomic::of(game, atomic).x_movement_function(), game);
}
