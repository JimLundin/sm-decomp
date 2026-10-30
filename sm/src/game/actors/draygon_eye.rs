//! Draygon eye tracking shares one facing-dependent animation rule.
//! Common actor fields use the existing owner; outer boss dispatch stays in bank A5.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use super::api::angle_from_delta;
use crate::game::enemy_projectiles::SpawnEprojWithRoomGfx;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::words;
use crate::types::Point16U;
use sm_game::enemy_script::EnemyScript;

pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(runtime.word(words::cur_enemy_index));
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = 0x9944;
        actor.variables[0] = 0x804b;
    });
}
pub(crate) fn set_mode(k: u16, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let mode = script.word(0);
    Runtime::new(game).edit_actor(Key::from_legacy(k), Native, |actor| {
        actor.variables[0] = mode
    });
    script.offset(1)
}
pub(crate) fn advance(k: u16, right: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let x_offset = if right { 24 } else { 0u16.wrapping_sub(24) };
    if runtime.resolve_actor(Key::from_legacy(k), Native).frame & 0x7f == 0 {
        // The left-facing emitter follows the eye; the right-facing one follows
        // the body. Both steering calculations below use the body's live position.
        let point = {
            let origin =
                runtime.resolve_actor(Key::from_legacy(if right { 0 } else { 0x40 }), Native);
            Point16U {
                x: origin.x.whole.wrapping_add(x_offset),
                y: origin.y.whole.wrapping_sub(32),
            }
        };
        runtime.foreign(|game| {
            game.eproj_spawn_pt = point;
            SpawnEprojWithRoomGfx(0xe509, 0x18, game);
        });
    }
    let (x, y) = {
        let body = runtime.resolve_actor(Key::from_legacy(0), Native);
        let x = runtime.word(words::samus_x_pos).wrapping_sub(body.x.whole);
        let y = runtime.word(words::samus_y_pos).wrapping_sub(body.y.whole);
        (x.wrapping_sub(x_offset), y.wrapping_add(32))
    };
    let angle = angle_from_delta(x, y);
    // The caller's k selects the emission timer; the post-callback live selector
    // chooses the eye animation and its previous-angle cache.
    let key = Key::from_legacy(runtime.word(words::cur_enemy_index));
    if angle != runtime.resolve_actor(key, Native).pre_instruction {
        let instructions = if right {
            [0x9d5c, 0x9d50, 0x9d62, 0x9d56]
        } else {
            [0x99ba, 0x99b4, 0x99c0, 0x99ae]
        };
        let sector = usize::from(angle.wrapping_add(32) & 0xff) / 64;
        runtime.edit_actor(key, Native, |actor| {
            actor.pre_instruction = angle;
            actor.restart_animation(instructions[sector]);
        });
    }
}
