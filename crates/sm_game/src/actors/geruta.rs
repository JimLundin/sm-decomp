//! Geruta and Holtz, Norfair's swooping flyers with a flames part each (bank $A2:C0D1-$A2:CA2A).
//! Was `norfair_rio`. Their function is the pre-instruction word; tables pair Geruta, Holtz.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::api::{self, Axis};
use crate::actors::combat::current;
use crate::actors::host::{ActorLease, BossEffects, LeaseActors};
use crate::actors::rio::{aim_velocity, bounce_horizontal, move_axis};
use crate::enemy_script::EnemyScript;
use crate::ram::words as W;

// $7E:7800 + enemy: instruction list, animation finished flag, flames Y offset / visible flag.
const EXTRA: usize = 0x7800;
const FINISHED: usize = 0x7802;
const IDLE: [u16; 2] = [0xc0f1, 0xc61a]; // instruction lists
const FLAMES: [u16; 2] = [0xc18f, 0xc6b0];
const DESCENDING: [u16; 2] = [0xc12f, 0xc65a];
const HOLTZ_SWOOP_COOLDOWN: u16 = 0xc686;
const IDLE_FUNCTION: [u16; 2] = [0xc2e7, 0xc771];
const SWOOP_DESCENDING: [u16; 2] = [0xc361, 0xc7d6];

/// Instructions setting the animation finished flag, flames Y offset or Holtz's flames visible
/// flag ($A2:C1C9-$A2:C241, $A2:C6D2-$A2:C6F2); the script continues.
pub fn instruction<H: LeaseActors>(address: u32, cursor: EnemyScript, game: &mut H) -> EnemyScript {
    let (variable, value) = match address {
        0xa2c1c9 | 0xa2c6d2 => (1, 1),
        0xa2c1d4 | 0xa2c1df => (2, 8),
        0xa2c1ea | 0xa2c237 => (2, 12),
        0xa2c1f5 | 0xa2c221 => (2, (-12i16) as u16),
        0xa2c200 => (2, 4),
        0xa2c20b | 0xa2c6dd => (2, 0),
        0xa2c216 => (2, (-4i16) as u16),
        0xa2c22c => (2, (-16i16) as u16),
        0xa2c6e8 => (2, 1),
        _ => unreachable!("unknown Rio animation command"),
    };
    let mut runtime = game.lease();
    let at = EXTRA + usize::from(runtime.word(W::CUR_ENEMY_INDEX)) + 2 * variable;
    runtime.write_slice_native(at, value);
    cursor
}

/// The function that the flies', Squeept's, Geruta's, Holtz's and Dragon's main AI runs.
pub fn function<H: LeaseActors>(game: &mut H) -> u32 {
    let runtime = game.lease();
    let function = runtime
        .resolve_actor(current(&runtime), Native)
        .pre_instruction;
    0xa2_0000 | u32::from(function)
}

/// Initialisation AI - enemies $D2FF and $D33F ($A2:C242, $A2:C6F3).
pub fn initialize<H: LeaseActors>(holtz: bool, game: &mut H) {
    let mut runtime = game.lease();
    let raw = runtime.word(W::CUR_ENEMY_INDEX);
    let key = Key::from_legacy(raw);
    let scratch = EXTRA + usize::from(raw);
    runtime.write_slice_native(scratch + 2, 0);
    if !holtz {
        runtime.write_slice_native(scratch + 4, 0);
    }
    let flames = usize::from((runtime.resolve_actor(key, Native).parameters[0] & 0x8000) != 0);
    let (list, function) = [(IDLE, IDLE_FUNCTION), (FLAMES, [0xc281, 0xc72e])][flames];
    let (list, function) = (list[usize::from(holtz)], function[usize::from(holtz)]);
    if holtz {
        set_function(&mut runtime, key, function);
    }
    runtime.write_slice_native(scratch, list);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = list;
        if !holtz {
            actor.pre_instruction = function;
        }
    });
}

/// Geruta / Holtz function - flames ($A2:C281, $A2:C72E): follow the main part one slot back.
pub fn flames<H: LeaseActors>(k: u16, holtz: bool, game: &mut H) {
    let mut runtime = game.lease();
    let key = Key::from_legacy(k);
    let main = 0xe84 + (usize::from(k) >> 1) * 2; // + C8h health, DAh frozen, B6h X, BAh Y
    let main_extra = EXTRA - 0x40 + usize::from(k);
    if runtime.word(main + 200) == 0 {
        runtime.edit_actor(key, Native, |actor| actor.properties |= 0x200);
        return;
    }
    let frozen = runtime.word(main + 218);
    runtime.edit_actor(key, Native, |actor| actor.frozen = frozen);
    if !holtz && frozen == 0 {
        runtime.edit_actor(key, Native, |actor| actor.properties &= !0x100);
    }
    let flames = runtime.lease_native_word(main_extra + 4);
    let idle = runtime.lease_native_word(main_extra) == IDLE[0];
    let hidden = [idle, flames == 0][usize::from(holtz)];
    if frozen != 0 || hidden {
        runtime.edit_actor(key, Native, |actor| actor.properties |= 0x100);
        return;
    }
    if !holtz {
        let list = [FLAMES[0], 0xc1a3][usize::from((flames & 0x8000) != 0)]; // ascending, descending
        set_instruction_list(&mut runtime, list, 0);
    }
    runtime.edit_actor(key, Native, |actor| actor.properties &= !0x100);
    let x = runtime.word(main + 182);
    runtime.edit_actor(key, Native, |actor| actor.x.whole = x);
    let y = (if holtz { 12 } else { flames }).wrapping_add(runtime.word(main + 186));
    runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
}

/// Geruta / Holtz function - idle ($A2:C2E7, $A2:C771): swoop at Samus when near.
pub fn idle<H: LeaseActors>(k: u16, holtz: bool, game: &mut H) {
    let mut runtime = game.lease();
    let key = Key::from_legacy(k);
    let finished = FINISHED + usize::from(k);
    let h = usize::from(holtz);
    let distance = (api::player_delta(&runtime, key, Axis::X) as i16).wrapping_abs() as u16;
    if (runtime.word(W::RANDOM_NUMBER) & 0x101) != 0 && distance < [0xc0, 0x70][h] {
        let random = runtime.word(W::RANDOM_NUMBER);
        let index = [((random >> 1 & 2) >> 1) as isize, 0][h];
        let tables = [[0xa2c1c1, 0xa2c1c5], [0xa2c6ca, 0xa2c6ce]][h]; // swoop Y, X speeds
        aim_velocity(&mut runtime, key, 2 * h, tables, index);
        set_instruction_list(&mut runtime, [0xc107, 0xc630][h], 0); // start swoop
        set_function(&mut runtime, key, [0xc33f, 0xc7bb][h]); // start swoop
    } else if holtz || runtime.lease_native_word(finished) != 0 {
        runtime.write_slice_native(finished, 0);
        set_instruction_list(&mut runtime, IDLE[h], 0);
    }
}

/// Geruta function - start and finish swoop ($A2:C33F, $A2:C406), Holtz function - prepare to
/// swoop and swoop cooldown ($A2:C7BB, $A2:C888).
pub fn finish_animation<H: LeaseActors + BossEffects>(
    k: u16,
    holtz: bool,
    end: bool,
    game: &mut H,
) {
    let mut runtime = game.lease();
    let key = Key::from_legacy(k);
    if !holtz && end {
        set_function(&mut runtime, key, IDLE_FUNCTION[0]);
        return;
    }
    let finished = FINISHED + usize::from(k);
    if runtime.lease_native_word(finished) == 0 {
        return;
    }
    runtime.write_slice_native(finished, 0);
    let h = usize::from(holtz);
    let (list, function) = match (holtz, end) {
        (true, true) => (HOLTZ_SWOOP_COOLDOWN, IDLE_FUNCTION[1]),
        _ => (DESCENDING[h], SWOOP_DESCENDING[h]),
    };
    set_instruction_list(&mut runtime, list, 0);
    set_function(&mut runtime, key, function);
    drop(runtime);
    if !holtz {
        game.queue_sound(1, 0x65, 6);
    }
}

fn set_function(runtime: &mut impl ActorLease, key: Key, function: u16) {
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = function);
}

/// Set geruta / holtz / squeept instruction list ($A2:C40D, $A2:C8A3, $A2:C012), cached
/// `cache_offset` bytes into the enemy's extra RAM: restart only when it changes.
pub fn set_instruction_list(runtime: &mut impl ActorLease, list: u16, cache_offset: usize) {
    let raw = runtime.word(W::CUR_ENEMY_INDEX);
    let cache = EXTRA + usize::from(raw) + cache_offset;
    if runtime.lease_native_word(cache) != list {
        runtime.write_slice_native(cache, list);
        let key = Key::from_legacy(raw);
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
    }
}

/// Geruta / Holtz function - swoop - descending ($A2:C361, $A2:C7D6) and ascending ($A2:C3B1,
/// $A2:C82D).
pub fn swoop<H: LeaseActors + BossEffects>(k: u16, holtz: bool, descending: bool, game: &mut H) {
    let mut runtime = game.lease();
    let key = Key::from_legacy(k);
    let h = usize::from(holtz);
    let y = 2 * h;
    bounce_horizontal(&mut runtime, key, y + 1);
    let blocked = move_axis(&mut runtime, key, Axis::Y, y);
    let decelerate = |actor: &mut crate::actors::state::Actor| {
        actor.variables[y] = actor.variables[y].wrapping_sub(32);
        (actor.variables[y] as i16) < 0
    };
    let mut cry = false;
    if descending {
        if blocked || runtime.edit_actor(key, Native, decelerate) {
            runtime.edit_actor(key, Native, |actor| actor.variables[y] = u16::MAX);
            set_instruction_list(&mut runtime, [0xc145, 0xc662][h], 0); // start ascending
            set_function(&mut runtime, key, [0xc3b1, 0xc82d][h]); // swoop ascending
            cry = holtz;
        }
    } else if blocked {
        if holtz {
            set_instruction_list(&mut runtime, HOLTZ_SWOOP_COOLDOWN, 0);
        }
        set_function(&mut runtime, key, [0xc406, 0xc888][h]); // finish swoop, swoop cooldown
    } else {
        runtime.edit_actor(key, Native, decelerate);
        let finished = FINISHED + usize::from(k);
        if runtime.lease_native_word(finished) != 0 {
            runtime.write_slice_native(finished, 0);
            set_instruction_list(&mut runtime, [0xc179, 0xc674][h], 0); // ascending
        }
    }
    drop(runtime);
    if cry {
        game.queue_sound(1, 0x64, 6);
    }
}
