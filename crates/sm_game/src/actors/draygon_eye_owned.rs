//! Draygon eye and function instructions on owned actor/player state.
#![forbid(unsafe_code)]
use super::{
    BUBBLE_FRAMES, BUBBLE_X, BUBBLE_Y, DUST_CLOUD, EIGHTH_TURN, LONG_BREATH_BUBBLES, QUARTER_TURN,
};
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::draygon_host::{BODY, EYE, function, instruction_list as list};
use crate::actors::queries::angle_from_delta;
use crate::enemy_script::EnemyScript;
use crate::session_access::SessionAccess;
pub trait Host: ActorAccess + SessionAccess {
    fn place_projectile_spawn(&mut self, x: u16, y: u16);
    fn spawn_room_projectile(&mut self, definition: u16, parameter: u16) -> u16;
}
pub fn initialize(runtime: &mut (impl Host + ?Sized)) {
    let key = Key::from_legacy(runtime.session().actors.controls.selected);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = list::EYE_LEFT_IDLE;
        actor.variables[0] = function::NONE;
    });
}
pub fn set_function<H: Host + ?Sized>(k: u16, script: EnemyScript, game: &mut H) -> EnemyScript {
    let function = script.word(0);
    game.edit_actor(Key::from_legacy(k), Native, |actor| {
        actor.variables[0] = function
    });
    script.offset(1)
}
pub fn look_at_samus<H: Host + ?Sized>(k: u16, right: bool, game: &mut H) {
    let runtime = game;
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
        runtime.place_projectile_spawn(x, y);
        runtime.spawn_room_projectile(DUST_CLOUD, LONG_BREATH_BUBBLES);
    }
    let (x, y) = {
        let body = runtime.resolve_actor(Key::from_legacy(BODY as u16), Native);
        let x = runtime.session().player.x.wrapping_sub(body.x.whole);
        let y = runtime.session().player.y.wrapping_sub(body.y.whole);
        (x.wrapping_sub(x_offset), y.wrapping_add(BUBBLE_Y))
    };
    let angle = angle_from_delta(x, y);
    // The caller's k selects the emission timer; the post-callback live selector
    // chooses the eye animation and its previous-angle cache.
    let key = Key::from_legacy(runtime.session().actors.controls.selected);
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
