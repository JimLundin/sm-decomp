//! Escape-room Etecoons and Dachora: setup, wall turns and script conditions.
#![forbid(unsafe_code)]
use super::{access::ActorKey as Key, api};
use crate::game::constants::enemy_properties::*;
use crate::game::enemy_layouts::EnemyLayout8 as Animal;
use crate::game::gameplay_flag_runtime::CheckEventHappened;
use crate::game::imported::{Imported, load};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rom::RomImage;
use sm_data::escape_etecoons::{ADDRESS, BYTES, EscapeEtecoon, EscapeEtecoons};
use sm_game::enemy_script::EnemyScript;

pub(crate) type ImportedTables = Option<Imported<EscapeEtecoons>>;

pub(crate) fn import(image: &RomImage) -> ImportedTables {
    Imported::import(image, ADDRESS, BYTES, EscapeEtecoons::import)
}

/// Etecoon `index`'s initial state; other images and indices run the five
/// original indexed loads.
fn etecoon(game: &Game, index: isize) -> EscapeEtecoon {
    Imported::read(
        &game.escape_etecoons,
        game,
        |table| table.etecoon(usize::try_from(index).ok()?),
        |rom| EscapeEtecoon {
            x_position: load(rom, 0xb3e718, index),
            y_position: load(rom, 0xb3e71e, index),
            function: load(rom, 0xb3e724, index),
            instruction_list: load(rom, 0xb3e72a, index),
            x_velocity: load(rom, 0xb3e730, index),
        },
    )
}

pub(crate) fn initialize(dachora: bool, game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    if CheckEventHappened(0xf, game) != 0 {
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
        let etecoon = etecoon(game, index);
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
pub(crate) fn state_address(game: &Game) -> u32 {
    let e = i32::from(word!(game.cur_enemy_index));
    0xb30000 | u32::from(slot!(game, Animal[e].var_F))
}
pub(crate) fn run_right(k: u16, game: &mut Game) {
    let e = i32::from(k);
    slot!(game, Animal[e].base(x_pos, x_subpos) += 0x38000);
}
pub(crate) fn await_rescue(k: u16, game: &mut Game) {
    if CheckEventHappened(0xf, game) != 0 {
        slot!(game, Animal[i32::from(k)].base.current_instruction = 0xe5da);
    }
}
pub(crate) fn walk(k: u16, game: &mut Game) {
    let e = i32::from(k);
    let amount = i32::from(slot!(game, Animal[e].var_A) as i16) << 8;
    let blocked = api::move_horizontal(
        &mut Runtime::new(game),
        Key::from_legacy(k),
        amount,
        api::SlopePolicy::Ignore,
    ) == api::MovementResult::Blocked;
    if blocked {
        slot!(game, Animal[e].base.instruction_timer = 1);
        let left = (-i32::from(slot!(game, Animal[e].var_A))) & 0x8000 != 0;
        let reversed = (-i32::from(slot!(game, Animal[e].var_A))) as u16;
        slot!(game, Animal[e].var_A = reversed);
        let instruction = if left { 0xe556 } else { 0xe582 };
        slot!(game, Animal[e].base.current_instruction = instruction);
        if CheckEventHappened(0xf, game) != 0 {
            slot!(game, Animal[e].base.current_instruction = 0xe5ae);
        }
    }
    api::move_vertical(&mut Runtime::new(game), Key::from_legacy(k), 1 << 16);
}
pub(crate) fn branch(check_rescue: bool, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let jump = if check_rescue {
        CheckEventHappened(0xf, game) != 0
    } else {
        word!(game.lava_acid_y_pos) < 0xce
    };
    if jump {
        EnemyScript::jump(script.word(0))
    } else {
        script.offset(1)
    }
}
pub(crate) fn move_scripted(k: u16, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let e = i32::from(k);
    slot!(game, Animal[e].base.x_pos, |old| old
        .wrapping_add(script.word(0)));
    script.offset(1)
}
pub(crate) fn step_dachora(amount: u16, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Animal[e].base.x_pos, |old| old.wrapping_add(amount));
    script
}
