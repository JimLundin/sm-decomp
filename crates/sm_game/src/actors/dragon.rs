//! Dragon, lava seahorses with wings (bank $A2:E57B-$A2:E977). Was `lava_seahorse_pair`. Lane
//! 3 is the timer; RAM at $7E:7800 + enemy holds the list index, current list index, fire flag.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::api;
use crate::actors::host::{ActorLease, ActorRuntime, Death, LeaseActors, Shot};
use crate::enemy_projectiles::host::Effects as _;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::table as list_table;
use crate::ram::words as W;

const WAIT_TO_RISE: u16 = 0xe654;
const RISING: u16 = 0xe6ad;
const ATTACKING: u16 = 0xe6f1;
const WAIT_TO_SINK: u16 = 0xe734;
const SINKING: u16 = 0xe749;
const LISTS: u32 = 0xa2_e5ef; // instruction list pointers

/// Set dragon (wings) instruction list ($A2:E782, $A2:E7A5) and Oum's: restart enemy `raw`'s
/// animation from list pointer table `table` when its list index in $7E:7800 + `raw` changed.
pub fn restart_from_list_cache(runtime: &mut impl ActorLease, raw: usize, table: u32) {
    let extra = 0x7800 + raw;
    let list = runtime.lease_native_word(extra);
    if list != runtime.lease_native_word(extra + 2) {
        runtime.write_slice_native(extra + 2, list);
        let instruction = list_table(runtime, table, list as isize);
        let base = 0xf78 + raw; // the instruction list, its timer and the enemy timer
        runtime.write_slice_native(base + 0x1a, instruction);
        runtime.write_slice_native(base + 0x1c, 1);
        runtime.write_slice_native(base + 0x18, 0);
    }
}

fn tick(runtime: &mut impl ActorLease, key: Key) -> u16 {
    runtime.edit_actor(key, Native, |actor| {
        let previous = actor.variables[3];
        actor.variables[3] = previous.wrapping_sub(1);
        previous
    })
}

fn set_timer(runtime: &mut impl ActorLease, key: Key, timer: u16, function: u16) {
    runtime.edit_actor(key, Native, |actor| {
        (actor.variables[3], actor.pre_instruction) = (timer, function)
    });
}

/// Initialisation AI - enemy $D4BF (dragon) ($A2:E606): parameter 1 non-zero for the wings.
pub fn initialize<H: LeaseActors>(game: &mut H) {
    let mut runtime = game.lease();
    let raw = runtime.word(W::CUR_ENEMY_INDEX);
    let key = Key::from_legacy(raw);
    let extra = 0x7800 + usize::from(raw);
    runtime.write_slice_native(extra + 4, 0);
    let wings = runtime.resolve_actor(key, Native).parameters[0] != 0;
    for at in [extra, extra + 2] {
        runtime.write_slice_native(at, if wings { 2 } else { 0 });
    }
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if wings { 0xe5a1 } else { 0xe59b };
        actor.properties |= if wings { 0x400 } else { 0 };
        actor.pre_instruction = if wings { 0xe781 } else { WAIT_TO_RISE };
    });
}

/// Instruction - set animation finished flag ($A2:E5FB): ready to fire.
pub fn set_animation_finished<H: LeaseActors>(cursor: EnemyScript, game: &mut H) -> EnemyScript {
    let mut runtime = game.lease();
    let at = 0x7804 + usize::from(runtime.word(W::CUR_ENEMY_INDEX));
    runtime.write_slice_native(at, 1);
    cursor
}

/// Touch, shot and power bomb reactions ($A2:E7C8-E7D4), then the dragon reaction ($A2:E7DA).
pub fn react<H: LeaseActors + ActorRuntime>(ea: u32, game: &mut H) {
    match ea {
        0xa2_e7c8 => game.contact_reaction(Death::Finalize),
        0xa2_e7ce => game.shot_reaction(Shot::Finalize),
        _ => game.power_bomb_reaction(Death::Finalize),
    }
    let mut runtime = game.lease();
    let base = 0xf78 + usize::from(runtime.word(W::CUR_ENEMY_INDEX));
    if runtime.lease_native_word(base + 0x14) != 0 {
        for field in [0x2a, 0x28, 0x24, 0x26, 0x12] {
            let value = runtime.lease_native_word(base + field);
            runtime.write_slice_native(base + 64 + field, value);
        }
    } else {
        let properties = runtime.lease_native_word(base + 64 + 0xe) | 0x200;
        runtime.write_slice_native(base + 64 + 0xe, properties);
    }
}

/// Dragon functions - wait to rise, rising, attacking, wait to sink and sinking ($A2:E654-E780).
pub fn step<H: LeaseActors>(k: u16, function: u16, game: &mut H) {
    let mut runtime = game.lease();
    let key = Key::from_legacy(k);
    let extra = 0x7800 + usize::from(k);
    match function {
        WAIT_TO_RISE | RISING | SINKING => {
            if (tick(&mut runtime, key).wrapping_sub(1) & 0x8000) != 0 {
                let (delay, next) = match function {
                    WAIT_TO_RISE => (48, RISING),
                    RISING => (3, ATTACKING),
                    _ => (128, WAIT_TO_RISE),
                };
                if function == RISING {
                    let list = runtime.lease_native_word(extra).wrapping_add(4);
                    runtime.write_slice_native(extra, list);
                }
                set_timer(&mut runtime, key, delay, next);
                if function == WAIT_TO_RISE {
                    let delta = api::player_delta(&runtime, key, api::Axis::X);
                    let left = runtime.edit_actor(key, Native, |actor| {
                        actor.variables[0] = actor.variables[0] & 0x7fff | delta & 0x8000;
                        (actor.variables[0] & 0x8000) != 0
                    });
                    // Bit 0 of the list index faces right; the high byte is rewritten as is.
                    for at in [extra, extra + 64] {
                        let index = runtime.lease_native_word(at);
                        runtime.write_slice_native(at, if left { index & !1 } else { index | 1 });
                    }
                    restart_from_list_cache(&mut runtime, usize::from(k), LISTS);
                    restart_from_list_cache(&mut runtime, usize::from(k) + 64, LISTS);
                }
            }
            if function != WAIT_TO_RISE {
                for raw in [usize::from(k), usize::from(k) + 64] {
                    let at = 0xf7e + raw; // Y position
                    let delta = if function == RISING { u16::MAX } else { 1 };
                    let y = runtime.lease_native_word(at).wrapping_add(delta);
                    runtime.write_slice_native(at, y);
                }
            }
        }
        ATTACKING => {
            restart_from_list_cache(&mut runtime, usize::from(k), LISTS);
            if runtime.lease_native_word(extra + 4) != 0 {
                runtime.write_slice_native(extra + 4, 0);
                runtime.write_slice_native(extra + 2, u16::MAX);
                runtime.foreign(|game| game.spawn_projectile(0xffff, k, 0xb5cb));
                runtime.queue_sound(2, 6, 0x61);
                if tick(&mut runtime, key) == 1 {
                    let list = runtime.lease_native_word(extra).wrapping_sub(4);
                    runtime.write_slice_native(extra, list);
                    set_timer(&mut runtime, key, 96, WAIT_TO_SINK);
                }
            }
        }
        WAIT_TO_SINK => {
            if tick(&mut runtime, key) == 1 {
                set_timer(&mut runtime, key, 48, SINKING);
                restart_from_list_cache(&mut runtime, usize::from(k), LISTS);
            }
        }
        _ => unreachable!("unknown paired seahorse phase"),
    }
}
