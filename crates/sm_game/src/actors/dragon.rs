//! Dragon, lava seahorses with wings (bank $A2:E57B-$A2:E977). Lane 3 is the timer; its
//! extended words hold the list index, current list index and fire flag.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorKey as Key};
use crate::actors::api;
use crate::actors::host::{ActorLease, Death, Shot};
use crate::actors::slot_view::{SlotState, slot_view};
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::table as list_table;

/// A lease the dragons run on: their slots are the Session's.
pub trait Lease: ActorLease + SlotState {}
impl<T: ActorLease + SlotState + ?Sized> Lease for T {}

slot_view! {
    /// A dragon's extended words: its instruction list index (bit 0 facing right),
    /// the index of the list it shows, and its animation finished (fire) flag.
    Dragon, DragonEdit {
        record: [],
        extended: [list_index = var_00, current_list_index = var_01, fire = var_02],
    }
}

const WAIT_TO_RISE: u16 = 0xe654;
const RISING: u16 = 0xe6ad;
const ATTACKING: u16 = 0xe6f1;
const WAIT_TO_SINK: u16 = 0xe734;
const SINKING: u16 = 0xe749;
const LISTS: u32 = 0xa2_e5ef; // instruction list pointers

/// Store `value` to the extended word `word` picks of dragon `k`, then refresh
/// the lease's observations after each byte.
fn store<H: Lease + ?Sized>(
    runtime: &mut H,
    k: u16,
    word: impl for<'e, 'h> FnOnce(&'e mut DragonEdit<'h, H>) -> &'e mut u16,
    value: u16,
) {
    *word(&mut DragonEdit::of(runtime, k)) = value;
    runtime.record_stored();
}

/// Set dragon (wings) instruction list ($A2:E782, $A2:E7A5): restart enemy `k`'s animation
/// from list pointer table `table` when its list index changed.
pub fn restart_from_list_cache(runtime: &mut impl Lease, k: u16, table: u32) {
    let dragon = Dragon::of(runtime, k);
    let list = dragon.list_index();
    if list != dragon.current_list_index() {
        store(runtime, k, |dragon| dragon.current_list_index(), list);
        let instruction = list_table(runtime, table, list as isize);
        let mut dragon = DragonEdit::of(runtime, k);
        let actor = dragon.actor();
        actor.instruction = instruction;
        actor.instruction_delay = 1;
        actor.timer = 0;
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
pub fn initialize(runtime: &mut impl crate::actors::species::TableLease) {
    let raw = runtime.session().actors.controls.selected;
    let key = Key::from_legacy(raw);
    store(runtime, raw, |dragon| dragon.fire(), 0);
    let wings = runtime.resolve_actor(key, Native).parameters[0] != 0;
    let list = if wings { 2 } else { 0 };
    store(runtime, raw, |dragon| dragon.list_index(), list);
    store(runtime, raw, |dragon| dragon.current_list_index(), list);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if wings { 0xe5a1 } else { 0xe59b };
        actor.properties |= if wings { 0x400 } else { 0 };
        actor.pre_instruction = if wings { 0xe781 } else { WAIT_TO_RISE };
    });
}

/// Instruction - set animation finished flag ($A2:E5FB): ready to fire.
pub fn set_animation_finished(
    cursor: EnemyScript,
    runtime: &mut impl crate::actors::species::TableLease,
) -> EnemyScript {
    let k = runtime.session().actors.controls.selected;
    store(runtime, k, |dragon| dragon.fire(), 1);
    cursor
}

/// Touch, shot and power bomb reactions ($A2:E7C8-E7D4), then the dragon reaction ($A2:E7DA):
/// the wings take the body's shake, invincibility, flash, frozen and AI handler words while
/// it lives, and become ignored by its death otherwise.
pub fn react(ea: u32, runtime: &mut impl crate::actors::species::TableLease) {
    match ea {
        0xa2_e7c8 => crate::actors::host::ActorRuntime::contact_reaction(runtime, Death::Finalize),
        0xa2_e7ce => crate::actors::host::ActorRuntime::shot_reaction(runtime, Shot::Finalize),
        _ => crate::actors::host::ActorRuntime::power_bomb_reaction(runtime, Death::Finalize),
    }
    let k = runtime.session().actors.controls.selected;
    let wings = k + 64;
    if Dragon::of(&*runtime, k).actor().health != 0 {
        let shake = Dragon::of(&*runtime, k).actor().shake;
        DragonEdit::of(runtime, wings).actor().shake = shake;
        let invincibility = Dragon::of(&*runtime, k).actor().invincibility;
        DragonEdit::of(runtime, wings).actor().invincibility = invincibility;
        let flash = Dragon::of(&*runtime, k).actor().flash;
        DragonEdit::of(runtime, wings).actor().flash = flash;
        let frozen = Dragon::of(&*runtime, k).actor().frozen;
        DragonEdit::of(runtime, wings).actor().frozen = frozen;
        let ai_handlers = Dragon::of(&*runtime, k).actor().ai_handlers;
        DragonEdit::of(runtime, wings).actor().ai_handlers = ai_handlers;
    } else {
        DragonEdit::of(runtime, wings).actor().properties |= 0x200;
    }
}

/// Dragon functions - wait to rise, rising, attacking, wait to sink and sinking ($A2:E654-E780).
pub fn step(k: u16, function: u16, runtime: &mut impl crate::actors::species::TableLease) {
    let key = Key::from_legacy(k);
    match function {
        WAIT_TO_RISE | RISING | SINKING => {
            if (tick(runtime, key).wrapping_sub(1) & 0x8000) != 0 {
                let (delay, next) = match function {
                    WAIT_TO_RISE => (48, RISING),
                    RISING => (3, ATTACKING),
                    _ => (128, WAIT_TO_RISE),
                };
                if function == RISING {
                    let list = Dragon::of(&*runtime, k).list_index().wrapping_add(4);
                    store(runtime, k, |dragon| dragon.list_index(), list);
                }
                set_timer(runtime, key, delay, next);
                if function == WAIT_TO_RISE {
                    let delta = api::player_delta(&*runtime, key, api::Axis::X);
                    let left = runtime.edit_actor(key, Native, |actor| {
                        actor.variables[0] = actor.variables[0] & 0x7fff | delta & 0x8000;
                        (actor.variables[0] & 0x8000) != 0
                    });
                    // Bit 0 of the list index faces right; the high byte is rewritten as is.
                    for part in [k, k + 64] {
                        let index = Dragon::of(&*runtime, part).list_index();
                        let index = if left { index & !1 } else { index | 1 };
                        store(runtime, part, |dragon| dragon.list_index(), index);
                    }
                    restart_from_list_cache(runtime, k, LISTS);
                    restart_from_list_cache(runtime, k + 64, LISTS);
                }
            }
            if function != WAIT_TO_RISE {
                for part in [k, k + 64] {
                    let delta = if function == RISING { u16::MAX } else { 1 };
                    let mut dragon = DragonEdit::of(runtime, part);
                    let y = &mut dragon.actor().y.whole;
                    *y = y.wrapping_add(delta);
                }
            }
        }
        ATTACKING => {
            restart_from_list_cache(runtime, k, LISTS);
            if Dragon::of(&*runtime, k).fire() != 0 {
                store(runtime, k, |dragon| dragon.fire(), 0);
                store(runtime, k, |dragon| dragon.current_list_index(), u16::MAX);
                crate::actors::boss_lease::BossLease::owned_step(runtime, |runtime| {
                    crate::enemy_projectiles::lifecycle::spawn_from_enemy(
                        runtime, 0xffff, k, 0xb5cb,
                    )
                });
                crate::enemy_projectiles::host::Effects::queue_sound(runtime, 2, 6, 0x61);
                if tick(runtime, key) == 1 {
                    let list = Dragon::of(&*runtime, k).list_index().wrapping_sub(4);
                    store(runtime, k, |dragon| dragon.list_index(), list);
                    set_timer(runtime, key, 96, WAIT_TO_SINK);
                }
            }
        }
        WAIT_TO_SINK => {
            if tick(runtime, key) == 1 {
                set_timer(runtime, key, 48, SINKING);
                restart_from_list_cache(runtime, k, LISTS);
            }
        }
        _ => unreachable!("unknown paired seahorse phase"),
    }
}
