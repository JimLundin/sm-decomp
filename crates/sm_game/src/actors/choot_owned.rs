//! Choot motion and captured native pattern records on owned actor state.
#![forbid(unsafe_code)]
use super::{
    Choot, FALLING, FALLING_LIST, IDLE_LIST, JUMPING, JUMPING_LIST, PATTERN_END, PREPARE_TO_JUMP,
    SPEED_INDEX_STEP, WAIT_FOR_SAMUS,
};
use crate::actors::access::AccessMode::Native;
use crate::actors::api::{self, Axis};
use crate::actors::combat::{self, current_slot};
use crate::actors::session_slots::SessionSlots;
use crate::session_slot;
use sm_data::enemy_speeds::{SpeedTable, SpeedTable::Quadratic};

#[derive(Clone, Copy, Debug)]
pub enum Table {
    Patterns,
    PatternYDistances,
}

pub trait Host: SessionSlots + crate::actors::access::ActorAccess {
    fn choot_speed(&self, table: SpeedTable, index: isize) -> u16;
    fn choot_table(&self, table: Table, index: isize) -> u16;
    fn choot_record(&mut self, address: u16) -> sm_data::cartridge::RomRecord;
    fn unknown_choot_function(&mut self);
}

fn set_instruction_list(runtime: &mut impl Host, list: u16) {
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
}
/// The quadratic speed at word `index`, a whole word above its subword.
fn speed(runtime: &impl Host, index: usize) -> u32 {
    let high = runtime.choot_speed(Quadratic, (index + 1) as isize);
    (u32::from(high) << 16) | u32::from(runtime.choot_speed(Quadratic, index as isize))
}
/// Initialisation AI - choot ($A2:DF76).
pub fn initialize(runtime: &mut impl Host) {
    set_instruction_list(runtime, IDLE_LIST);
    let choot = current_slot(runtime);
    let (x, y) = (
        session_slot!(runtime, Choot[choot].base.x_pos),
        session_slot!(runtime, Choot[choot].base.y_pos),
    );
    session_slot!(runtime, Choot[choot]; function = WAIT_FOR_SAMUS, spawn_x = x, spawn_y = y);
    let index = (session_slot!(runtime, Choot[choot].parameter_1) >> 8) as isize;
    let pattern = runtime.choot_table(Table::Patterns, index);
    let distance_at = runtime.choot_table(Table::PatternYDistances, index);
    let distance = runtime.choot_record(distance_at).word(0);
    session_slot!(runtime, Choot[choot]; pattern = pattern, pattern_y_distance = distance);
    // Calculate choot jump height ($A2:DFCE) and initial choot jump speed ($A2:DFE9).
    let loops = session_slot!(runtime, Choot[choot].parameter_1) & 0xff;
    let height = loops * (session_slot!(runtime, Choot[choot].pattern_y_distance) & 0xff);
    let (mut covered, mut index) = (0u32, 0u16);
    loop {
        index = index.wrapping_add(SPEED_INDEX_STEP);
        covered = covered.wrapping_add(speed(runtime, 4 * usize::from(index >> 8)));
        if (covered >> 16).wrapping_sub(u32::from(height)) & 0x8000 == 0 {
            break;
        }
    }
    session_slot!(runtime, Choot[choot].initial_y_speed_index = index);
    let falling_y = session_slot!(runtime, Choot[choot].spawn_y).wrapping_sub(height);
    let falling_x = session_slot!(runtime, Choot[choot].spawn_x);
    session_slot!(runtime, Choot[choot]; falling_y = falling_y, falling_x = falling_x, y_speed_index = index);
}
/// Main AI - choot ($A2:E02E): run its function.
pub fn main_ai(runtime: &mut impl Host) {
    let choot = current_slot(runtime);
    match session_slot!(runtime, Choot[choot].function) {
        WAIT_FOR_SAMUS => {
            let key = combat::current(runtime);
            if api::player_within(runtime, key, Axis::X, 0x50) {
                let delay = session_slot!(runtime, Choot[choot].parameter_2);
                session_slot!(runtime, Choot[choot]; jump_delay = delay, function = PREPARE_TO_JUMP);
            }
        }
        PREPARE_TO_JUMP => {
            let delay = session_slot!(runtime, Choot[choot].jump_delay).wrapping_sub(1);
            session_slot!(runtime, Choot[choot].jump_delay = delay);
            if (delay as i16) < 0 {
                set_instruction_list(runtime, JUMPING_LIST);
                session_slot!(runtime, Choot[choot].function = JUMPING);
            }
        }
        JUMPING => {
            let index = session_slot!(runtime, Choot[choot].y_speed_index) >> 8;
            let rise = speed(runtime, 4 * usize::from(index) + 2);
            session_slot!(runtime, Choot[choot].base(y_pos, y_subpos) += rise);
            let index =
                session_slot!(runtime, Choot[choot].y_speed_index).wrapping_sub(SPEED_INDEX_STEP);
            session_slot!(runtime, Choot[choot].y_speed_index = index);
            if (index as i16) < 0 {
                let x = session_slot!(runtime, Choot[choot].falling_x);
                session_slot!(runtime, Choot[choot].origin_x = x);
                session_slot!(runtime, Choot[choot].base.x_pos = x);
                let y = session_slot!(runtime, Choot[choot].falling_y);
                session_slot!(runtime, Choot[choot].origin_y = y);
                session_slot!(runtime, Choot[choot].base.y_pos = y);
                let loops =
                    (session_slot!(runtime, Choot[choot].parameter_1) & 0xff).wrapping_sub(1);
                session_slot!(runtime, Choot[choot]; pattern_index = 0, pattern_loops = loops);
                set_instruction_list(runtime, FALLING_LIST);
                session_slot!(runtime, Choot[choot].function = FALLING);
            }
        }
        FALLING => {
            let step = 4 * (session_slot!(runtime, Choot[choot].pattern_index) >> 8);
            let row_at = session_slot!(runtime, Choot[choot].pattern).wrapping_add(step);
            let row = runtime.choot_record(row_at);
            if row.word(0) != PATTERN_END {
                let x = row
                    .word(0)
                    .wrapping_add(session_slot!(runtime, Choot[choot].origin_x));
                let y = row
                    .word(2)
                    .wrapping_add(session_slot!(runtime, Choot[choot].origin_y));
                session_slot!(runtime, Choot[choot].base; x_pos = x, y_pos = y);
                session_slot!(runtime, Choot[choot].pattern_index, |i| i
                    .wrapping_add(0x100));
                return;
            }
            let distance = session_slot!(runtime, Choot[choot].pattern_y_distance);
            session_slot!(runtime, Choot[choot].origin_y, |old| old
                .wrapping_add(distance));
            let loops = session_slot!(runtime, Choot[choot].pattern_loops).wrapping_sub(1);
            session_slot!(runtime, Choot[choot]; pattern_index = 0, pattern_loops = loops);
            if loops & 0x8000 != 0 {
                let (x, y) = (
                    session_slot!(runtime, Choot[choot].spawn_x),
                    session_slot!(runtime, Choot[choot].spawn_y),
                );
                let index = session_slot!(runtime, Choot[choot].initial_y_speed_index);
                session_slot!(runtime, Choot[choot].y_speed_index = index);
                session_slot!(runtime, Choot[choot].base; x_pos = x, x_subpos = 0, y_pos = y, y_subpos = 0);
                set_instruction_list(runtime, IDLE_LIST);
                session_slot!(runtime, Choot[choot].function = WAIT_FOR_SAMUS);
            }
        }
        _ => runtime.unknown_choot_function(),
    }
}
