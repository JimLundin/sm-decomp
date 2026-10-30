//! The Work Robot, which walks the Wrecked Ship shooting lasers once the power is back,
//! and stands dormant before (bank $A8:C6B3-$A8:D820). Was `wrecked_ship_robots`.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess as _, ActorKey as Key};
use crate::actors::api::{self, MovementResult::Blocked, ShotHandling, SlopePolicy};
use crate::actors::combat;
use crate::actors::host::{ActorLease, ActorRuntime as _, EnemyHost, LeaseActors};
use crate::actors::session_slots::SlotAccess as _;
use crate::enemy_projectiles::host::Effects as _;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::table;
use crate::ram::{word_arrays as A, words as W};
use crate::session_access::SessionAccess as _;

/// The palette cycle ($A8:CC67): four colours then a duration, ended by -1.
const PALETTE: [i16; 31] = [
    0x1f, 0x18, 0xf, 8, 0x40, 0x18, 0xf, 8, 0x1f, 0x10, 0xf, 8, 0x1f, 0x18, 0x10, 8, 0x1f, 0x18,
    0xf, 0x40, 0xf, 8, 0x1f, 0x18, 0x10, 0x18, 0xf, 8, 0x1f, 0x10, -1,
];
/// $7E:D656: the palette cycle cursor (`work_robot_palanim_table_index`).
const PALETTE_CURSOR: usize = 0xd656;
const GFX_DRAWN_HOOK: usize = 0x178c;
const GFX_DRAWN_HOOK_BANK: usize = 0x178e;
/// The robot's colours, 137 entries into its palette line.
const COLOURS: usize = 137;

fn powered(game: &mut (impl EnemyHost + ?Sized)) -> bool {
    game.area_boss_flags() & 1 != 0
}
fn delay_laser(runtime: &mut impl ActorLease, key: Key, frames: u16) {
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[1] = actor.variables[1].wrapping_add(frames)
    });
}
/// Decrement enemy laser cooldown ($A8:CCFF).
fn cooldown(runtime: &mut impl ActorLease) {
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| {
        if actor.variables[1] != 0 {
            actor.variables[1] = actor.variables[1].wrapping_sub(1);
        }
    });
}
/// Enemy graphics drawn hook - work robot - periodically cycle between palettes ($A8:CC67).
pub fn palette<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let door = runtime.word(W::DOOR_TRANSITION_FLAG_ENEMIES) != 0;
    if door || runtime.word(W::WORK_ROBOT_PALANIM_TIMER) == 0 {
        return;
    }
    let remaining = runtime.word(W::WORK_ROBOT_PALANIM_TIMER).wrapping_sub(1);
    runtime.write_slice_le(W::WORK_ROBOT_PALANIM_TIMER, remaining);
    if runtime.word(W::WORK_ROBOT_PALANIM_TIMER) != 0 {
        return;
    }
    let mut cursor = runtime.lease_native_word(PALETTE_CURSOR);
    while PALETTE[usize::from(cursor >> 1)] < 0 {
        cursor = 0;
    }
    let index = usize::from(cursor >> 1);
    let line = usize::from(runtime.word(W::WORK_ROBOT_PALANIM_PALINDEX) >> 8) * 8;
    for i in 0..4 {
        let colour = PALETTE[index + i] as u16;
        runtime.write_slice_le(A::PALETTE_BUFFER + 2 * (line + COLOURS + i), colour);
    }
    runtime.write_slice_le(W::WORK_ROBOT_PALANIM_TIMER, PALETTE[index + 4] as u16);
    runtime.write_slice_native(PALETTE_CURSOR, cursor.wrapping_add(10));
}
/// Initialisation AI - work robot ($A8:CB77), or deactivated while the power is off.
pub fn initialize<H: LeaseActors + EnemyHost + ?Sized>(game: &mut H) {
    if !powered(game) {
        return initialize_deactivated(game);
    }
    let runtime = &mut game.lease();
    runtime.set_slot_byte(GFX_DRAWN_HOOK_BANK, 0xa8);
    runtime.write_slice_native(GFX_DRAWN_HOOK, 0xcc67);
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.properties |= 0xa000;
        actor.instruction_delay = 4;
        actor.timer = 0;
        actor.instruction = 0xc6e5;
        actor.variables[0] = 0xfe00;
    });
    runtime.write_slice_le(W::WORK_ROBOT_PALANIM_TIMER, 1);
    runtime.write_slice_native(PALETTE_CURSOR, 0);
    runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0);
    let palette = runtime.resolve_actor(key, Native).palette;
    runtime.write_slice_le(W::WORK_ROBOT_PALANIM_PALINDEX, palette);
}
/// Initialisation AI - work robot deactivated ($A8:CBCC).
pub fn initialize_deactivated<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let key = combat::current(runtime);
    let pose = runtime.edit_actor(key, Native, |actor| {
        if actor.parameters[0] & 0x8000 != 0 || (actor.parameters[0].wrapping_sub(4) as i16) >= 0 {
            actor.parameters[0] = 0;
        }
        actor.parameters[0]
    });
    let instruction = table(runtime, 0xa8_cc30, pose as isize);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        actor.properties |= 0x8000;
        actor.instruction_delay = 1;
        actor.timer = 0;
    });
    runtime.write_slice_le(W::WORK_ROBOT_PALANIM_TIMER, 0);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[4] = 0;
        actor.pre_instruction = 1;
    });
    let line = usize::from(runtime.word(W::WORK_ROBOT_PALANIM_PALINDEX) >> 8) * 8;
    for i in COLOURS..COLOURS + 4 {
        runtime.write_slice_le(A::PALETTE_BUFFER + 2 * (line + i), 10);
    }
}
/// Main AI - work robot ($A8:CC36): fall, accelerating.
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let key = combat::current(runtime);
    let actor = runtime.resolve_actor(key, Native);
    let speed = ((u32::from(actor.pre_instruction) << 16) | u32::from(actor.variables[4])) as i32;
    if api::move_vertical(runtime, key, speed) != Blocked {
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction_delay = actor.instruction_delay.wrapping_add(1);
            let speed = ((u32::from(actor.pre_instruction) << 16) | u32::from(actor.variables[4]))
                .wrapping_add(0x8000);
            actor.variables[4] = speed as u16;
            actor.pre_instruction = (speed >> 16) as u16;
        });
    }
}
/// Enemy touch - work robot ($A8:D174): push Samus away 4 px.
pub fn touch<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let key = combat::current(runtime);
    let x = runtime.resolve_actor(key, Native).x.whole;
    let left = (x as i16).wrapping_sub(runtime.word(W::SAMUS_X_POS) as i16) < 0;
    runtime.write_slice_le(W::EXTRA_SAMUS_X_DISPLACEMENT, if left { 4 } else { 0xfffc });
}
/// Enemy shot - work robot deactivated ($A8:D18D).
pub fn shot_deactivated<H: LeaseActors + ?Sized>(game: &mut H) {
    combat::shot_reaction(&mut game.lease(), ShotHandling::Finalize);
}
/// Enemy shot - work robot ($A8:D192): knocked back and its laser delayed, while powered.
pub fn shot<H: LeaseActors + EnemyHost + ?Sized>(game: &mut H) {
    if !powered(game) {
        return;
    }
    let runtime = &mut game.lease();
    combat::shot_reaction(runtime, ShotHandling::Finalize);
    let key = combat::current(runtime);
    let actor = runtime.resolve_actor(key, Native);
    if actor.health == 0 {
        return;
    }
    let left = (runtime.word(W::SAMUS_X_POS) as i16).wrapping_sub(actor.x.whole as i16) < 0;
    let instruction = match (actor.variables[0] & 0x8000 != 0, left) {
        (true, false) => 0xc833,
        (true, true) => 0xc7bb,
        (false, true) => 0xca7d,
        (false, false) => 0xca01,
    };
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        actor.instruction_delay = 1;
    });
    delay_laser(runtime, key, 64);
}
/// Move forwards or backwards by `step`; at a wall, or approaching a fall on the
/// `edge` commands, go to their instruction lists.
fn walk(
    runtime: &mut impl ActorLease,
    script: EnemyScript,
    (facing, step): (u16, i16),
    (wall, edge): (u16, Option<u16>),
    restore_y_first: bool,
) -> EnemyScript {
    let key = combat::current(runtime);
    cooldown(runtime);
    runtime.edit_actor(key, Native, |actor| actor.variables[0] = facing);
    if api::move_horizontal(runtime, key, i32::from(step) << 16, SlopePolicy::Ignore) == Blocked {
        delay_laser(runtime, key, 8);
        return EnemyScript::jump(wall);
    }
    if api::touches_player_from_above(runtime, key) {
        runtime.write_slice_le(W::EXTRA_SAMUS_X_SUBDISPLACEMENT, 0);
        runtime.write_slice_le(W::EXTRA_SAMUS_X_DISPLACEMENT, step as u16);
    }
    let Some(edge) = edge else {
        return script;
    };
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[3] = actor.y.whole;
        actor.variables[2] = actor.x.whole;
        let width = actor.radius.x.wrapping_mul(2);
        let width = [width, width.wrapping_neg()][usize::from(step < 0)];
        actor.x.whole = actor.x.whole.wrapping_add(width);
    });
    let ground = api::move_vertical(runtime, key, 1 << 16) == Blocked;
    if !ground {
        delay_laser(runtime, key, 8);
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
        let facing = if step < 0 { 512 } else { 0xfe00 };
        runtime.edit_actor(key, Native, |actor| actor.variables[0] = facing);
        return EnemyScript::jump(edge);
    }
    script
}
/// The work robot instructions ($A8:CD09-$A8:D16B).
pub fn instruction<H: LeaseActors + ?Sized>(
    ea: u32,
    k: u16,
    j: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    match ea {
        0xa8_cecb => return EnemyScript::jump(0xc92d),
        0xa8_d0c2 => return EnemyScript::jump(0xc6e9),
        _ => {}
    }
    let runtime = &mut game.lease();
    match ea {
        // Instruction - queue work robot sound effect if on screen ($A8:D091).
        0xa8_d091 => {
            let actor = runtime.resolve_actor(combat::current(runtime), Native);
            let (x, y) = (actor.x.whole, actor.y.whole);
            let [camera_x, camera_y] = runtime.session().room.camera.layer1;
            if (camera_x.wrapping_sub(x) as i16) < 0
                && (camera_x.wrapping_add(256).wrapping_sub(x) as i16) >= 0
                && (camera_y.wrapping_sub(y) as i16) < 0
                && (camera_y.wrapping_add(224).wrapping_sub(y) as i16) >= 0
            {
                runtime.queue_sound(2, 6, 0x68);
            }
        }
        0xa8_d16b => cooldown(runtime),
        // Instruction - try shooting laser up, horizontally or down, right or left.
        0xa8_d0c6 | 0xa8_d0d2 | 0xa8_d100 | 0xa8_d107 | 0xa8_d131 | 0xa8_d13d => {
            let (definition, next) = match ea {
                0xa8_d0c6 => (0xd2d0, 0xcb1d),
                0xa8_d0d2 => (0xd2a6, 0xc8d1),
                0xa8_d100 => (0xd2b4, 0xcb09),
                0xa8_d107 => (0xd2b4, 0xc8bd),
                0xa8_d131 => (0xd2de, 0xcafd),
                _ => (0xd2c2, 0xc8b1),
            };
            // Handle firing laser ($A8:D0DC, $A8:D10C, $A8:D147), once cooled down.
            let key = Key::from_legacy(k);
            if runtime.resolve_actor(key, Native).variables[1] != 0 {
                cooldown(runtime);
                return j;
            }
            let delay = (runtime.word(W::RANDOM_NUMBER) & 0x1f) + 16;
            runtime.edit_actor(key, Native, |actor| actor.variables[1] = delay);
            let current = combat::current(runtime).byte_index();
            runtime.foreign(|game| game.spawn_projectile(delay, current, definition));
            return EnemyScript::jump(next);
        }
        // Instruction - facing left or right - move forwards or backwards ($A8:CD09-$A8:D04B).
        0xa8_cd09 | 0xa8_cda4 | 0xa8_cdea | 0xa8_ce85 | 0xa8_cecf | 0xa8_cf6a | 0xa8_cfb0
        | 0xa8_d04b => {
            let (motion, instructions, restore_y_first) = match ea {
                0xa8_cd09 => ((0xfe00, -4), (0xc73f, Some(0xcb65)), false),
                0xa8_cda4 => ((0xfe00, -4), (0xc73f, None), false),
                0xa8_cdea => ((0xfe00, 4), (0xc6e9, Some(0xc91b)), false),
                0xa8_ce85 => ((0xfe00, 4), (0xc6e9, None), false),
                0xa8_cecf => ((512, 4), (0xc985, Some(0xc91b)), false),
                0xa8_cf6a => ((512, 4), (0xc985, None), false),
                0xa8_cfb0 => ((512, -4), (0xc6e9, Some(0xcb65)), true),
                _ => ((512, -4), (0xc6e9, None), false),
            };
            return walk(runtime, j, motion, instructions, restore_y_first);
        }
        _ => {
            runtime.unreachable();
            return EnemyScript::Stop;
        }
    }
    j
}
