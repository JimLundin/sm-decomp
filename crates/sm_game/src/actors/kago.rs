//! Kago, the bug hive of Norfair, which shakes the screen and releases a bug
//! each time it is shot (bank $A8:AB1E-$A8:ABCF).
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::api::{DeathKind, ShotHandling};
use crate::actors::combat;
use crate::actors::host::{ActorLease, ActorRuntime as _, LeaseActors};
use crate::actors::layouts::part_view;
use crate::enemy_projectiles::host::Effects as _;
use crate::ram::words;
use crate::session_slot;

part_view! {
    /// Kago's view of its slot.
    Kago: EnemyLayout40 {
        words: [
            function, fast_animation, var_c, var_d, unread_0fb0, unread_0fb2, hits, parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            var_00, var_01, var_02, var_03, hit_counter,
        ],
        _: [u16; 27],
    }
}

/// Bank $A8 Kago functions, stored in `function`.
pub mod function {
    pub const NOTHING: u16 = 0xab7b;
    /// $A8:AB81, an RTL.
    pub const RTL: u16 = 0xab81;
}

/// Bank $A8 Kago instruction lists.
pub mod instruction_list {
    pub const SLOW: u16 = 0xab1e;
    pub const FAST: u16 = 0xab32;
}

/// Properties bit 13: process instructions.
const PROCESS_INSTRUCTIONS: u16 = 0x2000;
/// $7E:183E and $7E:1840: the earthquake type (BG1 only, 1 px, diagonal) and timer.
const EARTHQUAKE_TYPE: usize = 0x183e;
const EARTHQUAKE_TIMER: usize = 0x1840;
const DIAGONAL_1PX: u16 = 2;
const SHAKE_FRAMES: u16 = 16;
/// The big explosion death, and the Kago bug projectile ($86:D02E).
const BIG_EXPLOSION: u16 = 4;
const KAGO_BUG: u16 = 0xd02e;

/// Initialisation AI - Kago ($A8:AB46).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let index = runtime.word(words::CUR_ENEMY_INDEX);
    let key = Key::from_legacy(index);
    runtime.edit_actor(key, Native, |actor| {
        actor.properties |= PROCESS_INSTRUCTIONS;
        actor.instruction_delay = 1;
        actor.timer = 0;
        actor.variables[4] = 0;
        actor.instruction = instruction_list::SLOW;
        actor.variables[0] = function::NOTHING;
        actor.pre_instruction = 0;
    });
    let hits = runtime.resolve_actor(key, Native).parameters[0];
    session_slot!(runtime, Kago[i32::from(index)].hit_counter = hits);
}
/// Main AI - Kago ($A8:AB75): Kago function - nothing ($A8:AB7B) becomes an
/// RTL. Was `advance`.
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    let key = Key::from_legacy(runtime.word(words::CUR_ENEMY_INDEX));
    match runtime.resolve_actor(key, Native).variables[0] {
        function::NOTHING => {
            runtime.edit_actor(key, Native, |actor| actor.variables[0] = function::RTL)
        }
        function::RTL => (),
        _ => runtime.unreachable(),
    }
}
/// Enemy shot - Kago ($A8:AB83): shake the screen, speed up, count the hit,
/// die at the last, and release a bug.
pub fn shot<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    combat::shot_reaction(runtime, ShotHandling::Finalize);
    runtime.write_slice_le(EARTHQUAKE_TYPE, DIAGONAL_1PX);
    runtime.write_slice_le(EARTHQUAKE_TIMER, SHAKE_FRAMES);
    // Damage may select another actor. Retain that identity through death,
    // then use the live selection only for the projectile's graphics source.
    let index = runtime.word(words::CUR_ENEMY_INDEX);
    let key = Key::from_legacy(index);
    if runtime.resolve_actor(key, Native).variables[1] == 0 {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[1] = 1;
            actor.instruction = instruction_list::FAST;
            actor.instruction_delay = 1;
        });
    }
    let kago = i32::from(index);
    let hits = session_slot!(runtime, Kago[kago].hit_counter).wrapping_sub(1);
    session_slot!(runtime, Kago[kago].hit_counter = hits);
    if (hits as i16) < 0 {
        combat::die(
            runtime,
            DeathKind::Normal {
                animation: BIG_EXPLOSION,
            },
        );
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 1);
    }
    let y = runtime.resolve_actor(key, Native).y.whole;
    let current = runtime.word(words::CUR_ENEMY_INDEX);
    runtime.foreign(|game| {
        game.spawn_projectile(y, current, KAGO_BUG);
    });
}
