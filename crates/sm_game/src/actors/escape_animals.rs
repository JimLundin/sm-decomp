//! Escape-room Etecoons and Dachora: setup, wall turns and script conditions.
#![forbid(unsafe_code)]
use crate::actors::layouts::EnemyLayout8 as Animal;
use crate::constants::enemy_properties::*;
use crate::enemy_script::EnemyScript;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};
use sm_data::escape_etecoons::EscapeEtecoon;

/// What the escape-room animals need beyond the common actor host.
pub trait Host: crate::actors::host::ActorHost {
    /// Whether game event `event` has happened.
    fn event_happened(&mut self, event: u16) -> bool;
    /// Etecoon `index`'s initial state at $B3:E718.
    fn escape_etecoon(&self, index: isize) -> EscapeEtecoon;
    /// Move actor `k` by `amount` (16.16) horizontally, ignoring slopes; true when blocked.
    fn move_horizontal_ignoring_slopes(&mut self, k: u16, amount: i32) -> bool;
    /// Move actor `k` by `amount` (16.16) vertically.
    fn move_vertical(&mut self, k: u16, amount: i32);
}

pub fn initialize(dachora: bool, game: &mut (impl Host + ?Sized)) {
    let e = i32::from(word!(game.cur_enemy_index));
    if game.event_happened(0xf) {
        slot!(game, Animal[e].base.properties, |old| old
            | kEnemyProps_Deleted as u16);
        return;
    }
    let flags = if dachora {
        kEnemyProps_DisableSamusColl as u16
    } else {
        (kEnemyProps_DisableSamusColl | kEnemyProps_Tangible) as u16 | 0x8000
    };
    slot!(game, Animal[e].base.properties, |old| old | flags);
    if dachora {
        slot!(game, Animal[e].base; spritemap_pointer = 0x804d, instruction_timer = 1, timer = 0, current_instruction = 0xe964);
    } else {
        slot!(game, Animal[e].base; instruction_timer = 1, timer = 0, palette_index = 0);
        let index = (slot!(game, Animal[e].parameter_1) >> 1) as isize;
        let etecoon = game.escape_etecoon(index);
        slot!(game, Animal[e].base.x_pos = etecoon.x_position);
        slot!(game, Animal[e].base.y_pos = etecoon.y_position);
        slot!(game, Animal[e].var_F = etecoon.function);
        slot!(
            game,
            Animal[e].base.current_instruction = etecoon.instruction_list
        );
        slot!(game, Animal[e].var_A = etecoon.x_velocity);
    }
}
pub fn state_address(game: &mut (impl Host + ?Sized)) -> u32 {
    let e = i32::from(word!(game.cur_enemy_index));
    0xb30000 | u32::from(slot!(game, Animal[e].var_F))
}
pub fn run_right(k: u16, game: &mut (impl Host + ?Sized)) {
    let e = i32::from(k);
    slot!(game, Animal[e].base(x_pos, x_subpos) += 0x38000);
}
pub fn await_rescue(k: u16, game: &mut (impl Host + ?Sized)) {
    if game.event_happened(0xf) {
        slot!(game, Animal[i32::from(k)].base.current_instruction = 0xe5da);
    }
}
pub fn walk(k: u16, game: &mut (impl Host + ?Sized)) {
    let e = i32::from(k);
    let amount = i32::from(slot!(game, Animal[e].var_A) as i16) << 8;
    let blocked = game.move_horizontal_ignoring_slopes(k, amount);
    if blocked {
        slot!(game, Animal[e].base.instruction_timer = 1);
        let left = (-i32::from(slot!(game, Animal[e].var_A))) & 0x8000 != 0;
        let reversed = (-i32::from(slot!(game, Animal[e].var_A))) as u16;
        slot!(game, Animal[e].var_A = reversed);
        let instruction = if left { 0xe556 } else { 0xe582 };
        slot!(game, Animal[e].base.current_instruction = instruction);
        if game.event_happened(0xf) {
            slot!(game, Animal[e].base.current_instruction = 0xe5ae);
        }
    }
    game.move_vertical(k, 1 << 16);
}
pub fn branch(
    check_rescue: bool,
    script: EnemyScript,
    game: &mut (impl Host + ?Sized),
) -> EnemyScript {
    let jump = if check_rescue {
        game.event_happened(0xf)
    } else {
        word!(game.lava_acid_y_pos) < 0xce
    };
    if jump {
        EnemyScript::jump(script.word(0))
    } else {
        script.offset(1)
    }
}
pub fn move_scripted(k: u16, script: EnemyScript, game: &mut (impl Host + ?Sized)) -> EnemyScript {
    let e = i32::from(k);
    slot!(game, Animal[e].base.x_pos, |old| old
        .wrapping_add(script.word(0)));
    script.offset(1)
}
pub fn step_dachora(
    amount: u16,
    script: EnemyScript,
    game: &mut (impl Host + ?Sized),
) -> EnemyScript {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Animal[e].base.x_pos, |old| old.wrapping_add(amount));
    script
}
