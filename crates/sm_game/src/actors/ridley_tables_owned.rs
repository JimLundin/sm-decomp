//! Ridley pogo selection and ordered native tail copies on owned actor state.
#![forbid(unsafe_code)]
use crate::actors::ridley::{Ridley, owned::Host};
use crate::actors::session_slots::session_slot;

pub fn set_pogo_speeds<H: Host + ?Sized>(game: &mut H) {
    let pick = isize::try_from(game.session().timing.random_number & 3).expect("pogo row");
    let x_row = game.pogo_row(true, pick);
    let y_row = game.pogo_row(false, pick);
    let offset = session_slot!(game, Ridley[0].acceleration_index)
        .wrapping_add(2)
        .wrapping_mul(2);
    let step = isize::try_from(offset >> 1).expect("pogo acceleration");
    let upwards = game.pogo_acceleration(true, step);
    let [low, high] = upwards.to_le_bytes();
    game.set_tail_byte(14, low);
    game.set_tail_byte(15, high);
    let downwards = game.pogo_acceleration(false, step);
    let [low, high] = downwards.to_le_bytes();
    game.set_tail_byte(12, low);
    game.set_tail_byte(13, high);
    let y = game.actor_record(0xa6, y_row).word(usize::from(offset));
    session_slot!(game, Ridley[0].y_velocity = y);
    let speed = game.actor_record(0xa6, x_row).word(usize::from(offset));
    let signed = if session_slot!(game, Ridley[0].x_velocity) & 0x8000 != 0 {
        speed.wrapping_neg()
    } else {
        speed
    };
    session_slot!(game, Ridley[0].x_velocity = signed);
}

pub fn set_tail_segments_from_list<H: Host + ?Sized>(destination: u16, source: u16, game: &mut H) {
    let words = game.actor_record(0xa6, source);
    for segment in 0..7 {
        let [first, second] = words.word(2 * segment).to_ne_bytes();
        game.set_tail_destination_byte(destination, 20 * segment, first);
        game.set_tail_destination_byte(destination, 20 * segment + 1, second);
    }
}
