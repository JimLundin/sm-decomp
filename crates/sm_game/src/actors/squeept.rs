//! Squeept, Norfair's lava jumpers (bank $A2:BE1C-$A2:C0D0). Was `lava_jumper`.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::geruta::set_instruction_list;
use crate::actors::host::{ActorLease, BossEffects, LeaseActors};
use crate::actors::slot_view::{SlotRef, slot_view};
use crate::actors::state::Actor;
use crate::legacy_overflow;
use crate::player_projectiles::content::table;
use crate::session_access::SessionAccess as _;

slot_view! {
    /// The squeept's extended words: its animation finished flag, then the
    /// instruction list `set_instruction_list` caches.
    Squeept, SqueeptEdit {
        record: [],
        extended: [animation_finished = var_00, instruction_list = var_01],
    }
}
// Instruction lists rising and flipping / falling; functions.
const RISING_LIST: u16 = 0xbe3c;
const JUMP: u16 = 0xbf1a;
/// The bytes of one enemy slot.
const SLOT: u16 = 0x40;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Flight {
    Rising,
    Flipping,
    Falling,
}

/// Initialisation AI - enemy $D2BF (squeept) ($A2:BE99): the main part or its flames.
pub fn initialize<H: LeaseActors>(game: &mut H) {
    let mut runtime = game.lease();
    let raw = runtime.session().actors.controls.selected;
    *SqueeptEdit::of(&mut runtime, raw).animation_finished() = 0;
    *SqueeptEdit::of(&mut runtime, raw).instruction_list() = 0;
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
    let k = runtime.session().actors.controls.selected;
    *SqueeptEdit::of(&mut runtime, k).animation_finished() = 1;
}

/// Word `field` (at record byte `offset`) of the flame `k`'s main part, the
/// slot before it.
fn main_part(runtime: &impl ActorLease, k: u16, field: fn(&Actor) -> u16, offset: usize) -> u16 {
    match k.checked_sub(SLOT) {
        Some(main) => field(SlotRef::of(runtime, main).actor),
        // LEGACY(original-overflow): a flame in slot 0 reads the 40h bytes
        // before the enemy table ($7E:0F38 + offset), as the original's
        // `LDA $0F38+offset,x` does.
        None => legacy_overflow::lease_word(runtime, 0xf78 + offset + usize::from(k) - 0x40),
    }
}

/// Squeept function - flame ($A2:BEDC): shown while the main part's Y velocity is negative.
pub fn flame<H: LeaseActors>(k: u16, game: &mut H) {
    let mut runtime = game.lease();
    let key = Key::from_legacy(k);
    // The main part's health ($0F8C), frozen timer ($0F9E), Y velocity ($0FAC)
    // and Y position ($0F7E).
    if main_part(&runtime, k, |main| main.health, 0x14) == 0 {
        runtime.edit_actor(key, Native, |actor| actor.properties |= 0x200);
        return;
    }
    let frozen = main_part(&runtime, k, |main| main.frozen, 0x26);
    runtime.edit_actor(key, Native, |actor| actor.frozen = frozen);
    if frozen != 0 || (main_part(&runtime, k, |main| main.variables[2], 0x34) & 0x8000) == 0 {
        runtime.edit_actor(key, Native, |actor| actor.properties |= 0x100);
    } else {
        runtime.edit_actor(key, Native, |actor| actor.properties &= !0x100);
        let y = main_part(&runtime, k, |main| main.y.whole, 0x06);
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
    } else if phase == Flight::Flipping && Squeept::of(&runtime, k).animation_finished() != 0 {
        *SqueeptEdit::of(&mut runtime, k).animation_finished() = 0;
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xbfbc);
    }
}
