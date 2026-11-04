//! Kago, the bug hive of Norfair, which shakes the screen and releases a bug
//! each time it is shot (bank $A8:AB1E-$A8:ABCF).
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::api::{DeathKind, ShotHandling};
use crate::actors::combat;
use crate::actors::host::{ActorLease, LeaseActors};
use crate::actors::slot_view::{SlotState, slot_view};
use crate::session_access::SessionAccess as _;

slot_view! {
    /// Kago's slot: the hits left before it dies.
    Kago, KagoEdit {
        record: [],
        extended: [hit_counter = var_04],
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
/// The earthquake type (BG1 only, 1 px, diagonal) and its frames.
const DIAGONAL_1PX: u16 = 2;
const SHAKE_FRAMES: u16 = 16;
/// The big explosion death, and the Kago bug projectile ($86:D02E).
const BIG_EXPLOSION: u16 = 4;
const KAGO_BUG: u16 = 0xd02e;

/// Initialisation AI - Kago ($A8:AB46).
pub fn initialize(runtime: &mut (impl ActorLease + SlotState)) {
    let index = runtime.session().actors.controls.selected;
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
    *KagoEdit::of(runtime, index).hit_counter() = hits;
}
/// Main AI - Kago ($A8:AB75): Kago function - nothing ($A8:AB7B) becomes an
/// RTL. Was `advance`.
pub fn main_ai(runtime: &mut (impl ActorLease + SlotState)) {
    let key = Key::from_legacy(runtime.session().actors.controls.selected);
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
    let feedback = &mut runtime.session_mut().feedback;
    feedback.earthquake_kind = DIAGONAL_1PX;
    feedback.earthquake_timer = SHAKE_FRAMES;
    // Damage may select another actor. Retain that identity through death,
    // then use the live selection only for the projectile's graphics source.
    let index = runtime.session().actors.controls.selected;
    let key = Key::from_legacy(index);
    if runtime.resolve_actor(key, Native).variables[1] == 0 {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[1] = 1;
            actor.instruction = instruction_list::FAST;
            actor.instruction_delay = 1;
        });
    }
    let hits = Kago::of(runtime, index).hit_counter().wrapping_sub(1);
    *KagoEdit::of(runtime, index).hit_counter() = hits;
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
    let current = runtime.session().actors.controls.selected;
    runtime.lease_owned_step(|runtime| {
        crate::enemy_projectiles::lifecycle::spawn_from_enemy(runtime, y, current, KAGO_BUG);
    });
}
