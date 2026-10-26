//! Falling and fast sinking platforms share travel, rider carry and animation.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use super::{api, combat};
use crate::game::enemy_script::EnemyScript;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

fn extra(key: Key) -> usize {
    0x7800 + usize::from(key.byte_index())
}
fn table(runtime: &Runtime<'_>, address: u32, index: isize) -> u16 {
    runtime
        .loaded_image()
        .expect("ROM must be loaded")
        .table_word(crate::rom::lorom_offset(address) as usize, index)
}
fn animate(runtime: &mut Runtime<'_>, falling: bool, right: bool) {
    let key = combat::current(runtime);
    let special = runtime.native_word(extra(key) + 62) & 0x8000 != 0;
    let instruction = [[0x9c3f, 0x9c13], [0x9be7, 0x9bbb]][usize::from(special)]
        [usize::from(falling)]
        + u16::from(right) * 0x16;
    runtime.edit_actor(key, Native, |actor| actor.restart_animation(instruction));
}
fn falling(runtime: &mut Runtime<'_>, value: bool) {
    let at = extra(combat::current(runtime));
    let (flag, opposite) = if value { (6, 8) } else { (8, 6) };
    if runtime.native_word(at + flag) == 0 {
        runtime.write_slice_native(at + flag, 1);
        let right = runtime.native_word(at + 4) != 0;
        animate(runtime, value, right);
    }
    runtime.write_slice_native(at + opposite, 0);
}
pub(crate) fn initialize(special: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    let at = extra(key);
    if special {
        runtime.write_slice_native(at + 62, 0xffff);
    }
    let direction = runtime.resolve_actor(key, Native).parameters[0];
    runtime.write_slice_native(at + 4, direction);
    // Initialization selects by kind; the fast variant retains its saved sign word.
    let instruction = if special { 0x9be7 } else { 0x9c3f } + if direction != 0 { 0x16 } else { 0 };
    runtime.edit_actor(key, Native, |actor| actor.instruction = instruction);
    let index = 4 * isize::from(runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[0]);
    for (offset, variable) in [2, 1, 4, 3].into_iter().enumerate() {
        let value = table(&runtime, 0xa28187, index + offset as isize);
        runtime.edit_actor(key, Native, |actor| actor.variables[variable] = value);
    }
    for offset in [0, 6, 8] {
        runtime.write_slice_native(at + offset, 0);
    }
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[0] = actor.y.whole.wrapping_add(1);
        actor.pre_instruction = 0;
    });
    let maximum = runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[1];
    runtime.write_slice_native(at + 10, u16::from(maximum));
}
fn accelerate(runtime: &mut Runtime<'_>, key: Key) -> u16 {
    runtime.edit_actor(key, Native, |actor| {
        actor.pre_instruction = actor.pre_instruction.wrapping_add(1)
    });
    let maximum = runtime.native_word(extra(key) + 10);
    runtime.edit_actor(key, Native, |actor| {
        if (actor.pre_instruction as i16).wrapping_sub(maximum as i16) >= 0 {
            actor.pre_instruction = maximum;
        }
        actor.pre_instruction
    })
}
fn horizontal(runtime: &mut Runtime<'_>, right: bool) {
    let key = combat::current(runtime);
    let at = extra(key);
    runtime.write_slice_native(at + 2, runtime.resolve_actor(key, Native).x.whole);
    let current = combat::current(runtime);
    let amount = {
        let actor = runtime.resolve_actor(key, Native);
        let low = if right { 1 } else { 3 };
        ((u32::from(actor.variables[low + 1]) << 16) | u32::from(actor.variables[low])) as i32
    };
    if api::move_horizontal(runtime, current, amount, api::SlopePolicy::Ignore)
        == api::MovementResult::Blocked
    {
        runtime.write_slice_native(at + 4, u16::from(!right));
        animate(runtime, false, !right);
    }
}
fn vertical(runtime: &mut Runtime<'_>, riding: bool) {
    let key = combat::current(runtime);
    let at = extra(key);
    let mut stopped = !riding && {
        let actor = runtime.resolve_actor(key, Native);
        (actor.y.whole as i16).wrapping_sub(actor.variables[0] as i16) < 0
    };
    if !stopped {
        if !riding {
            falling(runtime, true);
        }
        let step = accelerate(runtime, key);
        if riding {
            let delta = runtime
                .resolve_actor(key, Native)
                .x
                .whole
                .wrapping_sub(runtime.native_word(at + 2));
            let carry = &mut runtime.session_mut().player.contact.external_x.whole;
            *carry = carry.wrapping_add(delta);
            runtime.write_slice_native(at + 2, runtime.resolve_actor(key, Native).y.whole);
        }
        let current = combat::current(runtime);
        let index = 4 * step as isize + if riding { 0 } else { 2 };
        let high = table(runtime, 0xa2838f, index + 1);
        let low = table(runtime, 0xa2838f, index);
        let amount = ((u32::from(high) << 16) | u32::from(low)) as i32;
        stopped = api::move_vertical(runtime, current, amount) == api::MovementResult::Blocked;
    }
    if stopped {
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0);
        falling(runtime, false);
    }
    if riding {
        let delta = runtime
            .resolve_actor(key, Native)
            .y
            .whole
            .wrapping_sub(runtime.native_word(at + 2));
        let carry = &mut runtime.session_mut().player.contact.external_y.whole;
        *carry = carry.wrapping_add(delta);
    }
}
pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    let at = extra(key);
    runtime.write_slice_native(at, 0);
    if api::touches_player_from_above(&runtime, combat::current(&runtime)) {
        runtime.write_slice_native(at, 1);
    }
    // Full native dispatch indices keep both original two-entry bounds checks.
    let right = [false, true][usize::from(runtime.native_word(at + 4))];
    horizontal(&mut runtime, right);
    let current = combat::current(&runtime);
    let riding = [false, true][usize::from(runtime.native_word(extra(current)))];
    vertical(&mut runtime, riding);
    if runtime.native_word(at) != runtime.native_word(at + 12) {
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0);
    }
    runtime.write_slice_native(at + 12, runtime.native_word(at));
}
pub(crate) fn shot(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    combat::shot_reaction(&mut runtime, api::ShotHandling::Finalize);
    let key = combat::current(&runtime);
    if runtime.resolve_actor(key, Native).frozen != 0 {
        let right = runtime.native_word(extra(key) + 4) != 0;
        runtime.edit_actor(key, Native, |actor| {
            actor.spritemap = if right { 0xa015 } else { 0xa009 }
        });
    }
}
pub(crate) fn direction(right: bool, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let at = extra(combat::current(&runtime));
    runtime.write_slice_native(at + 4, u16::from(right));
    script
}
