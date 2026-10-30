//! Draygon's eye, which looks at Samus and puffs breath bubbles (bank
//! $A5:C46B-$A5:C598), and the function-setting instructions.
//! Common actor fields use the existing owner; outer boss dispatch stays in bank A5.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::draygon_host::{BODY, EYE, function, instruction_list as list};
use crate::actors::host::{ActorLease, ActorRuntime as _, BossEffects as _, LeaseActors};
use crate::actors::queries::angle_from_delta;
use crate::enemy_script::EnemyScript;
use crate::ram::words;

/// Every 80h frames the eye puffs long breath bubbles (dust cloud / explosion
/// $86:E509, parameter 18h) from 18h px ahead of and 20h px above it.
const BUBBLE_FRAMES: u16 = 0x7f;
const DUST_CLOUD: u16 = 0xe509;
const LONG_BREATH_BUBBLES: u16 = 0x18;
const BUBBLE_X: u16 = 0x18;
const BUBBLE_Y: u16 = 0x20;
/// Angles are a byte; each quarter turn, from 20h before up, picks a look.
const EIGHTH_TURN: u16 = 0x20;
const QUARTER_TURN: usize = 0x40;

/// Initialisation AI - Draygon eye ($A5:C46B).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    let key = Key::from_legacy(runtime.word(words::cur_enemy_index));
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = list::EYE_LEFT_IDLE;
        actor.variables[0] = function::NONE;
    });
}
/// Instruction - enemy function = [[Y]] ($A5:9736), for actor `k`; with `k`
/// the body or the eye, Draygon body function = [[Y]] ($A5:9F57) or Draygon
/// eye function = [[Y]] ($A5:C47B). Was `set_mode`.
pub fn set_function<H: LeaseActors + ?Sized>(
    k: u16,
    script: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    let function = script.word(0);
    game.lease()
        .edit_actor(Key::from_legacy(k), Native, |actor| {
            actor.variables[0] = function
        });
    script.offset(1)
}
/// Draygon eye function - facing left ($A5:C48D) or right ($A5:C513): puff
/// breath bubbles and look towards Samus. Was `advance`.
pub fn look_at_samus<H: LeaseActors + ?Sized>(k: u16, right: bool, game: &mut H) {
    let mut runtime = game.lease();
    let x_offset = if right {
        BUBBLE_X
    } else {
        0u16.wrapping_sub(BUBBLE_X)
    };
    if runtime.resolve_actor(Key::from_legacy(k), Native).frame & BUBBLE_FRAMES == 0 {
        // The left-facing emitter follows the eye; the right-facing one follows
        // the body. Both steering calculations below use the body's live position.
        let (x, y) = {
            let origin = runtime.resolve_actor(
                Key::from_legacy(if right { BODY as u16 } else { EYE as u16 }),
                Native,
            );
            (
                origin.x.whole.wrapping_add(x_offset),
                origin.y.whole.wrapping_sub(BUBBLE_Y),
            )
        };
        runtime.foreign(|game| {
            game.set_projectile_spawn_point(x, y);
            game.spawn_room_projectile(DUST_CLOUD, LONG_BREATH_BUBBLES);
        });
    }
    let (x, y) = {
        let body = runtime.resolve_actor(Key::from_legacy(BODY as u16), Native);
        let x = runtime.word(words::samus_x_pos).wrapping_sub(body.x.whole);
        let y = runtime.word(words::samus_y_pos).wrapping_sub(body.y.whole);
        (x.wrapping_sub(x_offset), y.wrapping_add(BUBBLE_Y))
    };
    let angle = angle_from_delta(x, y);
    // The caller's k selects the emission timer; the post-callback live selector
    // chooses the eye animation and its previous-angle cache.
    let key = Key::from_legacy(runtime.word(words::cur_enemy_index));
    if angle != runtime.resolve_actor(key, Native).pre_instruction {
        let instructions = if right {
            [
                list::EYE_RIGHT_LOOKING_UP,
                list::EYE_RIGHT_LOOKING_RIGHT,
                list::EYE_RIGHT_LOOKING_DOWN,
                list::EYE_RIGHT_LOOKING_LEFT,
            ]
        } else {
            [
                list::EYE_LEFT_LOOKING_UP,
                list::EYE_LEFT_LOOKING_RIGHT,
                list::EYE_LEFT_LOOKING_DOWN,
                list::EYE_LEFT_LOOKING_LEFT,
            ]
        };
        let sector = usize::from(angle.wrapping_add(EIGHTH_TURN) & 0xff) / QUARTER_TURN;
        runtime.edit_actor(key, Native, |actor| {
            actor.pre_instruction = angle;
            actor.restart_animation(instructions[sector]);
        });
    }
}
