//! Lower Norfair spikey platforms: wait, fall, rest, rise; Samus standing on one
//! rides it. Extended words (0x7800 block) stay RAM-authoritative.
#![forbid(unsafe_code)]
use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::game::enemy_projectiles::host::Effects;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::words as fields;

/// Routine words in variable A (bank A6 offsets).
const WAIT: u16 = 0x8bb4;
const FALL: u16 = 0x8bdc;
const REST: u16 = 0x8c4a;
const RISE: u16 = 0x8c5d;
/// Extended words: wait reload, previous y, speed table offset, rest timer.
const RELOAD: usize = 0x7800;
const PREVIOUS_Y: usize = 0x7802;
const SPEED: usize = 0x7804;
const REST_TIMER: usize = 0x7806;

/// One word of the 16.16 fall speed rows at 0xA28187.
fn speed_word(runtime: &Runtime<'_>, index: u16) -> u16 {
    let rom = runtime.loaded_image().expect("ROM must be loaded");
    rom.table_word(crate::rom::lorom_offset(0xa28187) as usize, index as isize)
}

pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let index = runtime.word(fields::cur_enemy_index);
    let (key, at) = (Key::from_legacy(index), usize::from(index));
    let [table, wait] = runtime
        .resolve_actor(key, Native)
        .parameters
        .map(|p| p & 0xff);
    let speed = 8u16.wrapping_mul(table);
    runtime.write_slice_native(at + SPEED, speed);
    let fraction = speed_word(&runtime, (speed >> 1) + 1);
    let whole = speed_word(&runtime, speed >> 1);
    runtime.write_slice_native(at + RELOAD, wait);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = 0x8b29;
        let top = actor.y.whole;
        let bottom = top.wrapping_add(actor.parameters[1] >> 8);
        actor.variables = [WAIT, wait, top, bottom, fraction];
        actor.pre_instruction = whole;
    });
}

pub(crate) fn run(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let index = runtime.word(fields::cur_enemy_index);
    let (key, at) = (Key::from_legacy(index), usize::from(index));
    let start = runtime.resolve_actor(key, Native).y.whole;
    let mut landed = false;
    match runtime.resolve_actor(key, Native).variables[0] {
        WAIT => {
            let reload = runtime.native_word(at + RELOAD);
            let speed = runtime.edit_actor(key, Native, |actor| {
                actor.variables[1] = actor.variables[1].wrapping_sub(1);
                (actor.variables[1] == 0).then(|| {
                    actor.variables[..2].copy_from_slice(&[FALL, reload]);
                    8u16.wrapping_mul(actor.parameters[0] & 0xff)
                })
            });
            if let Some(speed) = speed {
                runtime.write_slice_native(at + SPEED, speed);
            }
        }
        FALL => {
            runtime.write_slice_native(at + PREVIOUS_Y, start);
            let speed = runtime.native_word(at + SPEED);
            let velocity = (u32::from(speed_word(&runtime, speed >> 1)) << 16)
                | u32::from(speed_word(&runtime, (speed >> 1) + 1));
            landed = runtime.edit_actor(key, Native, |actor| {
                actor.y.advance(velocity as i32);
                let landed = actor.y.whole.wrapping_sub(actor.variables[3]) as i16 >= 0;
                if landed {
                    actor.variables[0] = REST;
                    actor.y.whole = actor.variables[3];
                }
                landed
            });
            if landed {
                runtime.write_slice_native(at + REST_TIMER, 64);
            }
            carry_samus(&mut runtime, key, start);
            let fast = speed.wrapping_sub(504) & 0x8000 == 0;
            runtime.write_slice_native(at + SPEED, if fast { 512 } else { speed.wrapping_add(8) });
        }
        REST => {
            let timer = runtime.native_word(at + REST_TIMER).wrapping_sub(1);
            runtime.write_slice_native(at + REST_TIMER, timer);
            if timer == 0 {
                runtime.edit_actor(key, Native, |actor| actor.variables[0] = RISE);
            }
        }
        RISE => {
            runtime.write_slice_native(at + PREVIOUS_Y, start);
            runtime.edit_actor(key, Native, |actor| {
                actor.y.advance(-0x8000);
                if (actor.y.whole.wrapping_sub(actor.variables[2]) as i16) < 0 {
                    actor.variables[0] = WAIT;
                    actor.y.whole = actor.variables[2];
                }
            });
            carry_samus(&mut runtime, key, start);
        }
        _ => runtime.unreachable(),
    }
    drop(runtime);
    if landed {
        crate::game::music::QueueSfx2_Max6(0x1b, game);
    }
}

/// Samus standing on the platform moves with it.
fn carry_samus(runtime: &mut Runtime<'_>, key: Key, start: u16) {
    let actor = runtime.resolve_actor(key, Native);
    let (x, y, width, height) = (actor.x.whole, actor.y.whole, actor.radius.x, actor.radius.y);
    let dx = crate::rtl::abs16(runtime.word(fields::samus_x_pos).wrapping_sub(x));
    let samus_width = runtime.word(fields::samus_x_radius);
    let feet = runtime.word(fields::samus_y_pos).wrapping_add(5);
    if dx >= samus_width && dx - samus_width >= width || feet.wrapping_sub(y) as i16 >= 0 {
        return;
    }
    let (gap, samus_height) = (y.wrapping_sub(feet), runtime.word(fields::samus_y_radius));
    if gap < samus_height || gap - samus_height <= height {
        let moved = runtime.word(fields::extra_samus_y_displacement);
        runtime.write_word_le(
            moved.wrapping_add(y.wrapping_sub(start)),
            fields::extra_samus_y_displacement,
        );
    }
}
