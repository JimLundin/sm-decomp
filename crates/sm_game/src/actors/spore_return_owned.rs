//! Spore descent/orbit/return shared by normal and Draygon alias dispatch.
#![forbid(unsafe_code)]
use crate::actors::host::{abs16, below};
use crate::actors::queries::angle_from_delta;
use crate::actors::remaining_owned::spore_spawn::{MIDDLE_X, MIDDLE_Y, Spore, function, list};
use crate::actors::session_slots::session_slot;
use crate::geometry::Vector32;
pub trait Host: crate::actors::draygon_flight::owned::Host {
    fn update_spore_stem(&mut self);
}
pub fn descend<L: Host + ?Sized>(game: &mut L) {
    game.update_spore_stem();
    let k = game.session().actors.controls.selected as i32;
    let y = session_slot!(game, Spore[k].base.y_pos).wrapping_add(1);
    session_slot!(game, Spore[k].base.y_pos = y);
    if !below(y, MIDDLE_Y) {
        session_slot!(game, Spore[k].base; current_instruction = list::DESCENDED, instruction_timer = 1);
    }
    session_slot!(game, Spore[0]; max_x_radius = 48, angle_delta = 1, angle = 192);
}
pub fn orbit<L: Host + ?Sized>(k: u16, game: &mut L) {
    game.update_spore_stem();
    let e = i32::from(k);
    let (angle, radius) = (
        session_slot!(game, Spore[0].angle),
        session_slot!(game, Spore[0].max_x_radius),
    );
    let x = session_slot!(game, Spore[e].origin_x).wrapping_add(game.scaled_cosine(angle, radius));
    session_slot!(game, Spore[e].base.x_pos = x);
    let phase = (2 * (i32::from(session_slot!(game, Spore[0].angle)) - 64)) as u16;
    let radius = session_slot!(game, Spore[0].max_x_radius).wrapping_sub(16);
    let y = session_slot!(game, Spore[e].origin_y).wrapping_add(game.scaled_sine(phase, radius));
    session_slot!(game, Spore[e].base.y_pos = y);
    let delta = session_slot!(byte game, Spore[0].angle_delta[0]);
    session_slot!(game, Spore[0].angle, |old| u16::from(
        delta.wrapping_add(old as u8)
    ));
}
pub fn begin_return<L: Host + ?Sized>(game: &mut L) {
    let dx = MIDDLE_X.wrapping_sub(session_slot!(game, Spore[0].base.x_pos));
    let dy = MIDDLE_Y.wrapping_sub(session_slot!(game, Spore[0].base.y_pos));
    let angle = 64u8.wrapping_sub(angle_from_delta(dx, dy) as u8);
    session_slot!(game, Spore[0].return_angle = u16::from(angle));
    let k = game.session().actors.controls.selected as i32;
    let angle = u16::from(session_slot!(byte game, Spore[0].return_angle[0]));
    let v = game.direction_step(angle, 1);
    session_slot!(game, Spore[k]; return_x_speed = (v.x >> 16) as u16, return_x_subspeed = v.x as u16,
        return_y_speed = (v.y >> 16) as u16, return_y_subspeed = v.y as u16);
}
pub fn finish_return<L: Host + ?Sized>(game: &mut L) {
    let k = game.session().actors.controls.selected as i32;
    let phase = u16::from(session_slot!(byte game, Spore[0].return_angle[0]));
    let joined = |high: u16, low: u16| (u32::from(high) << 16) | u32::from(low);
    let x = joined(
        session_slot!(game, Spore[k].return_x_speed),
        session_slot!(game, Spore[k].return_x_subspeed),
    );
    let y = joined(
        session_slot!(game, Spore[k].return_y_speed),
        session_slot!(game, Spore[k].return_y_subspeed),
    );
    game.set_step_scratch(phase);
    game.step_along(phase, Vector32 { x, y });
    let near = |at: u16, middle: u16| below(abs16(at.wrapping_sub(middle)), 8);
    if near(session_slot!(game, Spore[0].base.x_pos), MIDDLE_X)
        && near(session_slot!(game, Spore[0].base.y_pos), MIDDLE_Y)
    {
        session_slot!(game, Spore[0].function = function::NOTHING);
    }
    game.update_spore_stem();
    if game.frame() & 0xf == 0 {
        let random = game.roll_random();
        let (x, y) = ((random & 0x3f) + 96, ((random >> 8) & 0xf) + 480);
        game.create_sprite(x, y, 21, 0);
    }
}
