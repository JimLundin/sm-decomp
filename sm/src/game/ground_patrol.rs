//! Spike plants and spikey shells share velocity setup and bounded patrol.
#![forbid(unsafe_code)]
use crate::game::actors::{api::ShotHandling, combat};
use crate::game::enemy_speeds::SpeedTable;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use sm_game::enemy_projectiles::host::Effects;

fn selected(runtime: &Runtime<'_>) -> Key {
    Key::from_legacy(runtime.word(0xe54))
}
fn table(runtime: &Runtime<'_>, address: u32, index: usize) -> u16 {
    runtime
        .loaded_image()
        .expect("ROM must be loaded")
        .table_word(crate::rom::lorom_offset(address) as usize, index as isize)
}
fn animate_plant(runtime: &mut Runtime<'_>, firing: bool) {
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
    animate_plant(&mut runtime, false);
    let direction = runtime.resolve_actor(key, Native).parameters[0].to_ne_bytes()[0];
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[4] = u16::from(direction)
    });
    let phase = table(&runtime, 0xa29f42, usize::from(direction));
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = phase);
    let range = runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[0];
    let at = 0x7800 + usize::from(runtime.word(0xe54));
    initialize_bounds(&mut runtime, key, at, 0xa29f36, usize::from(range));
    cache_velocity(&mut runtime, key, 1);
}
fn initialize_bounds(runtime: &mut Runtime<'_>, key: Key, at: usize, rom: u32, range: usize) {
    let distance = table(runtime, rom, range);
    let x = runtime.resolve_actor(key, Native).x.whole;
    runtime.write_slice_native(at + 2, distance.wrapping_add(x));
    let x = runtime.resolve_actor(key, Native).x.whole;
    let distance = table(runtime, rom, range);
    runtime.write_slice_native(at, x.wrapping_sub(distance));
}
fn cache_velocity(runtime: &mut Runtime<'_>, key: Key, physical_byte: usize) {
    let speed =
        usize::from(runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[physical_byte])
            * 4;
    for (entry, variable) in [1, 0, 3, 2].into_iter().enumerate() {
        let value = runtime.enemy_speed(SpeedTable::Linear, (speed + entry) as isize);
        runtime.edit_actor(key, Native, |actor| actor.variables[variable] = value);
    }
}
pub(crate) fn advance_plant(right: bool, game: &mut Game) {
    patrol(false, right, &mut Runtime::new(game));
}
fn patrol(shell: bool, right: bool, runtime: &mut Runtime<'_>) {
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
    let chosen = shell.then(|| selected(runtime));
    runtime.next_random();
    let key = chosen.unwrap_or_else(|| selected(runtime));
    let chance = runtime.resolve_actor(key, Native).frame.to_ne_bytes()[0]
        .wrapping_add(runtime.word(0x5e5) as u8);
    if chance < if shell { 6 } else { 3 } {
        if shell {
            runtime.edit_actor(key, Native, |actor| actor.variables[4] = 3);
        } else {
            runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xa01b);
            animate_plant(runtime, true);
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

fn animate_shell(runtime: &mut Runtime<'_>, right: bool) {
    let key = selected(runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.restart_animation(if right { 0xa3bd } else { 0xa3ab });
    });
}
pub(crate) fn initialize_shell(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let raw = runtime.word(0xe54);
    let key = Key::from_legacy(raw);
    runtime.edit_actor(key, Native, |actor| actor.spritemap = 0x804d);
    let direction = runtime.resolve_actor(key, Native).parameters[0].to_ne_bytes()[0];
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[4] = u16::from(direction)
    });
    animate_shell(&mut runtime, direction & 1 != 0);
    cache_velocity(&mut runtime, key, 0);
    let range = usize::from(runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[1]);
    let at = 0x7800 + usize::from(raw);
    initialize_bounds(&mut runtime, key, at + 2, 0xa2a3dd, range);
    reload_shell_delay(&mut runtime, key, at);
    runtime.edit_actor(key, Native, |actor| {
        actor.pre_instruction = 0;
        if actor.variables[4] == 2 {
            actor.pre_instruction = 16;
            actor.y.whole = actor.y.whole.wrapping_add(16);
        }
    });
}
fn reload_shell_delay(runtime: &mut Runtime<'_>, key: Key, at: usize) {
    let delay = usize::from(runtime.resolve_actor(key, Native).parameters[0].to_ne_bytes()[1]);
    let delay = table(runtime, 0xa2a3ed, delay);
    runtime.write_slice_native(at, delay);
}
#[derive(Clone, Copy)]
enum ShellPhase {
    Animate(bool),
    Patrol(bool),
    Wait,
    Sink,
    Rise,
}
pub(crate) fn advance_shell(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let raw = runtime.word(0xe54);
    let key = Key::from_legacy(raw);
    let phase = runtime.resolve_actor(key, Native).variables[4] as i16;
    // Preserve signed indexing and the invalid-index panic before any effect.
    let phase = [
        ShellPhase::Animate(false),
        ShellPhase::Animate(true),
        ShellPhase::Patrol(false),
        ShellPhase::Patrol(true),
        ShellPhase::Wait,
        ShellPhase::Sink,
        ShellPhase::Rise,
    ][(i32::from(phase) + 2) as usize];
    let at = 0x7800 + usize::from(raw);
    match phase {
        ShellPhase::Animate(right) => animate_shell(&mut runtime, right),
        ShellPhase::Patrol(right) => patrol(true, right, &mut runtime),
        ShellPhase::Wait => {
            let delay = runtime.native_word(at).wrapping_sub(1);
            runtime.write_slice_native(at, delay);
            if delay == 0 {
                runtime.edit_actor(key, Native, |actor| actor.variables[4] = 4);
            }
        }
        ShellPhase::Sink | ShellPhase::Rise => {
            let sinking = matches!(phase, ShellPhase::Sink);
            let done = runtime.edit_actor(key, Native, |actor| {
                let step = if sinking { 1 } else { u16::MAX };
                actor.y.whole = actor.y.whole.wrapping_add(step);
                let previous = actor.pre_instruction;
                actor.pre_instruction = previous.wrapping_add(step);
                if sinking {
                    actor.pre_instruction.wrapping_sub(16) & 0x8000 == 0
                } else {
                    previous == 1
                }
            });
            if done {
                let direction = if sinking {
                    2
                } else {
                    u16::from(runtime.byte(0x5e6)) & 1
                };
                runtime.edit_actor(key, Native, |actor| actor.variables[4] = direction);
                if sinking {
                    reload_shell_delay(&mut runtime, key, at);
                }
            }
        }
    }
}
pub(crate) fn direct_shell(k: u16, direction: u16, game: &mut Game) {
    Runtime::new(game).edit_actor(Key::from_legacy(k), Native, |actor| {
        actor.variables[4] = direction
    });
}
pub(crate) fn shot_shell(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    if runtime.resolve_actor(key, Native).variables[4].wrapping_sub(1) & 0x8000 != 0 {
        combat::shot_reaction(&mut runtime, ShotHandling::Finalize);
    }
}
