//! Kraid's lint, the belly spikes Kraid fires at Samus: its collision with
//! Samus against the lint hitbox (bank $A7:B96A).
#![forbid(unsafe_code)]
use crate::actors::access::ActorAccess;
use crate::actors::host::Death;
use crate::actors::layouts::KraidLayout as Kraid;
use crate::actors::session_slots::SessionSlots;
use crate::actors::session_slots::session_slot;
use crate::constants::enemy_properties::K_ENEMY_PROPS_TANGIBLE;

const BANK: u8 = 0xa7;
/// The entry of "Hitbox - Kraid lint" ($A7:92B5) after its count word: left,
/// top, right and bottom words, each read where the original read it.
const HITBOX: u16 = 0x92b7;
const LEFT: usize = 0;
const TOP: usize = 2;
const BOTTOM: usize = 6;
/// The box is narrowed by this many pixels at the left, top and bottom.
const INSET: i32 = 2;
/// Samus is pushed out at most this far.
const MAX_PUSH: u16 = 16;
const SIGN: i32 = 0x8000;

fn hitbox(field: usize, game: &mut (impl Host + ?Sized)) -> i32 {
    i32::from(game.actor_record(BANK, HITBOX).word(field))
}

/// Push Samus out of the lint's box, clamp her extra displacement and make the
/// lint tangible; sign tests are on the low 16 bits as in the original.
/// Kraid lint / Samus collision handling ($A7:B96A). Was `Kraid_EnemyTouch_Lint`.
pub fn lint_touch_samus(k: u16, game: &mut (impl Host + ?Sized)) {
    let e = i32::from(k);
    if session_slot!(game, Kraid[e].base.properties) as i32 & K_ENEMY_PROPS_TANGIBLE as i32 != 0
        || game.session().player.invincibility != 0
    {
        return;
    }
    let (samus_x, x_radius) = (
        i32::from(game.session().player.x),
        i32::from(game.session().player.contact.x_radius),
    );
    let left = (hitbox(LEFT, game) + i32::from(session_slot!(game, Kraid[e].base.x_pos)) - INSET)
        as u16 as i32;
    if (x_radius + samus_x - left) & SIGN != 0 || (samus_x - x_radius - left) & SIGN == 0 {
        return;
    }
    let top = (hitbox(TOP, game) + i32::from(session_slot!(game, Kraid[e].base.y_pos)) + INSET)
        as u16 as i32;
    let (samus_y, y_radius) = (
        i32::from(game.session().player.y),
        i32::from(game.session().player.contact.radius),
    );
    if (y_radius + samus_y - top) & SIGN != 0 {
        return;
    }
    let bottom = (hitbox(BOTTOM, game) + i32::from(session_slot!(game, Kraid[e].base.y_pos))
        - INSET) as u16 as i32;
    if (samus_y - y_radius - bottom) & SIGN == 0 {
        return;
    }
    let mut push = (i32::from(game.session().player.contact.external_x.whole)
        + !(x_radius + i32::from(MAX_PUSH))) as u16;
    if (push as i32 - i32::from(MAX_PUSH)) & SIGN == 0 {
        push = MAX_PUSH;
    }
    game.session_mut().player.contact.external_x.whole = push;
    game.react_to_contact(Death::Finalize);
    session_slot!(game, Kraid[e].base.properties, |old| old
        | K_ENEMY_PROPS_TANGIBLE as u16);
}

/// Content capture and contact reaction occur at their original call sites.
pub trait Host: ActorAccess + SessionSlots {
    fn actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord;
    fn react_to_contact(&mut self, death: Death);
}
