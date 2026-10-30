//! Squeept, Norfair's lava jumpers (bank $A2:BE1C-$A2:C0D0). Was `lava_jumper`.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::geruta::set_instruction_list;
use crate::actors::host::{ActorLease, BossEffects, LeaseActors};
use crate::player_projectiles::content::table;
use crate::ram::words as W;

// $7E:7800 + enemy: the animation finished flag, then the cached instruction list.
const FINISHED: usize = 0x7800;
// Instruction lists rising and flipping / falling; functions.
const RISING_LIST: u16 = 0xbe3c;
const JUMP: u16 = 0xbf1a;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Flight {
    Rising,
    Flipping,
    Falling,
}

/// Initialisation AI - enemy $D2BF (squeept) ($A2:BE99): the main part or its flames.
pub fn initialize<H: LeaseActors>(game: &mut H) {
    let mut runtime = game.lease();
    let raw = runtime.word(W::CUR_ENEMY_INDEX);
    runtime.write_slice_native(FINISHED + usize::from(raw), 0);
    runtime.write_slice_native(FINISHED + 2 + usize::from(raw), 0);
    runtime.edit_actor(Key::from_legacy(raw), Native, |actor| {
        if (actor.parameters[0] & 0x8000) != 0 {
            (actor.instruction, actor.pre_instruction) = (0xbe62, 0xbedc);
        } else {
            (actor.variables[3], actor.variables[4]) = (actor.x.whole, actor.y.whole);
            (actor.instruction, actor.pre_instruction) = (RISING_LIST, JUMP);
        }
    });
}

/// Instruction - set animation finished flag ($A2:BE8E).
pub fn set_animation_finished<H: LeaseActors>(game: &mut H) {
    let mut runtime = game.lease();
    let at = FINISHED + usize::from(runtime.word(W::CUR_ENEMY_INDEX));
    runtime.write_slice_native(at, 1);
}

/// Squeept function - flame ($A2:BEDC): shown while the main part's Y velocity is negative.
pub fn flame<H: LeaseActors>(k: u16, game: &mut H) {
    let mut runtime = game.lease();
    let key = Key::from_legacy(k);
    let main = 0xe84 + (usize::from(k) >> 1) * 2; // + C8h health, DAh frozen, BAh Y
    if runtime.word(main + 200) == 0 {
        runtime.edit_actor(key, Native, |actor| actor.properties |= 0x200);
        return;
    }
    let frozen = runtime.word(main + 218);
    runtime.edit_actor(key, Native, |actor| actor.frozen = frozen);
    if frozen != 0 || (runtime.word(0xf68 + (usize::from(k) >> 1) * 2 + 4) & 0x8000) == 0 {
        runtime.edit_actor(key, Native, |actor| actor.properties |= 0x100);
    } else {
        runtime.edit_actor(key, Native, |actor| actor.properties &= !0x100);
        let y = runtime.word(main + 186);
        runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
    }
}

/// Squeept function - jump ($A2:BF1A): a random initial Y velocity ($A2:BE86) and a splash.
pub fn jump<H: LeaseActors + BossEffects>(k: u16, game: &mut H) {
    let mut runtime = game.lease();
    let index = isize::from((runtime.lease_byte(0x5e6) & 6) >> 1);
    let speed = table(&runtime, 0xa2_be86, index);
    runtime.edit_actor(Key::from_legacy(k), Native, |actor| {
        (actor.variables[2], actor.pre_instruction) = (speed, 0xbf3e);
        actor.properties |= 0x800;
    });
    drop(runtime);
    game.queue_sound(1, 0xd, 6);
}

/// Squeept function - rising, flipping and falling ($A2:BF3E, $A2:BF7C, $A2:BFBC).
pub fn fly<H: LeaseActors>(k: u16, phase: Flight, game: &mut H) {
    let mut runtime = game.lease();
    let key = Key::from_legacy(k);
    let velocity =
        i16::from_le_bytes(runtime.resolve_actor(key, Native).variables[2].to_ne_bytes());
    runtime.edit_actor(key, Native, |actor| actor.y.advance_8_8(velocity));
    if phase == Flight::Falling && (runtime.resolve_actor(key, Native).y.whole & 0xf0) == 0xf0 {
        let [_, _, _, x, y] = runtime.resolve_actor(key, Native).variables; // the spawn point
        runtime.edit_actor(key, Native, |actor| (actor.x.whole, actor.y.whole) = (x, y));
        set_instruction_list(&mut runtime, RISING_LIST, 2);
        runtime.edit_actor(key, Native, |actor| {
            actor.pre_instruction = JUMP;
            actor.properties &= !0x800;
        });
        return;
    }
    let speed = runtime.edit_actor(key, Native, |actor| {
        actor.variables[2] = actor.variables[2].wrapping_add(56);
        actor.variables[2]
    });
    if phase == Flight::Rising && speed >= 0xfc00 {
        set_instruction_list(&mut runtime, 0xbe42, 2);
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xbf7c);
    } else if phase == Flight::Flipping && runtime.lease_native_word(FINISHED + usize::from(k)) != 0
    {
        runtime.write_slice_native(FINISHED + usize::from(k), 0);
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xbfbc);
    }
}
