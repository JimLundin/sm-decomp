//! Wrecked Ship robots share walking, laser timing and their palette animation.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use super::{api, combat};
use crate::game::enemy_projectiles::{SpawnEprojWithGfx, host::Effects};
use crate::game::{gameplay_flag_runtime, session_runtime::Runtime};
use crate::game_state::Game;
use crate::ram_map::{word_arrays as A, words as W};
use sm_game::enemy_script::EnemyScript;
use sm_game::gameplay_flag_host::BossWordOrder;

const PALETTE: [i16; 31] = [
    0x1f, 0x18, 0xf, 8, 0x40, 0x18, 0xf, 8, 0x1f, 0x10, 0xf, 8, 0x1f, 0x18, 0x10, 8, 0x1f, 0x18,
    0xf, 0x40, 0xf, 8, 0x1f, 0x18, 0x10, 0x18, 0xf, 8, 0x1f, 0x10, -1,
];
fn selected(runtime: &Runtime<'_>) -> Key {
    Key::from_legacy(runtime.word(W::cur_enemy_index))
}
fn powered(game: &mut Game) -> bool {
    gameplay_flag_runtime::boss_word_at(game.area_index(), BossWordOrder::Native, game) & 1 != 0
}
fn cooldown(runtime: &mut Runtime<'_>, key: Key) {
    runtime.edit_actor(key, Native, |actor| {
        if actor.variables[1] != 0 {
            actor.variables[1] = actor.variables[1].wrapping_sub(1);
        }
    });
}
pub(crate) fn palette(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    if runtime.word(W::door_transition_flag_enemies) != 0
        || runtime.word(W::wrecked_ship_robot_palanim_timer) == 0
    {
        return;
    }
    let remaining = runtime
        .word(W::wrecked_ship_robot_palanim_timer)
        .wrapping_sub(1);
    runtime.write_slice_le(W::wrecked_ship_robot_palanim_timer, remaining);
    if runtime.word(W::wrecked_ship_robot_palanim_timer) != 0 {
        return;
    }
    let mut cursor = runtime.native_word(0xd656);
    while PALETTE[usize::from(cursor >> 1)] < 0 {
        cursor = 0;
    }
    let index = usize::from(cursor >> 1);
    let palette = usize::from(runtime.word(W::wrecked_ship_robot_palanim_palindex) >> 8) * 8;
    for i in 0..4 {
        runtime.write_slice_le(
            A::palette_buffer + 2 * (palette + 137 + i),
            PALETTE[index + i] as u16,
        );
    }
    runtime.write_slice_le(
        W::wrecked_ship_robot_palanim_timer,
        PALETTE[index + 4] as u16,
    );
    runtime.write_slice_native(0xd656, cursor.wrapping_add(10));
}
pub(crate) fn initialize(game: &mut Game) {
    if !powered(game) {
        initialize_dormant(game);
        return;
    }
    let mut runtime = Runtime::new(game);
    runtime.set_byte(0x178e, 0xa8);
    runtime.write_slice_native(0x178c, 0xcc67);
    let key = selected(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.properties |= 0xa000;
        actor.instruction_delay = 4;
        actor.timer = 0;
        actor.instruction = 0xc6e5;
        actor.variables[0] = 0xfe00;
    });
    runtime.write_slice_le(W::wrecked_ship_robot_palanim_timer, 1);
    runtime.write_slice_native(0xd656, 0);
    runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0);
    let palette = runtime.resolve_actor(key, Native).palette;
    runtime.write_slice_le(W::wrecked_ship_robot_palanim_palindex, palette);
}
pub(crate) fn initialize_dormant(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    let parameter = runtime.edit_actor(key, Native, |actor| {
        let mut parameter = actor.parameters[0];
        if parameter & 0x8000 != 0 || (parameter.wrapping_sub(4) as i16) >= 0 {
            parameter = 0;
        }
        actor.parameters[0] = parameter;
        parameter
    });
    let image = runtime.loaded_image().expect("ROM must be loaded");
    let instruction = image.table_word(
        crate::rom::lorom_offset(0xa8cc30) as usize,
        parameter as isize,
    );
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        actor.properties |= 0x8000;
        actor.instruction_delay = 1;
        actor.timer = 0;
    });
    runtime.write_slice_le(W::wrecked_ship_robot_palanim_timer, 0);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[4] = 0;
        actor.pre_instruction = 1;
    });
    let palette = usize::from(runtime.word(W::wrecked_ship_robot_palanim_palindex) >> 8) * 8;
    for i in 137..=140 {
        runtime.write_slice_le(A::palette_buffer + 2 * (palette + i), 10);
    }
}
pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    let actor = runtime.resolve_actor(key, Native);
    let speed = ((u32::from(actor.pre_instruction) << 16) | u32::from(actor.variables[4])) as i32;
    if api::move_vertical(&mut runtime, key, speed) != api::MovementResult::Blocked {
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction_delay = actor.instruction_delay.wrapping_add(1);
            let speed = ((u32::from(actor.pre_instruction) << 16) | u32::from(actor.variables[4]))
                .wrapping_add(0x8000);
            actor.variables[4] = speed as u16;
            actor.pre_instruction = (speed >> 16) as u16;
        });
    }
}
pub(crate) fn touch_dormant(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let x = runtime.resolve_actor(selected(&runtime), Native).x.whole;
    let left = (x as i16).wrapping_sub(runtime.word(W::samus_x_pos) as i16) < 0;
    runtime.write_slice_le(W::extra_samus_x_displacement, if left { 4 } else { 0xfffc });
}
pub(crate) fn shot_dormant(game: &mut Game) {
    combat::shot_reaction(&mut Runtime::new(game), api::ShotHandling::Finalize);
}
pub(crate) fn shot(game: &mut Game) {
    if !powered(game) {
        return;
    }
    let mut runtime = Runtime::new(game);
    combat::shot_reaction(&mut runtime, api::ShotHandling::Finalize);
    let key = selected(&runtime);
    let actor = runtime.resolve_actor(key, Native);
    if actor.health == 0 {
        return;
    }
    let left = (runtime.word(W::samus_x_pos) as i16).wrapping_sub(actor.x.whole as i16) < 0;
    let instruction = match (actor.variables[0] & 0x8000 != 0, left) {
        (true, false) => 0xc833,
        (true, true) => 0xc7bb,
        (false, true) => 0xca7d,
        (false, false) => 0xca01,
    };
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        actor.instruction_delay = 1;
        actor.variables[1] = actor.variables[1].wrapping_add(64);
    });
}
fn fire(
    runtime: &mut Runtime<'_>,
    key: Key,
    script: EnemyScript,
    definition: u16,
    next: u16,
) -> EnemyScript {
    if runtime.resolve_actor(key, Native).variables[1] != 0 {
        cooldown(runtime, key);
        return script;
    }
    let delay = (runtime.word(W::random_number) & 0x1f) + 16;
    runtime.edit_actor(key, Native, |actor| actor.variables[1] = delay);
    let current = runtime.word(W::cur_enemy_index);
    runtime.foreign(|game| SpawnEprojWithGfx(delay, current, definition, game));
    EnemyScript::jump(next)
}
fn walk(
    runtime: &mut Runtime<'_>,
    script: EnemyScript,
    motion: (u16, i16),
    instructions: (u16, Option<u16>),
    restore_y_first: bool,
) -> EnemyScript {
    let current = selected(runtime);
    cooldown(runtime, current);
    let key = selected(runtime);
    let (facing, step) = motion;
    let (wall, edge) = instructions;
    runtime.edit_actor(key, Native, |actor| actor.variables[0] = facing);
    let live = selected(runtime);
    if api::move_horizontal(
        runtime,
        live,
        i32::from(step) << 16,
        api::SlopePolicy::Ignore,
    ) == api::MovementResult::Blocked
    {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[1] = actor.variables[1].wrapping_add(8)
        });
        return EnemyScript::jump(wall);
    }
    if api::touches_player_from_above(runtime, selected(runtime)) {
        runtime.write_slice_le(W::extra_samus_x_subdisplacement, 0);
        runtime.write_slice_le(W::extra_samus_x_displacement, step as u16);
    }
    let Some(edge) = edge else {
        return script;
    };
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[3] = actor.y.whole;
        actor.variables[2] = actor.x.whole;
        let width = actor.radius.x.wrapping_mul(2);
        actor.x.whole = if step < 0 {
            actor.x.whole.wrapping_sub(width)
        } else {
            actor.x.whole.wrapping_add(width)
        };
    });
    let live = selected(runtime);
    let ground = api::move_vertical(runtime, live, 1 << 16) == api::MovementResult::Blocked;
    if !ground {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[1] = actor.variables[1].wrapping_add(8)
        });
    }
    // Keep the command-specific restore order after movement can mutate aliases.
    for y in [restore_y_first, !restore_y_first] {
        runtime.edit_actor(key, Native, |actor| {
            if y {
                actor.y.whole = actor.variables[3];
            } else {
                actor.x.whole = actor.variables[2];
            }
        });
    }
    if !ground {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[0] = if step < 0 { 512 } else { 0xfe00 }
        });
        return EnemyScript::jump(edge);
    }
    script
}
pub(crate) fn instruction(
    address: u32,
    k: u16,
    script: EnemyScript,
    game: &mut Game,
) -> EnemyScript {
    match address {
        11062987 => return EnemyScript::jump(0xc92d),
        11063490 => return EnemyScript::jump(0xc6e9),
        _ => {}
    }
    let mut runtime = Runtime::new(game);
    match address {
        11063441 => {
            let actor = runtime.resolve_actor(selected(&runtime), Native);
            let (x, y) = (actor.x.whole, actor.y.whole);
            let (camera_x, camera_y) =
                (runtime.word(W::layer1_x_pos), runtime.word(W::layer1_y_pos));
            if (camera_x.wrapping_sub(x) as i16) < 0
                && (camera_x.wrapping_add(256).wrapping_sub(x) as i16) >= 0
                && (camera_y.wrapping_sub(y) as i16) < 0
                && (camera_y.wrapping_add(224).wrapping_sub(y) as i16) >= 0
            {
                runtime.queue_sound(2, 6, 0x68);
            }
        }
        11063659 => {
            let key = selected(&runtime);
            cooldown(&mut runtime, key);
        }
        11063494 | 11063506 | 11063552 | 11063559 | 11063601 | 11063613 => {
            let (definition, next) = match address {
                11063494 => (0xd2d0, 0xcb1d),
                11063506 => (0xd2a6, 0xc8d1),
                11063552 => (0xd2b4, 0xcb09),
                11063559 => (0xd2b4, 0xc8bd),
                11063601 => (0xd2de, 0xcafd),
                _ => (0xd2c2, 0xc8b1),
            };
            return fire(&mut runtime, Key::from_legacy(k), script, definition, next);
        }
        11062537 | 11062692 | 11062762 | 11062917 | 11062991 | 11063146 | 11063216 | 11063371 => {
            let (motion, instructions, restore_y_first) = match address {
                11062537 => ((0xfe00, -4), (0xc73f, Some(0xcb65)), false),
                11062692 => ((0xfe00, -4), (0xc73f, None), false),
                11062762 => ((0xfe00, 4), (0xc6e9, Some(0xc91b)), false),
                11062917 => ((0xfe00, 4), (0xc6e9, None), false),
                11062991 => ((512, 4), (0xc985, Some(0xc91b)), false),
                11063146 => ((512, 4), (0xc985, None), false),
                11063216 => ((512, -4), (0xc6e9, Some(0xcb65)), true),
                _ => ((512, -4), (0xc6e9, None), false),
            };
            return walk(&mut runtime, script, motion, instructions, restore_y_first);
        }
        _ => {
            runtime.unreachable();
            return EnemyScript::Stop;
        }
    }
    script
}
