//! Fune fire after a delay or when Samus enters their vertical range.
//! The saved animation word remains RAM-authoritative.
#![forbid(unsafe_code)]
use crate::game::actors::api::{self, Axis};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use sm_game::enemy_script::EnemyScript;
use {
    crate::game::enemy_projectiles::SpawnEprojWithGfx, sm_game::enemy_projectiles::host::Effects,
};

fn selected(runtime: &Runtime<'_>) -> Key {
    Key::from_legacy(runtime.word(0xe54))
}
fn animate(runtime: &mut Runtime<'_>) {
    let key = selected(runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction_delay = 1;
        actor.timer = 0;
    });
    let table = runtime.resolve_actor(key, Native).variables[0];
    let instruction = runtime.legacy_native_record(0xa8, table).word(0);
    runtime.edit_actor(key, Native, |actor| actor.instruction = instruction);
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[0] = 0x96d7;
        actor.variables[1] = 0x9737;
        actor.variables[3] = actor.parameters[0] & 0xf;
        if actor.variables[3] != 0 {
            actor.variables[0] = 0x96df;
            actor.variables[1] = 0x975c;
        }
        if actor.parameters[0] & 0xf0 != 0 {
            actor.variables[0] = actor.variables[0].wrapping_add(2);
        }
    });
    animate(&mut runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[2] = u16::from(actor.parameters[1].to_ne_bytes()[1]);
        actor.pre_instruction = u16::from(actor.parameters[0].to_ne_bytes()[1]);
        actor.variables[4] = 0;
    });
}
pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    let state = 0xa80000 | u32::from(runtime.resolve_actor(key, Native).variables[1]);
    match state {
        0xa89737 => {
            let ready = runtime.edit_actor(key, Native, |actor| {
                actor.variables[4] = actor.variables[4].wrapping_add(1);
                (actor.variables[4] as i16).wrapping_sub(actor.pre_instruction as i16) >= 0
            });
            if ready {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[0] = actor.variables[0].wrapping_sub(4)
                });
                animate(&mut runtime);
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[1] = 0x978e;
                    actor.variables[4] = 0;
                });
            }
        }
        0xa8975c => {
            let saved = 0x7800 + usize::from(key.byte_index());
            runtime.write_slice_native(saved, runtime.resolve_actor(key, Native).variables[0]);
            let live = selected(&runtime);
            let range = runtime.resolve_actor(live, Native).variables[2];
            if api::player_within(&mut runtime, live, Axis::Y, range) {
                let table = runtime.native_word(saved);
                runtime.edit_actor(key, Native, |actor| actor.variables[0] = table);
                let table = table.wrapping_sub(4);
                runtime.write_slice_native(saved, table);
                runtime.edit_actor(key, Native, |actor| actor.variables[0] = table);
                animate(&mut runtime);
                runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0x978f);
            }
            let table = runtime.native_word(saved);
            runtime.edit_actor(key, Native, |actor| actor.variables[0] = table);
        }
        0xa8978e | 0xa8978f => {}
        _ => runtime.unreachable(),
    }
}
pub(crate) fn instruction(address: u32, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    match address {
        11048485 => runtime.queue_sound(2, 9, 0x1f),
        11048497 | 11048522 | 11048547 | 11048572 => {
            let key = selected(&runtime);
            let parameter = runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[0];
            runtime.write_slice_le(0x1995, u16::from(parameter));
            let actor = runtime.word(0xe54);
            let variant = u16::from(matches!(address, 11048522 | 11048572));
            let definition = if address < 11048547 { 0xdfbc } else { 0xdfca };
            runtime.foreign(|game| SpawnEprojWithGfx(variant, actor, definition, game));
        }
        11048597 | 11048628 => {
            let key = selected(&runtime);
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[0] = actor.variables[0].wrapping_add(4);
                actor.variables[1] = 0x9737;
                if actor.variables[3] != 0 {
                    actor.variables[1] = 0x975c;
                }
            });
        }
        _ => {
            runtime.unreachable();
            return EnemyScript::Stop;
        }
    }
    script
}
