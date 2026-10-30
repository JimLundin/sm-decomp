//! Lavaquake source actor: proximity, one randomized rock, then cooldown.
#![forbid(unsafe_code)]
use crate::game::actors::api::{self, Axis};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::words as W;
use sm_game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use {
    crate::game::enemy_projectiles::spawn_with_enemy_gfx, sm_game::enemy_projectiles::host::Effects,
};

pub(crate) fn update(initialize: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(runtime.word(W::CUR_ENEMY_INDEX));
    if initialize {
        runtime.edit_actor(key, Native, |actor| {
            actor.restart_animation(0xb51a);
            actor.variables[0] = 0xb596;
        });
        runtime.write_word_le(17, W::RANDOM_NUMBER);
        return;
    }
    let phase = runtime.resolve_actor(key, Native).variables[0];
    match phase {
        0xb596 => {
            for axis in [Axis::X, Axis::Y] {
                let live = Key::from_legacy(runtime.word(W::CUR_ENEMY_INDEX));
                if !api::player_within(&mut runtime, live, axis, 0x40) {
                    return;
                }
            }
            let live = Key::from_legacy(runtime.word(W::CUR_ENEMY_INDEX));
            runtime.edit_actor(live, Native, |actor| actor.variables[0] = 0xb5b2);
        }
        0xb5b2 => {
            // Loaded images precede the first and third random draws, but not the second.
            let image = runtime.loaded_image().expect("ROM must be loaded");
            let index = (runtime.next_random() & 0x1e) >> 1;
            let position = image.table_word(0x113550, index as isize);
            runtime.write_word_le(position, W::CINEMATIC_Y_SUB);
            let index = (runtime.next_random() & 0x1e) >> 1;
            let image = runtime.loaded_image().expect("ROM must be loaded");
            let parameter = image.table_word(0x113530, index as isize);
            let live = runtime.word(W::CUR_ENEMY_INDEX);
            runtime.foreign(|game| spawn_with_enemy_gfx(parameter, live, 0xbd5a, game));
            let live = Key::from_legacy(runtime.word(W::CUR_ENEMY_INDEX));
            runtime.edit_actor(live, Native, |actor| actor.variables[0] = 0xb5ea);
            let image = runtime.loaded_image().expect("ROM must be loaded");
            let index = (runtime.next_random() & 0xe) >> 1;
            let delay = image.table_word(0x113520, index as isize);
            runtime.edit_actor(live, Native, |actor| actor.variables[1] = delay);
        }
        0xb5ea => runtime.edit_actor(key, Native, |actor| {
            actor.variables[1] = actor.variables[1].wrapping_sub(1);
            if actor.variables[1] & 0x8000 != 0 {
                actor.variables[0] = 0xb596;
            }
        }),
        _ => runtime.unreachable(),
    }
}
