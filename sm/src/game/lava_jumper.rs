//! The lava jumper's flight phases share fractional movement and acceleration.
#![forbid(unsafe_code)]
use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::game::norfair_rio::animate_cached;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

pub(crate) enum Flight {
    Rising,
    Turning,
    Falling,
}

pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let raw = runtime.word(0xe54);
    runtime.write_slice_native(0x7800 + usize::from(raw), 0);
    runtime.write_slice_native(0x7802 + usize::from(raw), 0);
    runtime.edit_actor(Key::from_legacy(raw), Native, |actor| {
        if actor.parameters[0] & 0x8000 != 0 {
            actor.instruction = 0xbe62;
            actor.pre_instruction = 0xbedc;
        } else {
            actor.variables[3] = actor.x.whole;
            actor.variables[4] = actor.y.whole;
            actor.instruction = 0xbe3c;
            actor.pre_instruction = 0xbf1a;
        }
    });
}

pub(crate) fn instruction_finished(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let at = 0x7800 + usize::from(runtime.word(0xe54));
    runtime.write_slice_native(at, 1);
}

pub(crate) fn follow_parent(k: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(k);
    let offset = (usize::from(k) >> 1) * 2;
    if runtime.word(0xe84 + offset + 200) == 0 {
        runtime.edit_actor(key, Native, |actor| actor.properties |= 0x200);
        return;
    }
    let frozen = runtime.word(0xe84 + offset + 218);
    runtime.edit_actor(key, Native, |actor| actor.frozen = frozen);
    if frozen != 0 || runtime.word(0xf68 + offset + 4) & 0x8000 == 0 {
        runtime.edit_actor(key, Native, |actor| actor.properties |= 0x100);
    } else {
        runtime.edit_actor(key, Native, |actor| actor.properties &= !0x100);
        let y = runtime.word(0xe84 + offset + 186);
        runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
    }
}

pub(crate) fn launch(k: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let index = isize::from((runtime.byte(0x5e6) & 6) >> 1);
    let speed = runtime
        .loaded_image()
        .expect("ROM must be loaded")
        .table_word(crate::rom::lorom_offset(0xa2be86) as usize, index);
    runtime.edit_actor(Key::from_legacy(k), Native, |actor| {
        actor.variables[2] = speed;
        actor.pre_instruction = 0xbf3e;
        actor.properties |= 0x800;
    });
    drop(runtime);
    crate::game::music::QueueSfx2_Max6(0xd, game);
}

pub(crate) fn advance(k: u16, phase: Flight, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(k);
    runtime.edit_actor(key, Native, |actor| {
        actor
            .y
            .advance_8_8(i16::from_le_bytes(actor.variables[2].to_ne_bytes()));
    });
    if matches!(phase, Flight::Falling) && runtime.resolve_actor(key, Native).y.whole & 0xf0 == 0xf0
    {
        runtime.edit_actor(key, Native, |actor| {
            actor.x.whole = actor.variables[3];
            actor.y.whole = actor.variables[4];
        });
        animate_cached(&mut runtime, 0xbe3c, 2);
        runtime.edit_actor(key, Native, |actor| {
            actor.pre_instruction = 0xbf1a;
            actor.properties &= !0x800;
        });
        return;
    }
    let speed = runtime.edit_actor(key, Native, |actor| {
        actor.variables[2] = actor.variables[2].wrapping_add(56);
        actor.variables[2]
    });
    if matches!(phase, Flight::Rising) && speed >= 0xfc00 {
        animate_cached(&mut runtime, 0xbe42, 2);
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xbf7c);
    } else if matches!(phase, Flight::Turning) {
        let finished = 0x7800 + usize::from(k);
        if runtime.native_word(finished) != 0 {
            runtime.write_slice_native(finished, 0);
            runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xbfbc);
        }
    }
}
