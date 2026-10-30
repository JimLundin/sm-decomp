//! The metroid, with its electricity and shell sprite objects (bank $A3:E9AF-$A3:F310).
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess as _};
use crate::actors::api::{self, DeathHandling, DeathKind, ShotHandling, SlopePolicy};
use crate::actors::combat::{self, current_slot};
use crate::actors::host::{ActorLease, BossEffects as _, LeaseActors};
use crate::actors::session_slots::SlotAccess as _;
use crate::actors::{layouts::part_view, scheduling};
use crate::enemy_projectiles::host::Effects as _;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::table;
use crate::ram::{word_arrays, words};
use crate::session_access::SessionAccess as _;
use crate::session_slot;

part_view! {
    Metroid: EnemyLayout40 {
        words: [x_subvelocity, x_velocity, y_subvelocity, y_velocity, bombed_off_timer, function_index, parameter_1, freeze_threshold],
        pad1: [u8; 26696],
        words: [electricity_sprite, shell_sprite, samus_subhealth, var_03, var_04, var_05, var_06, var_07, var_08, var_09, var_0a,
            var_0b, var_0c, var_0d, var_0e, var_0f, var_10, var_11, var_12, var_13, var_14, var_15, var_16, var_17, var_18,
            var_19, var_1a, var_1b, var_1c, var_1d, var_1e, var_1f],
    }
}

const CHASE_SAMUS: u16 = 0; // Function indices, in `function_index`.
const LATCH_ONTO_SAMUS: u16 = 1;
const LATCHED_ONTO_SAMUS: u16 = 2;
const BOMBED_OFF_SAMUS: u16 = 3;
const LISTS: [u16; 2] = [0xe9cf, 0xea25]; // Chasing, draining Samus.
const BOMBED_OFF_DISPLACEMENTS: u32 = 0xa3_ea3f; // X then Y, by frame.
const CRIES: u32 = 0xa3_ead6;
const SPRITES: [u16; 2] = [0x32, 0x34]; // Electricity, shell.
const FROZEN_PROGRAMS: [u16; 2] = [0xc3ba, 0xc4b6]; // Palette 6.
const DISABLE_BLUE_FLASHING: u16 = 0x13; // Samus command; 12h enables it.
const DRAINS: [u16; 3] = [0x3000, 0x6000, 0xc000]; // Subhealth: gravity, varia, power suit.
const PROJECTILE_KIND: u16 = 0xf00;
const MISSILE: u16 = 0x100;
const SUPER_MISSILE: u16 = 0x200;
const BOMB: u16 = 0x500;

fn sprite_at(metroid: i32, part: usize) -> usize {
    crate::ram_slot!(@at Metroid, metroid, electricity_sprite) + 2 * part
}
fn sprites(runtime: &impl ActorLease, metroid: i32) -> [u16; 2] {
    [0, 1].map(|part| runtime.slot_word(sprite_at(metroid, part)))
}
fn restart_animation(runtime: &mut impl ActorLease, draining: bool) {
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = LISTS[usize::from(draining)];
        actor.instruction_delay = 1;
    });
}
/// Metroid function index 0 - chase Samus ($A3:EC11), or 1 - latch onto Samus ($A3:ECDC).
fn pursue(runtime: &mut impl ActorLease, target_y: u16, close: bool) {
    let key = combat::current(runtime);
    for axis in [1, 0] {
        let lanes = axis * 2..axis * 2 + 2;
        let target = [runtime.session().player.x, target_y][axis];
        let velocity = runtime.edit_actor(key, Native, |actor| {
            let position = [actor.x.whole, actor.y.whole][axis];
            let pair = &mut actor.variables[lanes.clone()];
            let next = if close {
                // The distance's high byte, over the low bytes' difference, scaled.
                let bytes = [
                    (target.wrapping_sub(position) >> 8) as u8,
                    (target as u8).wrapping_sub(position.to_ne_bytes()[0]),
                ];
                let mut high = (u16::from_ne_bytes(bytes) & 0xff00) >> 3;
                if high & 0x1000 != 0 {
                    high |= 0xe000;
                }
                u32::from(high) << 16
            } else {
                let acceleration = i32::from((position.wrapping_sub(target) as i16) >> 2) << 8;
                ((u32::from(pair[1]) << 16) | u32::from(pair[0])).wrapping_sub(acceleration as u32)
            };
            pair[0] = next as u16;
            pair[1] = (next >> 16) as u16;
            let high = pair[1] as i16;
            if !(-3..3).contains(&high) {
                pair[1] = high.clamp(-3, 3) as u16;
                pair[0] = 0;
            }
            ((u32::from(pair[1]) << 16) | u32::from(pair[0])) as i32
        });
        let result = if axis == 1 {
            api::move_vertical(runtime, key, velocity)
        } else {
            api::move_horizontal(runtime, key, velocity, SlopePolicy::Ignore)
        };
        if result == api::MovementResult::Blocked {
            runtime.edit_actor(key, Native, |actor| actor.variables[lanes].fill(0));
        }
    }
}
/// Metroid function index 3 - bombed off Samus ($A3:EDAB): shake, then chase again.
fn bombed_off(runtime: &mut impl ActorLease) {
    let metroid = current_slot(runtime);
    let frame = (session_slot!(runtime, Metroid[metroid].bombed_off_timer) & 3) as isize;
    let dx = table(runtime, BOMBED_OFF_DISPLACEMENTS, frame);
    session_slot!(runtime, Metroid[metroid].base.x_pos, |x| x.wrapping_add(dx));
    let dy = table(runtime, BOMBED_OFF_DISPLACEMENTS, frame + 4);
    session_slot!(runtime, Metroid[metroid].base.y_pos, |y| y.wrapping_add(dy));
    let key = combat::current(runtime);
    let finished = runtime.edit_actor(key, Native, |actor| {
        actor.variables[..4].fill(0);
        let previous = actor.variables[4];
        actor.variables[4] = previous.wrapping_sub(1);
        if previous == 1 {
            actor.pre_instruction = CHASE_SAMUS;
        }
        previous == 1
    });
    if finished {
        restart_animation(runtime, false);
    }
}
/// Initialisation AI - metroid ($A3:EA4F): create its electricity and shell.
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let key = combat::current(runtime);
    let metroid = current_slot(runtime);
    runtime.edit_actor(key, Native, |actor| actor.instruction = LISTS[0]);
    for (part, kind) in SPRITES.into_iter().enumerate() {
        let actor = runtime.resolve_actor(key, Native);
        let (x, y, appearance) = (actor.x.whole, actor.y.whole, actor.tile | actor.palette);
        let handle = runtime.foreign(|game| game.create_sprite(x, y, kind, appearance));
        runtime.set_slot_word(sprite_at(metroid, part), handle);
    }
    session_slot!(runtime, Metroid[metroid].samus_subhealth = 0);
    runtime.probe("sprite.metroid.init");
}
/// Frozen AI - metroid ($A3:EAE6): flash while bombed off, and freeze its sprites.
pub fn frozen<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    scheduling::update_frozen_enemy(runtime);
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| {
        if actor.variables[4] != 0 {
            actor.variables[4] = actor.variables[4].wrapping_sub(1);
            actor.flash = 2;
        }
    });
    let handles = sprites(runtime, current_slot(runtime));
    for (handle, program) in handles.into_iter().zip(FROZEN_PROGRAMS) {
        runtime.foreign(|game| game.freeze_sprite_attachment(handle, 0xc00, program));
    }
    runtime.probe("sprite.metroid.frozen");
}
/// Hurt AI - metroid ($A3:EB33): flash its sprites with it.
pub fn hurt<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let actor = runtime.resolve_actor(combat::current(runtime), Native);
    let appearance = [0, actor.palette][usize::from(actor.flash & 2 != 0)];
    for handle in sprites(runtime, current_slot(runtime)) {
        runtime.foreign(|game| game.flash_sprite_attachment(handle, appearance));
    }
    runtime.probe("sprite.metroid.hurt");
}
/// Main AI - metroid ($A3:EB98): run its function, and carry its sprites.
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let key = combat::current(runtime);
    let target_y = runtime.session().player.y.wrapping_sub(8);
    match runtime.resolve_actor(key, Native).pre_instruction {
        CHASE_SAMUS => pursue(runtime, target_y, false),
        LATCH_ONTO_SAMUS => pursue(runtime, target_y, true),
        // Metroid function index 2 - latched onto Samus ($A3:ED8F).
        LATCHED_ONTO_SAMUS => {
            let x = runtime.session().player.x;
            runtime.edit_actor(key, Native, |actor| {
                (actor.x.whole, actor.y.whole) = (x, target_y);
                actor.variables[..4].fill(0);
            });
        }
        BOMBED_OFF_SAMUS => bombed_off(runtime),
        index => panic!("metroid function index {index} out of range"),
    }
    let actor = runtime.resolve_actor(key, Native);
    let appearance = actor.tile | actor.palette;
    for handle in sprites(runtime, current_slot(runtime)) {
        runtime.foreign(|game| game.attach_sprite(handle, key.byte_index(), appearance));
    }
    runtime.probe("sprite.metroid.main");
}
/// Knock the metroid back from Samus at `target_y`, or from projectile 0, and chase.
fn recoil(runtime: &mut impl ActorLease, contact: bool, target_y: u16) {
    let key = combat::current(runtime);
    let metroid = current_slot(runtime);
    session_slot!(runtime, Metroid[metroid]; x_subvelocity = 0, y_subvelocity = 0);
    for axis in [0, 1] {
        let target = match (contact, axis) {
            (true, 0) => runtime.session().player.x,
            (true, _) => target_y,
            (false, 0) => runtime.word(word_arrays::projectile_x_pos),
            (false, _) => runtime.word(word_arrays::projectile_y_pos),
        };
        let actor = runtime.resolve_actor(key, Native);
        let difference = [actor.x.whole, actor.y.whole][axis].wrapping_sub(target) as i16;
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[axis * 2 + 1] = if difference < 0 { 0xff00 } else { 0 }
        });
        // The scaled difference lands in the middle two bytes of the 16.16 velocity.
        let scale = if contact { 64 } else { 32 };
        let [low, high] = (difference as u16).wrapping_mul(scale).to_ne_bytes();
        session_slot!(byte runtime, Metroid[metroid].x_subvelocity[1 + 4 * axis] = low);
        session_slot!(byte runtime, Metroid[metroid].x_subvelocity[2 + 4 * axis] = high);
    }
    session_slot!(runtime, Metroid[metroid].function_index = CHASE_SAMUS);
    restart_animation(runtime, false);
}
/// Deal metroid contact damage to Samus ($A3:EECE): a health point as the subhealth wraps.
fn drain(runtime: &mut impl ActorLease, metroid: i32) {
    let suits = runtime.word(words::equipped_items);
    let amount = DRAINS[usize::from(suits & 0x20 == 0) + usize::from(suits & 0x21 == 0)];
    let previous = session_slot!(runtime, Metroid[metroid].samus_subhealth);
    let subhealth = previous.wrapping_sub(amount);
    session_slot!(runtime, Metroid[metroid].samus_subhealth = subhealth);
    if previous < amount {
        runtime.damage_player(1);
    }
}
/// Enemy touch - metroid ($A3:EDEB): knocked back, or drain Samus and latch on within 8 px.
pub fn touch<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let key = combat::current(runtime);
    let metroid = current_slot(runtime);
    let target_y = runtime.session().player.y.wrapping_sub(8);
    let index = runtime.resolve_actor(key, Native).pre_instruction;
    if runtime.word(words::samus_contact_damage_index) != 0 {
        if index != LATCHED_ONTO_SAMUS {
            recoil(runtime, true, target_y);
        }
        return;
    }
    if index != BOMBED_OFF_SAMUS {
        if runtime.word(words::random_enemy_counter) & 7 == 7
            && runtime.word(words::samus_health).wrapping_sub(30) & 0x8000 == 0
        {
            runtime.queue_sound(3, 6, 0x2d); // Health tick.
        }
        drain(runtime, metroid);
    }
    if runtime.resolve_actor(key, Native).pre_instruction < LATCHED_ONTO_SAMUS {
        let actor = runtime.resolve_actor(key, Native);
        let dx = actor.x.whole.wrapping_sub(runtime.session().player.x) as i16;
        let near = dx.unsigned_abs() < 8
            && (actor.y.whole.wrapping_sub(target_y) as i16).unsigned_abs() < 8;
        if near {
            runtime.foreign(|game| game.command_player(0x12));
        }
        let next = [LATCH_ONTO_SAMUS, LATCHED_ONTO_SAMUS][usize::from(near)];
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = next);
        if near {
            restart_animation(runtime, true);
        }
    }
}
fn retire_sprites(runtime: &mut impl ActorLease, metroid: i32) -> u16 {
    let handles = sprites(runtime, metroid);
    for handle in handles {
        runtime.foreign(|game| game.retire_sprite_natively(handle));
    }
    handles[1]
}
/// Power bomb reaction - metroid ($A3:F042).
pub fn power_bomb<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let key = combat::current(runtime);
    let metroid = current_slot(runtime);
    combat::power_bomb_reaction(runtime, DeathHandling::Finalize);
    if runtime.resolve_actor(key, Native).health == 0 {
        runtime.foreign(|game| game.command_player(DISABLE_BLUE_FLASHING));
        retire_sprites(runtime, metroid);
    }
}
/// Enemy shot - metroid ($A3:EF07): shattered, bombed off Samus, or recoil and freeze.
pub fn shot<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let projectile = usize::from(runtime.session().room.collision_detection_index);
    let kind = runtime.word(word_arrays::projectile_type + 2 * projectile);
    let key = combat::current(runtime);
    let metroid = current_slot(runtime);
    if runtime.resolve_actor(key, Native).frozen != 0 {
        if matches!(kind & PROJECTILE_KIND, MISSILE | SUPER_MISSILE) {
            let x = runtime.resolve_actor(key, Native).x.whole;
            runtime.write_slice_le(words::special_death_item_drop_x_origin_pos, x);
            let y = runtime.resolve_actor(key, Native).y.whole;
            runtime.write_slice_le(words::special_death_item_drop_y_origin_pos, y);
            combat::shot_reaction(runtime, ShotHandling::ReactionOnly);
            if runtime.resolve_actor(key, Native).health == 0 {
                runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0);
                combat::die(runtime, DeathKind::Normal { animation: 4 }); // Big explosion.
                runtime.foreign(|game| game.command_player(DISABLE_BLUE_FLASHING));
                let handle = retire_sprites(runtime, metroid);
                runtime.foreign(|game| game.scatter_metroid_drop(handle));
            }
        }
    } else if runtime.resolve_actor(key, Native).pre_instruction == LATCHED_ONTO_SAMUS {
        if kind & PROJECTILE_KIND == BOMB {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[4] = 4; // Bombed off frames.
                actor.pre_instruction = BOMBED_OFF_SAMUS;
            });
            restart_animation(runtime, false);
            runtime.foreign(|game| game.command_player(DISABLE_BLUE_FLASHING));
        }
    } else {
        recoil(runtime, false, 0);
        if runtime.word(word_arrays::projectile_type + 2 * projectile) & 2 != 0 {
            runtime.queue_sound(3, 6, 0xa); // Ice beam: enemy frozen.
            let damage = runtime.word(word_arrays::projectile_damage + 2 * projectile);
            let threshold = session_slot!(runtime, Metroid[metroid].freeze_threshold);
            runtime.edit_actor(key, Native, |actor| actor.flash = 4);
            let threshold_left = threshold.saturating_sub(damage);
            session_slot!(runtime, Metroid[metroid].freeze_threshold = threshold_left);
            if damage >= threshold {
                runtime.edit_actor(key, Native, |actor| {
                    actor.frozen = 400;
                    actor.ai_handlers |= 4; // Frozen AI.
                });
                return;
            }
        }
        runtime.queue_sound(2, 6, 0x5a); // Shot metroid.
    }
}
/// Instruction - play Metroid draining Samus ($A3:EAA5).
pub fn draining_sound<H: LeaseActors + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    game.lease().queue_sound(2, 6, 0x50);
    script
}
/// Instruction - play random Metroid cry ($A3:EAB1).
pub fn random_cry<H: LeaseActors + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let index = runtime.next_random() & 7;
    let cry = table(runtime, CRIES, index as isize);
    runtime.queue_sound(2, 6, cry);
    script
}
