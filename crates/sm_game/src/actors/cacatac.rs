//! Cacatac, the spiked cactus that paces and fires spikes, and Owtch, the spiked shell that
//! shares its patrol and burrows (bank $A2:9E6A-$A2:A59D). Was `ground_patrol`.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess as _, ActorKey};
use crate::actors::api::ShotHandling;
use crate::actors::combat::{self, current_slot};
use crate::actors::host::{ActorLease, LeaseActors};
use crate::actors::layouts::part_view;
use crate::actors::session_slots::SlotAccess as _;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::table;
use crate::session_slot;
use sm_data::enemy_speeds::SpeedTable::Linear;

part_view! {
    Cacatac: EnemyLayout40 {
        words: [right_subvelocity, right_velocity, left_subvelocity, left_velocity, direction, function, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [min_x, max_x, var_02, var_03, var_04, var_05, var_06, var_07, var_08, var_09, var_0a, var_0b, var_0c,
            var_0d, var_0e, var_0f, var_10, var_11, var_12, var_13, var_14, var_15, var_16, var_17, var_18, var_19,
            var_1a, var_1b, var_1c, var_1d, var_1e, var_1f],
    }
}

const MOVING_LEFT: u16 = 0x9fba;
const MOVING_RIGHT: u16 = 0x9fec;
const ATTACKING: u16 = 0xa01b; // An RTS: wait for the attack's instructions.
const LISTS: [[u16; 2]; 2] = [[0x9eda, 0x9f00], [0x9e8a, 0x9eb0]]; // Idle, attack; down, up.
fn set_instruction_list(runtime: &mut impl ActorLease, attacking: bool) {
    let cacatac = current_slot(runtime);
    let up = usize::from(session_slot!(runtime, Cacatac[cacatac].parameter_1) >> 8 != 0);
    let key = combat::current(runtime);
    let list = LISTS[up][usize::from(attacking)];
    runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
}
pub(crate) fn set_bounds(runtime: &mut impl ActorLease, owtch: bool, distances: u32, range: u16) {
    let min_x =
        crate::ram_slot!(@at Cacatac, current_slot(runtime), min_x) + 2 * usize::from(owtch);
    let distance = table(runtime, distances, range as isize);
    let key = combat::current(runtime);
    let x = runtime.resolve_actor(key, Native).x.whole;
    runtime.set_slot_word(min_x + 2, distance.wrapping_add(x));
    runtime.set_slot_word(min_x, x.wrapping_sub(distance));
}
pub(crate) fn set_velocities(runtime: &mut impl ActorLease, speed: u16) {
    let speeds = [0, 1, 2, 3].map(|i| runtime.lease_enemy_speed(Linear, 4 * speed as isize + i));
    session_slot!(runtime, Cacatac[current_slot(runtime)]; right_velocity = speeds[0],
        right_subvelocity = speeds[1], left_velocity = speeds[2], left_subvelocity = speeds[3]);
}
/// Initialisation AI - cacatac ($A2:9F48).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let cacatac = current_slot(runtime);
    session_slot!(runtime, Cacatac[cacatac].base.spritemap_pointer = 0x804d);
    set_instruction_list(runtime, false);
    let [direction, _] = session_slot!(runtime, Cacatac[cacatac].parameter_1).to_le_bytes();
    let function = table(runtime, 0xa2_9f42, isize::from(direction));
    session_slot!(runtime, Cacatac[cacatac]; direction = u16::from(direction), function = function);
    let [range, speed] = session_slot!(runtime, Cacatac[cacatac].parameter_2).to_le_bytes();
    set_bounds(runtime, false, 0xa2_9f36, u16::from(range));
    set_velocities(runtime, u16::from(speed));
}
/// Move by the left or right velocity; at `bound`, turn, and a random draw below `odds`.
pub(crate) fn patrol(runtime: &mut impl ActorLease, owtch: bool, right: bool) {
    let slot = current_slot(runtime);
    let speed = match right {
        true => session_slot!(runtime, Cacatac[slot](right_velocity, right_subvelocity)),
        false => session_slot!(runtime, Cacatac[slot](left_velocity, left_subvelocity)),
    };
    session_slot!(runtime, Cacatac[slot].base(x_pos, x_subpos) += speed);
    let x = session_slot!(runtime, Cacatac[slot].base.x_pos);
    let min_x = crate::ram_slot!(@at Cacatac, slot, min_x);
    let bound = runtime.slot_word(min_x + 2 * (usize::from(right) + usize::from(owtch)));
    if (x.wrapping_sub(bound) as i16 >= 0) == right {
        if owtch {
            let left = session_slot!(runtime, Cacatac[slot].direction).wrapping_sub(1);
            session_slot!(
                runtime,
                Cacatac[slot].direction = if right { 0 } else { left }
            );
        } else {
            let function = if right { MOVING_LEFT } else { MOVING_RIGHT };
            session_slot!(runtime, Cacatac[slot]; function = function, direction = u16::from(!right));
        }
    }
    // Maybe make cacatac attack ($A2:A01C) or owtch sink ($A2:A553).
    runtime.next_random();
    let frame = session_slot!(runtime, Cacatac[slot].base.frame_counter) as u8;
    let random = runtime.word(crate::ram::words::RANDOM_NUMBER) as u8;
    if frame.wrapping_add(random) < if owtch { 6 } else { 3 } {
        if owtch {
            session_slot!(runtime, Cacatac[slot].direction = 3);
        } else {
            session_slot!(runtime, Cacatac[slot].function = ATTACKING);
            set_instruction_list(runtime, true);
        }
    }
}
/// Cacatac function - moving left ($A2:9FBA) or right ($A2:9FEC).
pub fn moving<H: LeaseActors + ?Sized>(right: bool, game: &mut H) {
    patrol(&mut game.lease(), false, right);
}
/// Instruction - function = moving left/right ($A2:A095), by the direction of enemy `k`.
pub fn resume_moving<H: LeaseActors + ?Sized>(k: u16, j: EnemyScript, game: &mut H) -> EnemyScript {
    game.lease()
        .edit_actor(ActorKey::from_legacy(k), Native, |actor| {
            actor.pre_instruction =
                [MOVING_LEFT, MOVING_RIGHT][usize::from(actor.variables[4] != 0)]
        });
    j
}
/// Owtch ($A2:A38B-$A2:A59D).
pub mod owtch {
    use super::*;
    part_view! {
        Owtch: EnemyLayout40 {
            words: [right_subvelocity, right_velocity, left_subvelocity, left_velocity, function_index, sink_offset, parameter_1, parameter_2],
            pad1: [u8; 26696],
            words: [underground_timer, min_x, max_x, var_03, var_04, var_05, var_06, var_07, var_08, var_09, var_0a, var_0b,
                var_0c, var_0d, var_0e, var_0f, var_10, var_11, var_12, var_13, var_14, var_15, var_16, var_17, var_18,
                var_19, var_1a, var_1b, var_1c, var_1d, var_1e, var_1f],
        }
    }
    /// Function indices; -2 and -1 set the moving left and right lists.
    const UNDERGROUND: u16 = 2;
    const SINKING: u16 = 3;
    const RISING: u16 = 4;
    const SINK_DEPTH: u16 = 16;
    /// Set owtch instruction list pointer - moving left ($A2:A48A) or right ($A2:A49D).
    fn set_instruction_list(runtime: &mut impl ActorLease, right: bool) {
        let key = combat::current(runtime);
        let list = if right { 0xa3bd } else { 0xa3ab };
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
    }
    fn reload_underground_timer(runtime: &mut impl ActorLease, owtch: i32) {
        let index = session_slot!(runtime, Owtch[owtch].parameter_1) >> 8;
        let timer = table(runtime, 0xa2_a3ed, index as isize);
        session_slot!(runtime, Owtch[owtch].underground_timer = timer);
    }
    /// Initialisation AI - owtch ($A2:A3F9).
    pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
        let mut lease = game.lease();
        let runtime = &mut lease;
        let owtch = current_slot(runtime);
        session_slot!(runtime, Owtch[owtch].base.spritemap_pointer = 0x804d);
        let index = session_slot!(runtime, Owtch[owtch].parameter_1) & 0xff;
        session_slot!(runtime, Owtch[owtch].function_index = index);
        set_instruction_list(runtime, index & 1 != 0);
        let [speed, range] = session_slot!(runtime, Owtch[owtch].parameter_2).to_le_bytes();
        set_velocities(runtime, u16::from(speed));
        set_bounds(runtime, true, 0xa2_a3dd, u16::from(range));
        reload_underground_timer(runtime, owtch);
        let underground = session_slot!(runtime, Owtch[owtch].function_index) == UNDERGROUND;
        let depth = SINK_DEPTH * u16::from(underground);
        session_slot!(runtime, Owtch[owtch].sink_offset = depth);
        if underground {
            let y = session_slot!(runtime, Owtch[owtch].base.y_pos).wrapping_add(SINK_DEPTH);
            session_slot!(runtime, Owtch[owtch].base.y_pos = y);
        }
    }
    /// Main AI - owtch ($A2:A47E): run its function index.
    pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
        let mut lease = game.lease();
        let runtime = &mut lease;
        let owtch = current_slot(runtime);
        let index = session_slot!(runtime, Owtch[owtch].function_index) as i16;
        match index {
            -2 | -1 => set_instruction_list(runtime, index == -1),
            0 | 1 => patrol(runtime, true, index == 1),
            2 => {
                let timer = session_slot!(runtime, Owtch[owtch].underground_timer).wrapping_sub(1);
                session_slot!(runtime, Owtch[owtch].underground_timer = timer);
                if timer == 0 {
                    session_slot!(runtime, Owtch[owtch].function_index = RISING);
                }
            }
            3 | 4 => {
                // Owtch function index 3 - sinking ($A2:A517) or 4 - rising ($A2:A53E).
                let sinking = index as u16 == SINKING;
                let step = if sinking { 1 } else { u16::MAX };
                session_slot!(runtime, Owtch[owtch].base.y_pos, |y| y.wrapping_add(step));
                let previous = session_slot!(runtime, Owtch[owtch].sink_offset);
                let offset = previous.wrapping_add(step);
                session_slot!(runtime, Owtch[owtch].sink_offset = offset);
                if sinking && previous.wrapping_add(1).wrapping_sub(SINK_DEPTH) & 0x8000 == 0 {
                    session_slot!(runtime, Owtch[owtch].function_index = UNDERGROUND);
                    reload_underground_timer(runtime, owtch);
                } else if !sinking && previous == 1 {
                    let direction =
                        u16::from(runtime.lease_byte(crate::ram::words::RANDOM_NUMBER + 1)) & 1;
                    session_slot!(runtime, Owtch[owtch].function_index = direction);
                }
            }
            // The source indexes its table signed; past either end fails.
            _ => panic!("owtch function index {index}"),
        }
    }
    /// Instruction - enemy function index = 0 (moving left, $A2:A56D) or 1 ($A2:A571).
    pub fn set_function_index<H: LeaseActors + ?Sized>(
        ea: u32,
        k: u16,
        j: EnemyScript,
        game: &mut H,
    ) -> EnemyScript {
        let index = u16::from(ea == 0xa2_a571);
        let key = ActorKey::from_legacy(k);
        game.lease()
            .edit_actor(key, Native, |actor| actor.variables[4] = index);
        j
    }
    /// Enemy shot - owtch ($A2:A579): hurt only while moving.
    pub fn shot<H: LeaseActors + ?Sized>(game: &mut H) {
        let mut lease = game.lease();
        let runtime = &mut lease;
        let owtch = current_slot(runtime);
        if session_slot!(runtime, Owtch[owtch].function_index).wrapping_sub(1) & 0x8000 != 0 {
            combat::shot_reaction(runtime, ShotHandling::Finalize);
        }
    }
}
