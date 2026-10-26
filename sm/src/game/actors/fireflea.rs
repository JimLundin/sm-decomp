//! Fireflea motion and combat preserve native lanes and callback intervals.
#![forbid(unsafe_code)]
use super::{
    api::{DeathHandling, DeathKind, ShotHandling},
    combat,
};
use crate::game::enemy_layouts::EnemyLayout40 as Flea;
use crate::game::session_runtime::Runtime;
use crate::game::session_runtime::{actor_cosine, actor_sine};
use crate::game::visual_runtime::darken_fireflea_room;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rtl::rom_table_word;

fn phase<const INITIAL: bool>(key: i32, game: &Game) -> u16 {
    if INITIAL {
        slot!(game, Flea[key].var_D)
    } else {
        u16::from(slot!(byte game, Flea[key].var_D[1]))
    }
}
fn orbit<const INITIAL: bool>(key: i32, game: &mut Game) {
    let x = slot!(game, Flea[key].var_E).wrapping_add(actor_cosine(
        phase::<INITIAL>(key, game),
        slot!(game, Flea[key].var_C),
        game,
    ));
    slot!(game, Flea[key].base.x_pos = x);
    let y = slot!(game, Flea[key].var_F).wrapping_add(actor_sine(
        phase::<INITIAL>(key, game),
        slot!(game, Flea[key].var_C),
        game,
    ));
    slot!(game, Flea[key].base.y_pos = y);
}
fn speed(key: i32, game: &mut Game) {
    let mut index = 8 * u16::from(slot!(byte game, Flea[key].parameter_2[0]));
    if slot!(game, Flea[key].parameter_1) & 1 == 0 {
        index += 4;
    }
    slot!(game, Flea[key].var_02 = index);
    let index = (index >> 1) as isize;
    let high = rom_table_word!(game, 0xa28187, index);
    slot!(game, Flea[key].var_B = high);
    let low = rom_table_word!(game, 0xa28187, index + 1);
    slot!(game, Flea[key].var_A = low);
}
fn radius(key: i32, game: &mut Game) {
    let image = game.capture_native_rom();
    let index = usize::from(slot!(byte game, Flea[key].parameter_2[1]));
    let radius = image.read(crate::rtl::lorom_offset(0xa38d1d) as usize + 2 * index);
    slot!(game, Flea[key].var_C = u16::from(radius));
}
pub(crate) fn initialize(game: &mut Game) {
    let key = i32::from(word!(game.cur_enemy_index));
    slot!(game, Flea[key].base.current_instruction = 0x8c2f);
    let orbiting = slot!(game, Flea[key].parameter_1) & 2 != 0;
    if orbiting {
        let key = i32::from(word!(game.cur_enemy_index));
        slot!(game, Flea[key].var_E = slot!(game, Flea[key].base.x_pos));
        slot!(game, Flea[key].var_F = slot!(game, Flea[key].base.y_pos));
        let key = i32::from(word!(game.cur_enemy_index));
        let phase = u16::from(slot!(byte game, Flea[key].parameter_1[1]));
        slot!(game, Flea[key].var_D = phase << 8);
    }
    speed(i32::from(word!(game.cur_enemy_index)), game);
    radius(i32::from(word!(game.cur_enemy_index)), game);
    if orbiting {
        orbit::<true>(i32::from(word!(game.cur_enemy_index)), game);
    } else {
        let key = i32::from(word!(game.cur_enemy_index));
        let low = slot!(game, Flea[key].base.y_pos).wrapping_sub(slot!(game, Flea[key].var_C));
        slot!(game, Flea[key].var_00 = low);
        let high = slot!(game, Flea[key].var_C).wrapping_add(slot!(game, Flea[key].base.y_pos));
        slot!(game, Flea[key].var_01 = high);
    }
}
pub(crate) fn advance(game: &mut Game) {
    let key = i32::from(word!(game.cur_enemy_index));
    if slot!(game, Flea[key].parameter_1) & 2 != 0 {
        orbit::<false>(key, game);
        let step = u16::from_ne_bytes([
            slot!(byte game, Flea[key].var_A[1]),
            slot!(byte game, Flea[key].var_A[2]),
        ]);
        slot!(game, Flea[key].var_D, |old| old.wrapping_add(step));
    } else {
        let index = (slot!(game, Flea[key].var_02) >> 1) as isize;
        let step = (u32::from(rom_table_word!(game, 0xa28187, index)) << 16)
            | u32::from(rom_table_word!(game, 0xa28187, index + 1));
        slot!(game, Flea[key].base(y_pos, y_subpos) += step);
        if (slot!(game, Flea[key].base.y_pos) as i16)
            .wrapping_sub(slot!(game, Flea[key].var_00) as i16)
            < 0
            || (slot!(game, Flea[key].base.y_pos) as i16)
                .wrapping_sub(slot!(game, Flea[key].var_01) as i16)
                >= 0
        {
            slot!(game, Flea[key].var_02, |old| old ^ 4);
        }
    }
}

pub(crate) fn touch(game: &mut Game) {
    combat::contact_reaction(&mut Runtime::new(game), DeathHandling::Finalize);
    combat::die(&mut Runtime::new(game), DeathKind::Normal { animation: 0 });
    darken_fireflea_room(game);
}
fn darken_if_dead(game: &mut Game) {
    let key = i32::from(word!(game.cur_enemy_index));
    if slot!(game, Flea[key].base.health) == 0 {
        darken_fireflea_room(game);
    }
}
pub(crate) fn power_bomb(game: &mut Game) {
    combat::power_bomb_reaction(&mut Runtime::new(game), DeathHandling::Finalize);
    darken_if_dead(game);
}
pub(crate) fn shot(game: &mut Game) {
    combat::shot_reaction(&mut Runtime::new(game), ShotHandling::Finalize);
    darken_if_dead(game);
}
