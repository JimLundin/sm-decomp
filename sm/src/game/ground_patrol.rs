//! Bounded patrol shared by spike plants and Maridia spikey shells.
#![forbid(unsafe_code)]
use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::game::enemy_projectiles::host::Effects;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

pub(crate) enum Kind {
    Plant,
    Shell,
}

fn selected(runtime: &Runtime<'_>) -> Key {
    Key::from_legacy(runtime.word(0xe54))
}
fn table(runtime: &Runtime<'_>, address: u32, index: usize) -> u16 {
    runtime
        .loaded_image()
        .expect("ROM must be loaded")
        .table_word(crate::rom::lorom_offset(address) as usize, index as isize)
}
fn animate(runtime: &mut Runtime<'_>, firing: bool) {
    let key = selected(runtime);
    runtime.edit_actor(key, Native, |actor| {
        let inverted = actor.parameters[0].to_ne_bytes()[1] != 0;
        actor.restart_animation(
            [[0x9eda, 0x9f00], [0x9e8a, 0x9eb0]][usize::from(inverted)][usize::from(firing)],
        );
    });
}
pub(crate) fn initialize_plant(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    runtime.edit_actor(key, Native, |actor| actor.spritemap = 0x804d);
    animate(&mut runtime, false);
    let direction = runtime.resolve_actor(key, Native).parameters[0].to_ne_bytes()[0];
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[4] = u16::from(direction)
    });
    let phase = table(&runtime, 0xa29f42, usize::from(direction));
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = phase);
    let range = runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[0];
    let distance = table(&runtime, 0xa29f36, usize::from(range));
    let x = runtime.resolve_actor(key, Native).x.whole;
    let at = 0x7800 + usize::from(runtime.word(0xe54));
    runtime.write_slice_native(at + 2, distance.wrapping_add(x));
    let x = runtime.resolve_actor(key, Native).x.whole;
    let distance = table(&runtime, 0xa29f36, usize::from(range));
    runtime.write_slice_native(at, x.wrapping_sub(distance));
    let speed = usize::from(runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[1]) * 4;
    for (entry, variable) in [1, 0, 3, 2].into_iter().enumerate() {
        let value = table(&runtime, 0xa28187, speed + entry);
        runtime.edit_actor(key, Native, |actor| actor.variables[variable] = value);
    }
}
pub(crate) fn advance(kind: Kind, right: bool, game: &mut Game) {
    let shell = matches!(kind, Kind::Shell);
    let mut runtime = Runtime::new(game);
    let raw = runtime.word(0xe54);
    let key = Key::from_legacy(raw);
    let x = runtime.edit_actor(key, Native, |actor| {
        let low = if right { 0 } else { 2 };
        let speed = (u32::from(actor.variables[low + 1]) << 16) | u32::from(actor.variables[low]);
        actor.x.advance(speed as i32);
        actor.x.whole
    });
    let bound_offset = 2 * (usize::from(right) + usize::from(shell));
    let bound = runtime.native_word(0x7800 + usize::from(raw) + bound_offset);
    if (x.wrapping_sub(bound) as i16 >= 0) == right {
        runtime.edit_actor(key, Native, |actor| {
            if shell {
                actor.variables[4] = if right {
                    0
                } else {
                    actor.variables[4].wrapping_sub(1)
                };
            } else {
                actor.pre_instruction = if right { 0x9fba } else { 0x9fec };
                actor.variables[4] = u16::from(!right);
            }
        });
    }
    // Shell captures its RNG target before the effect; Plant selects afterward.
    let chosen = shell.then(|| selected(&runtime));
    runtime.next_random();
    let key = chosen.unwrap_or_else(|| selected(&runtime));
    let chance = runtime.resolve_actor(key, Native).frame.to_ne_bytes()[0]
        .wrapping_add(runtime.word(0x5e5) as u8);
    if chance < if shell { 6 } else { 3 } {
        if shell {
            runtime.edit_actor(key, Native, |actor| actor.variables[4] = 3);
        } else {
            runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xa01b);
            animate(&mut runtime, true);
        }
    }
}
pub(crate) fn resume_plant(k: u16, game: &mut Game) {
    Runtime::new(game).edit_actor(Key::from_legacy(k), Native, |actor| {
        actor.pre_instruction = if actor.variables[4] != 0 {
            0x9fec
        } else {
            0x9fba
        };
    });
}
