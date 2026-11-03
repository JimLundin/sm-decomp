//! Boss part hitboxes against Samus: a count word in bank $A9, then
//! eight-byte boxes of left, top, right and bottom extents. They run on the
//! Mother Brain fight's lease, over the Session's player.
use crate::actors::mother_brain_fight::{Contact, Lease};
use crate::actors::slot_view::SlotRef;

/// The dead Torizo's hitbox list ($A9:D77C).
const DEAD_TORIZO_HITBOXES: u16 = 0xd77c;
/// Samus's least push out of a box.
const LEAST_PUSH: u16 = 4;

/// Whether `value` is negative as a signed 16-bit word.
fn negative(value: u16) -> bool {
    value & 0x8000 != 0
}

/// Push Samus out of the first box of the list at `boxes` she overlaps, centred
/// on (`x`, `y`): each extent is chosen by her side of the centre, and the push
/// is at least 4. False when she overlaps none. The sums are the original's
/// 16-bit ones, compared as signed words.
fn push_out(boxes: u16, x: u16, y: u16, game: &mut impl Lease) -> bool {
    let table = crate::cartridge::native_record(0xa9, boxes, game);
    let magnitude = |value: u16| value.cast_signed().unsigned_abs();
    for part in 0..usize::from(table.word(0)) {
        let at = 2 + 8 * part;
        let samus_y = game.player_y();
        let (distance, extent) = if !negative(samus_y.wrapping_sub(y)) {
            (samus_y.wrapping_sub(y), table.word(at + 6))
        } else {
            (y.wrapping_sub(samus_y), table.word(at + 2))
        };
        let y_radius = game.player_y_radius();
        if negative(
            y_radius
                .wrapping_add(magnitude(extent))
                .wrapping_sub(distance),
        ) {
            continue;
        }
        let samus_x = game.player_x();
        let (distance, extent) = if !negative(samus_x.wrapping_sub(x)) {
            (samus_x.wrapping_sub(x), table.word(at + 4))
        } else {
            (x.wrapping_sub(samus_x), table.word(at))
        };
        let x_radius = game.session().player.contact.x_radius;
        let mut overlap = x_radius
            .wrapping_add(magnitude(extent))
            .wrapping_sub(distance);
        if negative(overlap) {
            continue;
        }
        if negative(overlap.wrapping_sub(LEAST_PUSH)) {
            overlap = LEAST_PUSH;
        }
        let contact = &mut game.session_mut().player.contact;
        contact.external_x.whole = overlap;
        contact.external_y.whole = LEAST_PUSH;
        contact.external_x.fraction = 0;
        contact.external_y.fraction = 0;
        return true;
    }
    false
}

/// Mother Brain's parts also knock Samus back; right of the body she is hurt.
pub fn samus_contact(boxes: u16, x: u16, y: u16, game: &mut impl Lease) -> Contact {
    if !push_out(boxes, x, y, game) {
        return Contact::None;
    }
    let player = &mut game.session_mut().player;
    player.invincibility = 96;
    player.knockback_timer = 5;
    player.knockback_x_direction = 1;
    if negative(game.player_y().wrapping_sub(192)) {
        game.session_mut().player.contact.y_direction = 2;
    }
    let body = SlotRef::of(game, 0).actor.x.whole.wrapping_add(24);
    if negative(body.wrapping_sub(game.player_x())) {
        Contact::Hurt
    } else {
        Contact::Pushed
    }
}

/// The dead Torizo's fixed box list pushes Samus out without knockback.
pub fn dead_torizo_contact(game: &mut impl Lease) -> bool {
    let torizo = SlotRef::of(game, 0).actor;
    push_out(DEAD_TORIZO_HITBOXES, torizo.x.whole, torizo.y.whole, game)
}
