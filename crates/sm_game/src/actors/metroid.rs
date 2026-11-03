//! The metroid, with its electricity and shell sprite objects (bank $A3:E9AF-$A3:F310).
use crate::actors::access::{AccessMode::Native, ActorAccess as _};
use crate::actors::api::{self, DeathHandling, DeathKind, ShotHandling, SlopePolicy};
use crate::actors::combat;
use crate::actors::host::{ActorLease, BossEffects as _, LeaseActors};
use crate::actors::scheduling;
use crate::actors::slot_view::{SlotState, slot_view};
use crate::enemy_projectiles::host::Effects as _;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::{access::shot, content::table, host::Host as _};
use crate::session_access::SessionAccess as _;

/// A lease the metroids run on: their slots are the Session's.
pub trait Lease: ActorLease + SlotState {}
impl<T: ActorLease + SlotState + ?Sized> Lease for T {}

slot_view! {
    /// A metroid's slot: its X and Y velocities (16.16, in the actor's first
    /// four words), the bombed off timer, function index and freeze threshold;
    /// its electricity and shell sprites and Samus's drained subhealth.
    Metroid, MetroidEdit {
        record: [
            x_subvelocity = var_a, y_subvelocity = var_c, bombed_off_timer = var_e,
            function_index = var_f, freeze_threshold = parameter_2,
        ],
        extended: [electricity_sprite = var_00, shell_sprite = var_01, samus_subhealth = var_02],
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

fn sprites(runtime: &impl Lease, metroid: u16) -> [u16; 2] {
    let view = Metroid::of(runtime, metroid);
    [view.electricity_sprite(), view.shell_sprite()]
}
fn restart_animation(runtime: &mut impl Lease, draining: bool) {
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = LISTS[usize::from(draining)];
        actor.instruction_delay = 1;
    });
}
/// Metroid function index 0 - chase Samus ($A3:EC11), or 1 - latch onto Samus ($A3:ECDC).
fn pursue(runtime: &mut impl Lease, target_y: u16, close: bool) {
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
fn bombed_off(runtime: &mut impl Lease) {
    let metroid = combat::current(runtime).byte_index();
    let frame = (Metroid::of(runtime, metroid).bombed_off_timer() & 3) as isize;
    let dx = table(runtime, BOMBED_OFF_DISPLACEMENTS, frame);
    {
        let mut edit = MetroidEdit::of(runtime, metroid);
        let old = edit.actor().x.whole;
        edit.actor().x.whole = old.wrapping_add(dx);
    }
    let dy = table(runtime, BOMBED_OFF_DISPLACEMENTS, frame + 4);
    {
        let mut edit = MetroidEdit::of(runtime, metroid);
        let old = edit.actor().y.whole;
        edit.actor().y.whole = old.wrapping_add(dy);
    }
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
    let runtime = &mut game.lease();
    let key = combat::current(runtime);
    let metroid = combat::current(runtime).byte_index();
    runtime.edit_actor(key, Native, |actor| actor.instruction = LISTS[0]);
    for (part, kind) in SPRITES.into_iter().enumerate() {
        let actor = runtime.resolve_actor(key, Native);
        let (x, y, appearance) = (actor.x.whole, actor.y.whole, actor.tile | actor.palette);
        let handle = runtime.foreign(|game| game.create_sprite(x, y, kind, appearance));
        let mut edit = MetroidEdit::of(runtime, metroid);
        *match part {
            0 => edit.electricity_sprite(),
            _ => edit.shell_sprite(),
        } = handle;
    }
    *MetroidEdit::of(runtime, metroid).samus_subhealth() = 0;
    runtime.probe("sprite.metroid.init");
}
/// Frozen AI - metroid ($A3:EAE6): flash while bombed off, and freeze its sprites.
pub fn frozen<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    scheduling::update_frozen_enemy(runtime);
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| {
        if actor.variables[4] != 0 {
            actor.variables[4] = actor.variables[4].wrapping_sub(1);
            actor.flash = 2;
        }
    });
    let handles = sprites(runtime, combat::current(runtime).byte_index());
    for (handle, program) in handles.into_iter().zip(FROZEN_PROGRAMS) {
        runtime.foreign(|game| game.freeze_sprite_attachment(handle, 0xc00, program));
    }
    runtime.probe("sprite.metroid.frozen");
}
/// Hurt AI - metroid ($A3:EB33): flash its sprites with it.
pub fn hurt<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let actor = runtime.resolve_actor(combat::current(runtime), Native);
    let appearance = [0, actor.palette][usize::from(actor.flash & 2 != 0)];
    for handle in sprites(runtime, combat::current(runtime).byte_index()) {
        runtime.foreign(|game| game.flash_sprite_attachment(handle, appearance));
    }
    runtime.probe("sprite.metroid.hurt");
}
/// Main AI - metroid ($A3:EB98): run its function, and carry its sprites.
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    crate::probe::hit("metroid.main_ai");
    let runtime = &mut game.lease();
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
    for handle in sprites(runtime, combat::current(runtime).byte_index()) {
        runtime.foreign(|game| game.attach_sprite(handle, key.byte_index(), appearance));
    }
    runtime.probe("sprite.metroid.main");
}
/// Knock the metroid back from Samus at `target_y`, or from projectile 0, and chase.
fn recoil(runtime: &mut impl Lease, contact: bool, target_y: u16) {
    let key = combat::current(runtime);
    let metroid = combat::current(runtime).byte_index();
    {
        let mut edit = MetroidEdit::of(runtime, metroid);
        *edit.x_subvelocity() = 0;
        *edit.y_subvelocity() = 0;
    }
    for axis in [0, 1] {
        let target = match (contact, axis) {
            (true, 0) => runtime.session().player.x,
            (true, _) => target_y,
            (false, 0) => runtime.session().projectiles.shots[0].x,
            (false, _) => runtime.session().projectiles.shots[0].y,
        };
        let actor = runtime.resolve_actor(key, Native);
        let difference = [actor.x.whole, actor.y.whole][axis].wrapping_sub(target) as i16;
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[axis * 2 + 1] = if difference < 0 { 0xff00 } else { 0 }
        });
        // The scaled difference lands in the middle two bytes of the 16.16 velocity.
        let scale = if contact { 64 } else { 32 };
        // Its low byte is the subvelocity's high byte, its high byte the velocity's low byte.
        let [low, high] = (difference as u16).wrapping_mul(scale).to_le_bytes();
        let mut edit = MetroidEdit::of(runtime, metroid);
        let lanes = &mut edit.actor().variables;
        lanes[2 * axis] = (lanes[2 * axis] & 0xff) | (u16::from(low) << 8);
        lanes[2 * axis + 1] = (lanes[2 * axis + 1] & 0xff00) | u16::from(high);
    }
    *MetroidEdit::of(runtime, metroid).function_index() = CHASE_SAMUS;
    restart_animation(runtime, false);
}
/// Deal metroid contact damage to Samus ($A3:EECE): a health point as the subhealth wraps.
fn drain(runtime: &mut impl Lease, metroid: u16) {
    let suits = runtime.session().inventory.equipment.active;
    let amount = DRAINS[usize::from(suits & 0x20 == 0) + usize::from(suits & 0x21 == 0)];
    let previous = Metroid::of(runtime, metroid).samus_subhealth();
    let subhealth = previous.wrapping_sub(amount);
    *MetroidEdit::of(runtime, metroid).samus_subhealth() = subhealth;
    if previous < amount {
        runtime.damage_player(1);
    }
}
/// Enemy touch - metroid ($A3:EDEB): knocked back, or drain Samus and latch on within 8 px.
pub fn touch<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let key = combat::current(runtime);
    let metroid = combat::current(runtime).byte_index();
    let target_y = runtime.session().player.y.wrapping_sub(8);
    let index = runtime.resolve_actor(key, Native).pre_instruction;
    if runtime.session().player.contact.contact_attack != 0 {
        if index != LATCHED_ONTO_SAMUS {
            recoil(runtime, true, target_y);
        }
        return;
    }
    if index != BOMBED_OFF_SAMUS {
        if runtime.session().actors.controls.phase_counter & 7 == 7
            && runtime.session().inventory.health.current.wrapping_sub(30) & 0x8000 == 0
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
fn retire_sprites(runtime: &mut impl Lease, metroid: u16) -> u16 {
    let handles = sprites(runtime, metroid);
    for handle in handles {
        runtime.foreign(|game| game.retire_sprite_natively(handle));
    }
    handles[1]
}
/// Power bomb reaction - metroid ($A3:F042).
pub fn power_bomb<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let key = combat::current(runtime);
    let metroid = combat::current(runtime).byte_index();
    combat::power_bomb_reaction(runtime, DeathHandling::Finalize);
    if runtime.resolve_actor(key, Native).health == 0 {
        runtime.foreign(|game| game.command_player(DISABLE_BLUE_FLASHING));
        retire_sprites(runtime, metroid);
    }
}
/// Enemy shot - metroid ($A3:EF07): shattered, bombed off Samus, or recoil and freeze.
pub fn shot<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let projectile = runtime
        .session()
        .room
        .collision_detection_index
        .wrapping_mul(2);
    let kind = shot!(runtime, projectile, kind);
    let key = combat::current(runtime);
    let metroid = combat::current(runtime).byte_index();
    if runtime.resolve_actor(key, Native).frozen != 0 {
        if matches!(kind & PROJECTILE_KIND, MISSILE | SUPER_MISSILE) {
            let x = runtime.resolve_actor(key, Native).x.whole;
            runtime.session_mut().enemy_projectiles.drop_origin.x = x;
            runtime.words_stored();
            let y = runtime.resolve_actor(key, Native).y.whole;
            runtime.session_mut().enemy_projectiles.drop_origin.y = y;
            runtime.words_stored();
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
        if shot!(runtime, projectile, kind) & 2 != 0 {
            runtime.queue_sound(3, 6, 0xa); // Ice beam: enemy frozen.
            let damage = shot!(runtime, projectile, damage);
            let threshold = Metroid::of(runtime, metroid).freeze_threshold();
            runtime.edit_actor(key, Native, |actor| actor.flash = 4);
            let threshold_left = threshold.saturating_sub(damage);
            *MetroidEdit::of(runtime, metroid).freeze_threshold() = threshold_left;
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
pub fn random_cry(script: EnemyScript, runtime: &mut impl Lease) -> EnemyScript {
    let index = runtime.next_random() & 7;
    let cry = table(runtime, CRIES, index as isize);
    runtime.queue_sound(2, 6, cry);
    script
}
