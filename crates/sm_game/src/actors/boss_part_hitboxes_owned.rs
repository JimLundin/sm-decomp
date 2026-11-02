//! Boss part hitboxes against Samus: a count word at bank $A9 followed by
//! eight-byte boxes of left, top, right and bottom extents.
#![forbid(unsafe_code)]
use crate::actors::mother_brain_fight_owned::Contact;
use crate::constants::bank_a9::ADDR_K_DEAD_TORIZO_HITBOX_D77C;
use sm_data::cartridge::RomRecord;
pub trait Host: crate::actors::boss_fields_owned::State {
    fn native_actor_record(&mut self, bank: u8, address: u16) -> RomRecord;
}

/// Each extent is chosen by Samus's side of the centre (`x`, `y`); the first
/// overlapping box sets the push words, clamped to at least 4, and stops.
fn push_out<H: Host>(boxes: u16, x: u16, y: u16, game: &mut H) -> bool {
    let table = game.native_actor_record(0xa9, boxes);
    let abs = |value: u16| i32::from((value as i16).unsigned_abs());
    for part in 0..usize::from(table.word(0)) {
        let at = 2 + 8 * part;
        let samus_y = game.session().player.y;
        let (distance, extent) = if (samus_y as i16).wrapping_sub(y as i16) >= 0 {
            (samus_y.wrapping_sub(y), table.word(at + 6))
        } else {
            (y.wrapping_sub(samus_y), table.word(at + 2))
        };
        if ((i32::from(game.session().player.contact.radius) + abs(extent) - i32::from(distance))
            as i16)
            < 0
        {
            continue;
        }
        let samus_x = game.session().player.x;
        let (distance, extent) = if (samus_x as i16).wrapping_sub(x as i16) >= 0 {
            (samus_x.wrapping_sub(x), table.word(at + 4))
        } else {
            (x.wrapping_sub(samus_x), table.word(at))
        };
        let mut overlap = (i32::from(game.session().player.contact.x_radius) + abs(extent)
            - i32::from(distance)) as i16;
        if overlap < 0 {
            continue;
        }
        if (i32::from(overlap) - 4) & 0x8000 != 0 {
            overlap = 4;
        }
        game.session_mut().player.contact.external_x.whole = overlap as u16;
        game.session_mut().player.contact.external_y.whole = 4;
        game.session_mut().player.contact.external_x.fraction = 0;
        game.session_mut().player.contact.external_y.fraction = 0;
        return true;
    }
    false
}

/// Mother Brain's parts also knock Samus back; right of the body she is hurt.
pub fn samus_contact<H: Host>(boxes: u16, x: u16, y: u16, game: &mut H) -> Contact {
    if !push_out(boxes, x, y, game) {
        return Contact::None;
    }
    game.session_mut().player.invincibility = 96;
    game.session_mut().player.knockback_timer = 5;
    game.session_mut().player.knockback_x_direction = 1;
    if (i32::from(game.session().player.y) - 192) & 0x8000 != 0 {
        game.session_mut().player.contact.y_direction = 2;
    }
    let body = i32::from(game.session().actors.slots[0].x.whole) + 24;
    if ((body - i32::from(game.session().player.x)) as i16) < 0 {
        Contact::Hurt
    } else {
        Contact::Pushed
    }
}

/// The dead Torizo's fixed box list pushes Samus out without knockback.
pub fn dead_torizo_contact<H: Host>(game: &mut H) -> bool {
    let (x, y) = (
        game.session().actors.slots[0].x.whole,
        game.session().actors.slots[0].y.whole,
    );
    push_out(ADDR_K_DEAD_TORIZO_HITBOX_D77C as u16, x, y, game)
}

impl Host for crate::actors::owned_execution::PreparedActorState<'_> {
    fn native_actor_record(&mut self, bank: u8, address: u16) -> RomRecord {
        crate::actors::owned_execution::PreparedActorState::native_actor_record(self, bank, address)
    }
}
