//! Reflec, the mirror enemy that bounces Samus's beams off at an angle and
//! glows (bank $A3:DA9C-$A3:DF1A).
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::api::ShotHandling;
use crate::actors::combat;
use crate::actors::host::{ActorLease, LeaseActors};
use crate::enemy_projectiles::access::{set_shot_direction, shot_word};
use crate::enemy_projectiles::host::ShotMember;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::{access::shot, content, host::Host as _, motion};
use crate::ram::{word_arrays, words};
use crate::session_access::SessionAccess as _;

/// $A3:DABC: the palette cycle colours, 4 a frame of 8; $A3:DC0B: the initial
/// instruction lists; $A3:DCAE: the reflected direction for each reflection
/// axis and shot direction, 8000h to absorb; $A3:DCA6: the health reached
/// zero instruction lists, by axis.
const CYCLE_COLOURS: u32 = 0xa3_dabc;
const INITIAL_LISTS: u32 = 0xa3_dc0b;
const REFLECTIONS: u32 = 0xa3_dcae;
const DEATH_LISTS: u32 = 0xa3_dca6;
const ABSORB: u16 = 0x8000;
/// $7E:178C: the enemy graphics drawn hook, the reflec palette cycle ($A3:DB0C).
const GFX_DRAWN_HOOK: usize = 0x178c;
const GFX_DRAWN_HOOK_BANK: usize = 0x178e;
const PALETTE_CYCLE_HOOK: u16 = 0xdb0c;
const BANK: u16 = 0xa3;
/// The hook's variables: the palette's byte offset, the colour set and the timer.
const HOOK_PALETTE: usize = 0;
const HOOK_COLOUR_SET: usize = 2;
const HOOK_TIMER: usize = 4;
const CYCLE_FRAMES: u16 = 16;
const COLOUR_SETS: u16 = 7;
const COLOURS: usize = 4;
/// Colours 9-12 of the enemy's palette, past the palette buffer's 128 BG colours.
const FIRST_GLOW_COLOUR: usize = 137;
/// Properties bit 12: block plasma beam.
const BLOCK_PLASMA: u16 = 0x1000;
/// A shot leaves the reflec invincible for 10 frames.
const INVINCIBLE_FRAMES: u16 = 10;
/// Each reflection axis has 16 directions; a shot's direction is its low nibble.
const DIRECTIONS_PER_AXIS: u16 = 32;
const DIRECTION_BITS: u16 = 15;
/// Projectile kind bit 15 is cleared by a reflection.
const KIND_BIT: u16 = 0x8000;
/// The reflection sound, 57h, library 2.
const REFLECT_SOUND: u16 = 0x57;
const SIGN: u16 = 0x8000;

/// Enemy graphics drawn hook - reflec - periodically cycle between palettes
/// ($A3:DB0C). Was `palette`.
pub fn cycle_palette<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let hook = word_arrays::variables_for_enemy_graphics_drawn_hook;
    if runtime.word(words::door_transition_flag_enemies) != 0 {
        return;
    }
    runtime.write_slice_le(
        hook + HOOK_TIMER,
        runtime.word(hook + HOOK_TIMER).wrapping_sub(1),
    );
    if runtime.word(hook + HOOK_TIMER) != 0 {
        return;
    }
    runtime.write_slice_le(hook + HOOK_TIMER, CYCLE_FRAMES);
    let mut destination = runtime.word(hook + HOOK_PALETTE);
    let mut source = runtime.word(hook + HOOK_COLOUR_SET).wrapping_mul(8);
    let image = runtime.leased_image().expect("ROM must be loaded");
    for _ in 0..COLOURS {
        let colour = image.table_word(content::offset(CYCLE_COLOURS), (source >> 1) as isize);
        runtime.write_slice_native(
            word_arrays::palette_buffer + 2 * (usize::from(destination >> 1) + FIRST_GLOW_COLOUR),
            colour,
        );
        source = source.wrapping_add(2);
        destination = destination.wrapping_add(2);
    }
    runtime.write_slice_le(
        hook + HOOK_COLOUR_SET,
        (u16::from(runtime.lease_byte(hook + HOOK_COLOUR_SET)) + 1) & COLOUR_SETS,
    );
}
/// Initialisation AI - reflec ($A3:DBD3).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| actor.properties |= BLOCK_PLASMA);
    let axis = runtime.resolve_actor(key, Native).parameters[0];
    let image = runtime.leased_image().expect("ROM must be loaded");
    let instruction = image.table_word(content::offset(INITIAL_LISTS), axis as isize);
    runtime.edit_actor(key, Native, |actor| actor.instruction = instruction);
    runtime.write_slice_native(GFX_DRAWN_HOOK, PALETTE_CYCLE_HOOK);
    runtime.write_slice_native(GFX_DRAWN_HOOK_BANK, BANK);
    let palette =
        ((16 * u32::from(runtime.resolve_actor(key, Native).palette) & 0xff00) >> 8) as u16;
    runtime.write_slice_le(
        word_arrays::variables_for_enemy_graphics_drawn_hook + HOOK_PALETTE,
        palette,
    );
    runtime.write_slice_le(
        word_arrays::variables_for_enemy_graphics_drawn_hook + HOOK_TIMER,
        CYCLE_FRAMES,
    );
}
/// Enemy shot - reflec ($A3:DC1C), its first half: reflect the shot by the
/// reflection axis, or mark it to be absorbed and return the reflection
/// target. A redirected target requires the retained native diagnostic
/// before absorption. Was `prepare_shot`.
pub fn reflect_shot<H: LeaseActors + ?Sized>(game: &mut H) -> Option<u16> {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let raw = runtime
        .word(words::collision_detection_index)
        .wrapping_mul(2);
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| actor.invincibility = INVINCIBLE_FRAMES);
    let axis = runtime.resolve_actor(key, Native).parameters[1];
    let shot_index = (raw >> 1) as isize;
    let target = axis
        .wrapping_mul(DIRECTIONS_PER_AXIS)
        .wrapping_add((shot_word(runtime, ShotMember::Direction, shot_index) & DIRECTION_BITS) * 2);
    let index = (target >> 1) as isize;
    let image = runtime.leased_image().expect("ROM must be loaded");
    if image.table_word(content::offset(REFLECTIONS), index) == ABSORB {
        combat::mark_shot(runtime, shot_index);
        Some(target)
    } else {
        let mut direction = image.table_word(content::offset(REFLECTIONS), index);
        if direction & SIGN != 0 {
            let image = runtime.leased_image().expect("ROM must be loaded");
            direction = image
                .table_word(content::offset(REFLECTIONS), (target >> 1) as isize)
                .wrapping_neg();
        }
        set_shot_direction(runtime, shot_index, direction);
        shot!(runtime, raw, kind = shot!(runtime, raw, kind) & !KIND_BIT; SliceLe);
        runtime.owned_lease_call(|runtime| motion::reflect(runtime, raw));
        runtime.sound(2, 6, REFLECT_SOUND);
        None
    }
}
/// Enemy shot - reflec, its second half: the shot hurts it, and at zero
/// health it shows its health reached zero instruction list. Was `absorb`.
pub fn absorb_shot<H: LeaseActors + ?Sized>(raw: u16, game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let target = Key::from_legacy(raw);
    if runtime.resolve_actor(target, Native).health != 0 {
        combat::shot_reaction(runtime, ShotHandling::ReactionOnly);
        if runtime.resolve_actor(target, Native).health == 0 {
            let axis = runtime.resolve_actor(target, Native).parameters[1];
            let image = runtime.leased_image().expect("ROM must be loaded");
            let instruction = image.table_word(content::offset(DEATH_LISTS), axis as isize);
            runtime.edit_actor(target, Native, |actor| actor.restart_animation(instruction));
        }
    }
}
/// Instruction - enemy reflection axis = [[Y]] ($A3:DBC8). Was `set_parameter`.
pub fn set_reflection_axis<H: LeaseActors + ?Sized>(
    raw: u16,
    script: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    let axis = script.word(0);
    game.lease()
        .edit_actor(Key::from_legacy(raw), Native, |actor| {
            actor.parameters[1] = axis
        });
    script.offset(1)
}
