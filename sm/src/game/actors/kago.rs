//! Kago damage releases a bug after updating its animation and death state.
//! The extended hit counter remains RAM-authoritative for native consumers.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use super::{api, combat};
use crate::game::enemy_projectiles::{SpawnEprojWithGfx, host::Effects};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let index = runtime.word(0xe54);
    let key = Key::from_legacy(index);
    runtime.edit_actor(key, Native, |actor| {
        actor.properties |= 0x2000;
        actor.instruction_delay = 1;
        actor.timer = 0;
        actor.variables[4] = 0;
        actor.instruction = 0xab1e;
        actor.variables[0] = 0xab7b;
        actor.pre_instruction = 0;
    });
    let hits = runtime.resolve_actor(key, Native).parameters[0];
    runtime.write_slice_native(0x7808 + usize::from(index), hits);
}

pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(runtime.word(0xe54));
    match runtime.resolve_actor(key, Native).variables[0] {
        0xab7b => runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0xab81),
        0xab81 => (),
        _ => runtime.unreachable(),
    }
}

pub(crate) fn shot(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    combat::shot_reaction(&mut runtime, api::ShotHandling::Finalize);
    runtime.write_slice_le(0x183e, 2);
    runtime.write_slice_le(0x1840, 16);
    // Damage may select another actor. Retain that identity through death,
    // then use the live selection only for the projectile's graphics source.
    let index = runtime.word(0xe54);
    let key = Key::from_legacy(index);
    if runtime.resolve_actor(key, Native).variables[1] == 0 {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[1] = 1;
            actor.instruction = 0xab32;
            actor.instruction_delay = 1;
        });
    }
    let counter = 0x7808 + usize::from(index);
    let hits = runtime.native_word(counter).wrapping_sub(1);
    runtime.write_slice_native(counter, hits);
    if (hits as i16) < 0 {
        combat::die(&mut runtime, api::DeathKind::Normal { animation: 4 });
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 1);
    }
    let y = runtime.resolve_actor(key, Native).y.whole;
    let current = runtime.word(0xe54);
    runtime.foreign(|game| SpawnEprojWithGfx(y, current, 0xd02e, game));
}
